use secrecy::{ExposeSecret, SecretString};

use thiserror::Error;
use tracing::debug;
use uuid::Uuid;

use crate::{
    config::AuthConfig,
    controller::{ControllerError, Result},
    model::{
        auth::{
            dto::{CreateRefreshTokenDto, LoginRequestDto, UpdateUserPasswordDto},
            vo::{AuthResponseVo, LoggedUserInfoVo},
            UserCredentialsEntity,
        },
        user::UserEntity,
    },
    repository::{
        auth_repo::AuthRepository, user_repo::UserRepository, Create, Get, RepositoryError,
        RepositoryManager,
    },
    utils::{
        hmac,
        password::{PasswordError, PasswordUtils},
        token::{self, AccessClaims, TokenError},
    },
};

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("hashing failed for {0}")]
    HashingFailed(String),

    #[error("invalid email {email}")]
    InvalidEmail { email: String },

    #[error("invalid password for user {user_id}")]
    InvalidPassword { user_id: i64 },

    #[error("token not found")]
    NotFoundToken,

    #[error("invalid token")]
    InvalidToken,

    #[error("expired token")]
    ExpiredToken,

    #[error("revoked token: {token_id} family {family_id} user {user_id} ")]
    RevokedToken {
        token_id: i64,
        user_id: i64,
        family_id: Uuid,
    },

    #[error("token creation failed")]
    TokenCreationFailed,

    #[error("concurrent refreshes using same token")]
    ConcurrentRefresh,
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
            PasswordError::PasswordHashingFailed => {
                AuthError::HashingFailed("password".to_string())
            }
        }
    }
}

impl From<hmac::InvalidLength> for AuthError {
    fn from(_: hmac::InvalidLength) -> Self {
        AuthError::HashingFailed("hmac".to_string())
    }
}

pub struct AuthController;

impl AuthController {
    // Login user.
    #[tracing::instrument(name = "auth_login", skip_all, fields(email = %request.email))]
    pub async fn login(
        rm: &RepositoryManager,
        request: LoginRequestDto,
        auth_config: &AuthConfig,
    ) -> Result<AuthResponseVo> {
        // 1. Verify login credentials
        let credentials = AuthRepository::get_user_credentials_by_email(rm, &request.email)
            .await?
            .ok_or(AuthError::InvalidEmail {
                email: request.email,
            })?;

        Self::verify_credentials(
            &credentials,
            &request.password,
            &auth_config.password_pepper,
        )
        .await?;

        // 2. Generate token pair
        let tokens = token::generate_tokens(credentials.id, credentials.role, auth_config)
            .map_err(AuthError::from)?;
        let refresh_token_hash = hmac::hash_sha512(
            tokens.refresh_token.as_bytes(),
            auth_config.refresh_token_pepper.expose_secret().as_bytes(),
        )
        .map_err(AuthError::from)?;

        // 3. Get user info
        let user_info = Self::get_login_info(rm, credentials.id).await?;

        // 4. Store refresh token
        let create_dto = CreateRefreshTokenDto {
            token_hash: &refresh_token_hash,
            user_id: tokens.user_id,
            family_id: tokens.family_id,
            access_id: tokens.access_token_id,
            expires_at: tokens.refresh_expires_at,
        };
        let _ = AuthRepository::create_token(rm, &create_dto).await?;

        // 5. Update last login time (fire & forget)
        {
            let rm_clone = rm.clone();
            tokio::spawn(async move {
                let _ = AuthRepository::update_last_login(&rm_clone, credentials.id).await;
            });
        }

        debug!(
            user_id = %user_info.id,
            user_role = %user_info.role as i16,
            "Login successful",
        );

        // 6. Return login VO
        Ok(AuthResponseVo {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            user_info,
        })
    }

    // TODO: Create background task to periodically clean up dead tokens

    /// Refresh tokens.
    #[tracing::instrument(name = "auth_refresh", skip_all)]
    pub async fn refresh(
        rm: &RepositoryManager,
        input_token: &str,
        auth_config: &AuthConfig,
    ) -> Result<AuthResponseVo> {
        // 1. Hash incoming token
        let input_token_hash = hmac::hash_sha512(
            input_token.as_bytes(),
            auth_config.refresh_token_pepper.expose_secret().as_bytes(),
        )
        .map_err(AuthError::from)?;

        // 2. Fetch token record and acquire lock
        // We need to know if it exists to check for reuse or expiration
        let token_lock = AuthRepository::acquire_rotation_lock(rm, &input_token_hash)
            .await?
            .ok_or(AuthError::NotFoundToken)?;
        let token_entity = &token_lock.entity;

        // 3. User status check (CRITICAL)
        if let Err(e) = token_entity.user_status.check_status() {
            // If the user was banned since their last refresh, kill the token.
            token_lock.revoke_family().await?;
            return Err(e.into());
        }

        // 4. Reuse detection (CRITICAL)
        if token_entity.is_revoked {
            let err = AuthError::RevokedToken {
                token_id: token_entity.id,
                user_id: token_entity.user_id,
                family_id: token_entity.family_id,
            };

            token_lock.revoke_family().await?;

            return Err(err.into());
        }

        // 5. Expiration check (CRITICAL)
        if token_entity.expires_at < chrono::Utc::now().naive_utc() {
            return Err(AuthError::ExpiredToken.into());
        }

        // 6. Generate new refresh token pair
        // We pass the existing family_id to maintain the chain
        let new_tokens = token::generate_tokens_with_family_id(
            token_entity.user_id,
            token_entity.user_role,
            token_entity.family_id,
            auth_config,
        )
        .map_err(AuthError::from)?;

        let new_token_hash = hmac::hash_sha512(
            new_tokens.refresh_token.as_bytes(),
            auth_config.refresh_token_pepper.expose_secret().as_bytes(),
        )
        .map_err(AuthError::from)?;

        // 7. Create user info
        let user_info = LoggedUserInfoVo {
            id: token_entity.user_id,
            email: token_entity.user_email.clone(),
            role: token_entity.user_role,
        };

        // 8. Rotate tokens
        let create_dto = CreateRefreshTokenDto {
            token_hash: &new_token_hash,
            user_id: new_tokens.user_id,
            family_id: new_tokens.family_id,
            access_id: new_tokens.access_token_id,
            expires_at: new_tokens.refresh_expires_at,
        };

        let result = token_lock.rotate(&create_dto).await?;

        debug!(
            user_id = %user_info.id,
            user_role = user_info.role as i16,
            "Refresh successful",
        );

        Ok(AuthResponseVo {
            access_token: new_tokens.access_token,
            refresh_token: new_tokens.refresh_token,
            user_info,
        })
    }

