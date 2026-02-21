use std::thread::AccessError;

use secrecy::{ExposeSecret, SecretString};

use thiserror::Error;
use uuid::Uuid;

use crate::{
    config::AuthConfig,
    model::{
        refresh_token::CreateRefreshTokenDto,
        user::{
            dto::LoginRequestDto,
            error::UserStatusError,
            vo::{AuthResponseVo, LoggedUserInfoVo},
            LoginCredentialsEntity, UserEntity, UserStatus,
        },
    },
    repository::{
        token_repo::TokenRepository, user_repo::UserRepository, Create, Get, RepositoryError,
        RepositoryManager,
    },
    service::{Result, ServiceError},
    utils::{
        hmac,
        password::{PasswordError, PasswordUtils},
        token::{self, AccessClaims, TokenError},
    },
};

#[derive(Debug, Error)]
pub enum AuthError {
    HashingFailed,
    InvalidLoginCredentials,
    MissingToken,
    InvalidToken,
    ExpiredToken,
    RevokedToken,
    TokenCreationFailed,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl From<TokenError> for AuthError {
    fn from(error: TokenError) -> Self {
        match error {
            TokenError::InvalidToken => AuthError::InvalidToken,
            TokenError::ExpiredToken => AuthError::ExpiredToken,
            TokenError::TokenCreationFailed => AuthError::TokenCreationFailed,
        }
    }
}

impl From<PasswordError> for AuthError {
    fn from(error: PasswordError) -> Self {
        match error {
            PasswordError::PasswordHashingFailed => AuthError::HashingFailed,
        }
    }
}

impl From<hmac::InvalidLength> for AuthError {
    fn from(_: hmac::InvalidLength) -> Self {
        AuthError::HashingFailed
    }
}

pub struct AuthController;

impl AuthController {
    // Login user.
    pub async fn login(
        rm: &RepositoryManager,
        request: LoginRequestDto,
        auth_config: &AuthConfig,
    ) -> Result<AuthResponseVo> {
        let start = std::time::Instant::now();
        tracing::trace!("Login attempt received for email: {}", request.email);

        // 1. Verify login credentials
        let credentials = Self::verify_login(
            &rm.clone(),
            &request.email,
            &request.password,
            &auth_config.password_pepper,
        )
        .await
        .map_err(|e| {
            tracing::warn!(
                "Login verification failed for email={}: {:?}",
                request.email,
                e
            );
            e
        })?;

        let verification_time = start.elapsed();
        tracing::trace!(
            "User verification completed in {:?} for id={}",
            verification_time,
            credentials.id
        );

        // 2. Generate token pair
        let tokens = token::generate_tokens(credentials.id, credentials.role, auth_config)
            .map_err(AuthError::from)?;
        let refresh_token_hash = hmac::hash_sha512(
            tokens.refresh_token.as_bytes(),
            auth_config.refresh_token_pepper.expose_secret().as_bytes(),
        )
        .map_err(AuthError::from)?;

        tracing::trace!(
            "Tokens generated successfully for id={}",
            credentials.id
        );

        // 3. Get user info
        let user_info = Self::get_login_info(rm, credentials.id).await?;

        tracing::trace!(
            "User info retrieved successfully for id={}",
            credentials.id
        );

        // 4. Store refresh token
        let create_dto = CreateRefreshTokenDto {
            user_id: credentials.id,
            family_id: tokens.family_id,
            token_hash: &refresh_token_hash,
            expires_at: tokens.refresh_expires_at,
        };
        let _ = TokenRepository::create(rm, &create_dto).await?;

        // 5. Update last login time (fire & forget)
        {
            let rm_clone = rm.clone();
            tokio::spawn(async move {
                let _ = UserRepository::update_last_login(&rm_clone, credentials.id).await;
            });
        }

        let total_time = start.elapsed();
        tracing::trace!(
            "Login successful for email={}, id={}, total_time={:?}",
            &request.email,
            credentials.id,
            total_time
        );

        // 6. Return login VO
        Ok(AuthResponseVo {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            user_info,
        })
    }

