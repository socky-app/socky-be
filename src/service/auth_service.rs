use secrecy::{ExposeSecret, SecretString};
use thiserror::Error;

use crate::{
    common::config::AuthConfig,
    utils::{
        hmac,
        password::PasswordUtils,
        token::{self, TokenError},
    },
    model::{refresh_token::CreateRefreshTokenDto, user::{
        LoginCredentialsEntity, UserEntity, UserStatus, dto::LoginRequestDto, error::UserError, vo::{AuthResponseVo, LoggedUserInfoVo}
    }},
    repository::{RepositoryError, RepositoryManager, ops::{create::Create, get::Get}, token_repo::TokenRepository, user_repo::UserRepository},
};

#[derive(Debug, Error)]
pub enum AuthServiceError {
    Repository(#[from] RepositoryError),
    UserNotFound,
    WrongPassword,
    InvalidUser(#[from] UserError),
    InvalidRefreshToken,
    ExpiredRefreshToken,
    RevokedRefreshToken,
    InternalError,
}

type Result<T> = core::result::Result<T, AuthServiceError>;

impl core::fmt::Display for AuthServiceError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl From<TokenError> for AuthServiceError {
    fn from(error: TokenError) -> Self {
        match error {
            // Direct mappings
            TokenError::InvalidToken => AuthServiceError::InvalidRefreshToken,
            TokenError::ExpiredToken => AuthServiceError::ExpiredRefreshToken,

            // Map technical/unexpected errors to a generic InternalError
            TokenError::TokenCreationFailed => AuthServiceError::InternalError,
        }
    }
}

impl From<hmac::InvalidLength> for AuthServiceError {
    fn from(_: hmac::InvalidLength) -> Self {
        AuthServiceError::InternalError
    }
}

pub struct AuthService;

impl AuthService {
    // Login user.
    pub async fn login(
        rm: &RepositoryManager,
        request: LoginRequestDto,
        auth_config: &AuthConfig,
    ) -> Result<AuthResponseVo> {
        let start = std::time::Instant::now();
        tracing::info!("Login attempt received for username: {}", request.username);

        // 1. Verify login credentials
        let credentials = Self::verify_login(
            &rm.clone(),
            &request.username,
            &request.password,
            &auth_config.password_pepper,
        )
        .await
        .map_err(|e| {
            tracing::warn!(
                "Login verification failed for username={}: {:?}",
                request.username,
                e
            );
            e
        })?;

        let verification_time = start.elapsed();
        tracing::debug!(
            "User verification completed in {:?} for user_id={}",
            verification_time,
            credentials.id
        );

        // 2. Generate token pair
        let tokens = token::generate_tokens(credentials.id, auth_config)?;
        let refresh_token_hash = hmac::hash_sha512(
            tokens.refresh_token.as_bytes(),
            auth_config.refresh_token_pepper.expose_secret().as_bytes(),
        )?;

        tracing::debug!(
            "Tokens generated successfully for user_id={}",
            credentials.id
        );

        // 3. Get user info
        let user_info = Self::get_login_info(rm, credentials.id).await?;

        tracing::debug!(
            "User info retrieved successfully for user_id={}",
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
            let user_id_clone = credentials.id;
            tokio::spawn(async move {
                let _ = UserRepository::update_last_login(&rm_clone, user_id_clone).await;
            });
        }

        let total_time = start.elapsed();
        tracing::info!(
            "Login successful for username={}, user_id={}, total_time={:?}",
            &request.username,
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
        input_token: &str,
        auth_config: &AuthConfig,
    ) -> Result<AuthResponseVo> {
        // 1. Hash incoming token
        let input_token_hash = hmac::hash_sha512(
            input_token.as_bytes(),
            auth_config.refresh_token_pepper.expose_secret().as_bytes(),
        )?;

        // 2. Fetch token record
        // We need to know if it exists to check for reuse or expiration
        let token_entity = TokenRepository::get_by_hash(rm, &input_token_hash).await?
            .ok_or(AuthServiceError::InvalidRefreshToken)?;

        // 3. Reuse Detection (Security Critical)
        if token_entity.is_revoked {
            tracing::error!("Token reuse detected! Revoking family: {}", token_entity.family_id);
            TokenRepository::revoke_family(rm, &token_entity.family_id).await?;
            // Return generic error to avoid leaking implementation details
            return Err(AuthServiceError::RevokedRefreshToken); 
        }

        // 4. Check expiration
        // We check against the DB record, not the JWT claim (Stateful check)
        if token_entity.expires_at < chrono::Utc::now().naive_utc() {
            return Err(AuthServiceError::ExpiredRefreshToken);
        }

        // 5. Generate new refresh token pair
        // CRITICAL: We pass the EXISTING family_id to maintain the chain
        let new_tokens = token::generate_tokens_with_family_id(
            token_entity.user_id,
            token_entity.family_id, 
            auth_config
        )?;

        let new_token_hash = hmac::hash_sha512(
            new_tokens.refresh_token.as_bytes(),
            auth_config.refresh_token_pepper.expose_secret().as_bytes(),
        )?;

        // 6. Get user info
        let user_info = Self::get_login_info(rm, token_entity.user_id).await?;

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
        ).await?;

        // 8. Return new tokens
        Ok(AuthResponseVo {
            access_token: new_tokens.access_token,
            refresh_token: new_tokens.refresh_token,
            user_info,
        })
    }

    /// Logout user.
    pub async fn logout(
        rm: &RepositoryManager,
        refresh_token: &str,
        auth_config: &AuthConfig,
    ) -> Result<()> {
        tracing::info!("Logout attempt received");

        // 1. Hash incoming token
        let token_hash = hmac::hash_sha512(
            refresh_token.as_bytes(),
            auth_config.refresh_token_pepper.expose_secret().as_bytes(),
        )?;

        // 2. Find the token to get its Family ID
        // If it's already gone/invalid, we can just return Ok (idempotent)
        if let Some(token_entity) = TokenRepository::get_by_hash(rm, &token_hash).await? {
           
            // 3. Revoke the entire Family
            tracing::info!("Revoking session family: {}", token_entity.family_id);
            TokenRepository::revoke_family(rm, &token_entity.family_id).await?;
        }

        tracing::info!("Logout successful.");
        Ok(())
    }

    /// Verify login credentials.
    async fn verify_login(
        rm: &RepositoryManager,
        username: &str,
        password: &str,
        pepper: &SecretString,
    ) -> Result<LoginCredentialsEntity> {
        tracing::info!("Starting login verification for username: {}", username);

        // 1. Get login credentials
        let user = UserRepository::get_login_credentials(rm, username)
            .await?
            .ok_or(AuthServiceError::UserNotFound)?;

        tracing::debug!(
            "User found for username={}, user_id={}, status={}",
            username,
            user.id,
            user.status
        );

        // 2. Check if user is enabled
        let status = UserStatus::try_from(user.status)?;
        status.check_status()?;

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
            .map_err(|_e| AuthServiceError::InternalError)?
        };

        if !is_valid {
            tracing::warn!(
                "Invalid login attempt: password verification failed for username={}, user_id={}",
                username,
                user.id
            );
            return Err(AuthServiceError::WrongPassword);
        }

        tracing::info!(
            "Login verification successful for username={}, user_id={}",
            username,
            user.id
        );

        Ok(user)
    }

    async fn get_login_info(rm: &RepositoryManager, user_id: i64) -> Result<LoggedUserInfoVo> {
        tracing::info!(user_id, "Starting to fetch comprehensive user info");

        // Get user basic info
        let user: UserEntity = UserRepository::get(rm, user_id).await?;

        tracing::debug!(
            "User basic info retrieved for user_id={}, username={}",
            user_id,
            user.username
        );

        // TODO: Get user permissions, may update permissions cache, and retrieve
        // any other attributes relevant to the logged user.

        tracing::info!(
            "User info retrieved successfully for user_id={}, username={}",
            user_id,
            user.username
        );

        Ok(LoggedUserInfoVo {
            id: user.id,
            username: user.username,
            email: user.email,
        })
    }
}
