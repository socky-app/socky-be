//! Authentication and session management controller.
//!
//! Handles business logic for logging in, refreshing sessions via token rotation,
//! logging out, and updating user passwords. Includes defensive hashing to prevent timing attacks.

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

const DUMMY_PASSWORD_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$UFxiL8tpj0KYEN3aDEKaFg$nzKJ6p6BignX4wQJqfOjRtzga6iue7uJSGn//wHlW3g";

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

/// Controller orchestrating all authentication workflows.
pub struct AuthController;

impl AuthController {
    /// Authenticates a user using their email and password.
    ///
    /// Performs constant-time password verification using a dummy hash if the email is not found,
    /// preventing username enumeration via timing analysis. Generates a fresh JWT access token
    /// and an opaque refresh token stored in the database.
    #[tracing::instrument(name = "auth_login", skip_all, fields(email = %request.email))]
    pub async fn login(
        rm: &RepositoryManager,
        request: LoginRequestDto,
        auth_config: &AuthConfig,
    ) -> Result<AuthResponseVo> {
        // 1. Fetch user credentials by email
        let credentials_opt =
            AuthRepository::get_user_credentials_by_email(rm, &request.email).await?;

        // 2. Determine which hash to verify to normalize CPU time
        let hash_to_verify = credentials_opt
            .as_ref()
            .map(|c| c.password_hash.clone())
            .unwrap_or_else(|| DUMMY_PASSWORD_HASH.to_string());

        // 3. Always execute the blocking password verification
        let is_password_valid = {
            let pwd = request.password.clone();
            let pepper = auth_config.password_pepper.clone();

            tokio::task::spawn_blocking(move || {
                PasswordUtils::verify_password(&pwd, &hash_to_verify, pepper.expose_secret())
            })
            .await
            .map_err(|_e| {
                ControllerError::Internal(
                    "blocking thread for password verification failed to join".to_string(),
                )
            })?
        };

        // 4. Verify credentials
        let credentials = match credentials_opt {
            Some(c) => c,
            None => {
                // The email didn't exist, but we still performed the hashing to normalize the request duration.
                return Err(AuthError::InvalidEmail {
                    email: request.email,
                }
                .into());
            }
        };

        if !is_password_valid {
            return Err(AuthError::InvalidPassword {
                user_id: credentials.id,
            }
            .into());
        }

        credentials.status.check_status()?;

        debug!(
            user_id = %credentials.id,
            "Credentials verification successful",
        );

        // 5. Generate token pair
        let tokens = token::generate_tokens(credentials.id, credentials.role, auth_config)
            .map_err(AuthError::from)?;
        let refresh_token_hash = hmac::hash_sha512(
            tokens.refresh_token.as_bytes(),
            auth_config.refresh_token_pepper.expose_secret().as_bytes(),
        )
        .map_err(AuthError::from)?;

        // 6. Get user info
        let user_info = Self::get_login_info(rm, credentials.id).await?;

        // 7. Store refresh token
        let create_dto = CreateRefreshTokenDto {
            token_hash: &refresh_token_hash,
            user_id: tokens.user_id,
            family_id: tokens.family_id,
            access_id: tokens.access_token_id,
            expires_at: tokens.refresh_expires_at,
        };
        let _ = AuthRepository::create_token(rm, &create_dto).await?;

        // 8. Update last login time (fire & forget)
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

        // 9. Return login VO
        Ok(AuthResponseVo {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            user_info,
        })
    }

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
    ///
    /// We don't enforce the expensive password hash here like we do in login. This is a protected
    /// endpoint, so the attack surface is N=1 and therefore there is no real benefit of doing so.
    #[tracing::instrument(name = "auth_update_password", skip_all)]
    pub async fn update_password(
        rm: &RepositoryManager,
        user_id: i64,
        request: UpdateUserPasswordDto,
        auth_config: &AuthConfig,
    ) -> Result<()> {
        // 1. Fetch current user credentials
        let credentials = AuthRepository::get_user_credentials(rm, user_id).await?;

        // 2. Check if user is active
        credentials.status.check_status()?;

        // 3. Verify old password (blocking thread)
        let is_valid = {
            let pwd = request.old_password.clone();
            let pwd_hash = credentials.password_hash.clone();
            let pepper = auth_config.password_pepper.clone();

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
            return Err(AuthError::InvalidPassword {
                user_id: credentials.id,
            }
            .into());
        }

        debug!("Credentials verification successful");

        // 4. Hash the new password (blocking thread)
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

        // 5. Execute atomic database update
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
}