    /// Refresh tokens.
    pub async fn refresh(
        rm: &RepositoryManager,
        input_token: Option<&str>,
        auth_config: &AuthConfig,
    ) -> Result<AuthResponseVo> {
        // 1. Hash incoming token
        let input_token = input_token.ok_or(AuthError::MissingToken)?;

        let input_token_hash = hmac::hash_sha512(
            input_token.as_bytes(),
            auth_config.refresh_token_pepper.expose_secret().as_bytes(),
        )
        .map_err(AuthError::from)?;

        // 2. Fetch token record
        // We need to know if it exists to check for reuse or expiration
        let token_entity = TokenRepository::get_by_hash(rm, &input_token_hash)
            .await?
            .ok_or(AuthError::InvalidToken)?;

        // 3. Reuse Detection (Security Critical)
        if token_entity.is_revoked {
            tracing::warn!(
                "Token reuse detected! Revoking family: {}",
                token_entity.family_id
            );
            TokenRepository::revoke_family(rm, &token_entity.family_id).await?;
            // Return generic error to avoid leaking implementation details
            return Err(AuthError::RevokedToken.into());
        }

        // 4. Check expiration
        // We check against the DB record, not the JWT claim (Stateful check)
        if token_entity.expires_at < chrono::Utc::now().naive_utc() {
            return Err(AuthError::ExpiredToken.into());
        }

        // 5. Get user info
        let user_info = Self::get_login_info(rm, token_entity.user_id).await?;

        // 6. Generate new refresh token pair
        // CRITICAL: We pass the EXISTING family_id to maintain the chain
        let new_tokens = token::generate_tokens_with_family_id(
            user_info.id,
            user_info.role,
            token_entity.family_id,
            auth_config,
        )
        .map_err(AuthError::from)?;

        let new_token_hash = hmac::hash_sha512(
            new_tokens.refresh_token.as_bytes(),
            auth_config.refresh_token_pepper.expose_secret().as_bytes(),
        )
        .map_err(AuthError::from)?;

        // 7. Rotate tokens in single DB transaction
        let create_dto = CreateRefreshTokenDto {
            user_id: token_entity.user_id,
            family_id: token_entity.family_id,
            token_hash: &new_token_hash,
            expires_at: new_tokens.refresh_expires_at,
        };

        TokenRepository::rotate(
            rm,
            &create_dto,
            token_entity.id, // Old token (to revoke)
        )
        .await?;

        // 8. Return new tokens
        Ok(AuthResponseVo {
            access_token: new_tokens.access_token,
            refresh_token: new_tokens.refresh_token,
            user_info,
        })
    }

    /// Logout user.
    pub async fn logout(rm: &RepositoryManager, family_id: &Uuid) -> Result<()> {
        tracing::trace!("Logout attempt received");

        TokenRepository::revoke_family(rm, family_id).await?;

        tracing::trace!("Logout successful.");
        Ok(())
    }

    /// Verify if the access token is valid.
    pub fn verify_access_token(
        token: Option<&str>,
        auth_config: &AuthConfig,
    ) -> Result<AccessClaims> {
        let token = token.ok_or(AuthError::MissingToken)?;

        Ok(token::validate_token::<AccessClaims>(
            token,
            auth_config.access_token_secret.expose_secret(),
        )
        .map_err(AuthError::from)?)
    }

    /// Verify login credentials.
    async fn verify_login(
        rm: &RepositoryManager,
        email: &str,
        password: &str,
        pepper: &SecretString,
    ) -> Result<LoginCredentialsEntity> {
        tracing::trace!("Starting login verification for user: {}", email);

        // 1. Get login credentials
        let user = UserRepository::get_login_credentials(rm, email)
            .await?
            .ok_or(AuthError::InvalidLoginCredentials)?;

        tracing::trace!(
            "User found for email={}, id={}, status={:?}",
            email,
            user.id,
            user.status
        );

        // 2. Check if user is active
        user.status.check_status()?;

        // 3. Verify password
        // Move to a spawn_blocking thread to release the executor threads from
        // the expensive password hashing operation.
        let is_valid = {
            let pwd = password.to_string();
            let pwd_hash = user.password_hash.clone();
            let pepper = pepper.clone();

            tokio::task::spawn_blocking(move || {
                PasswordUtils::verify_password(&pwd, &pwd_hash, pepper.expose_secret())
            })
            .await
            .map_err(|_e| {
                ServiceError::Internal(
                    "Password verification blocking thread failed to join".to_string(),
                )
            })?
        };

        if !is_valid {
            tracing::warn!(
                "Invalid login attempt: password verification failed for email={}, id={}",
                email,
                user.id
            );
            return Err(AuthError::InvalidLoginCredentials.into());
        }

        tracing::trace!(
            "Login verification successful for email={}, id={}",
            email,
            user.id
        );

        Ok(user)
    }

    async fn get_login_info(rm: &RepositoryManager, user_id: i64) -> Result<LoggedUserInfoVo> {
        tracing::trace!("Starting to fetch logged user info for id={}", user_id);

        // Get user basic info
        let user: UserEntity = UserRepository::get(rm, user_id).await?;

        tracing::trace!("User info retrieved successfully for id={}", user_id,);

        Ok(LoggedUserInfoVo {
            id: user.id,
            email: user.email,
            role: user.role,
        })
    }
}