    /// Logout user.
    #[tracing::instrument(name = "auth_logout", skip_all)]
    pub async fn logout(
        rm: &RepositoryManager,
        refresh_token: &str,
        auth_config: &AuthConfig,
    ) -> Result<()> {
        // 1. Hash incoming token
        let token_hash = hmac::hash_sha512(
            refresh_token.as_bytes(),
            auth_config.refresh_token_pepper.expose_secret().as_bytes(),
        )
        .map_err(AuthError::from)?;

        // 2. Perform the get and revoke in a single atomic database query
        if let Some(info) = AuthRepository::revoke_token_family_by_hash(rm, &token_hash).await? {
            if info.was_already_revoked {
                // This is a red flag in a token rotation setup
                tracing::warn!(
                    user_id = %info.user_id,
                    family_id = %info.family_id,
                    "Logout attempted with an ALREADY REVOKED token. Possible replay attack or client retry."
                );
            } else {
                // Normal, happy-path logout
                tracing::debug!(
                    user_id = %info.user_id,
                    family_id = %info.family_id,
                    "Token family successfully revoked"
                );
            }
        } else {
            // The token literally does not exist in the database
            tracing::debug!("Refresh token not found");
        }

        Ok(())
    }

    /// Logout user from all devices.
    #[tracing::instrument(name = "auth_logout_all", skip_all)]
    pub async fn logout_all(rm: &RepositoryManager, user_id: i64) -> Result<()> {
        // Revoke all tokens associated with the user in a single DB query
        AuthRepository::revoke_tokens_for_user(rm, user_id).await?;

        tracing::info!("Successfully revoked all tokens for user");

        Ok(())
    }

    /// Change user password.
    #[tracing::instrument(name = "auth_update_password", skip_all)]
    pub async fn update_password(
        rm: &RepositoryManager,
        user_id: i64,
        request: UpdateUserPasswordDto,
        auth_config: &AuthConfig,
    ) -> Result<()> {
        // 1. Fetch current user credentials
        let credentials = AuthRepository::get_user_credentials(rm, user_id).await?;

        // 2. Verify old password (blocking thread)
        Self::verify_credentials(
            &credentials,
            &request.old_password,
            &auth_config.password_pepper,
        )
        .await?;

        // 3. Hash the new password (blocking thread)
        let new_hash = {
            let new_pwd = request.new_password.clone();
            let pepper = auth_config.password_pepper.clone();

            tokio::task::spawn_blocking(move || {
                PasswordUtils::hash_password(&new_pwd, pepper.expose_secret())
            })
            .await
            .map_err(|_| ControllerError::Internal("blocking thread failed".into()))?
            .map_err(AuthError::from)?
        };

        // 4. Execute atomic database update
        AuthRepository::change_password_and_revoke_tokens(rm, user_id, &new_hash).await?;

        tracing::info!("Password changed successfully and all previous sessions revoked");

        Ok(())
    }

    /// Verify if the access token is valid.
    #[tracing::instrument(name = "auth_verify_token", skip_all)]
    pub fn verify_access_token(token: &str, auth_config: &AuthConfig) -> Result<AccessClaims> {
        let claims = token::validate_jwt::<AccessClaims>(
            token,
            auth_config.access_token_secret.expose_secret(),
        )
        .map_err(AuthError::from)?;

        Ok(claims)
    }

    #[tracing::instrument(name = "auth_login_info", skip_all, fields(user_id = %user_id))]
    pub async fn get_login_info(rm: &RepositoryManager, user_id: i64) -> Result<LoggedUserInfoVo> {
        let user: UserEntity = UserRepository::get(rm, user_id).await?;

        debug!("Logged user info retrieved successfully");

        Ok(LoggedUserInfoVo {
            id: user.id,
            email: user.email,
            role: user.role,
        })
    }

    /// Verify user credentials.
    async fn verify_credentials(
        user: &UserCredentialsEntity,
        password: &str,
        pepper: &SecretString,
    ) -> Result<()> {
        // 1. Check if user is active
        user.status.check_status()?;

        // 2. Verify password
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
                ControllerError::Internal(
                    "blocking thread for password verification failed to join".to_string(),
                )
            })?
        };

        if !is_valid {
            return Err(AuthError::InvalidPassword { user_id: user.id }.into());
        }

        debug!(
            user_id = %user.id,
            "Credentials verification successful",
        );

        Ok(())
    }
}
