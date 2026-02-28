use secrecy::{ExposeSecret, SecretString};

use thiserror::Error;
use tracing::debug;
use uuid::Uuid;

use crate::{
    config::AuthConfig,
    controller::{ControllerError, Result},
    model::{
        auth::{
            dto::{CreateRefreshTokenDto, LoginRequestDto},
            vo::{AuthResponseVo, LoggedUserInfoVo},
            LoginCredentialsEntity,
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

    #[error("invalid password for user {user_id} {email}")]
    InvalidPassword { user_id: i64, email: String },

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
        let credentials = Self::verify_login(
            &rm.clone(),
            &request.email,
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

        // 2. Fetch token record
        // We need to know if it exists to check for reuse or expiration
        let token_entity = AuthRepository::get_token_by_hash(rm, &input_token_hash)
            .await?
            .ok_or(AuthError::NotFoundToken)?;

        // 3. Reuse Detection (Security Critical)
        if token_entity.is_revoked {
            AuthRepository::revoke_token_family(rm, &token_entity.family_id).await?;

            return Err(AuthError::RevokedToken {
                token_id: token_entity.id,
                user_id: token_entity.user_id,
                family_id: token_entity.family_id,
            }
            .into());
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
            token_hash: &new_token_hash,
            user_id: new_tokens.user_id,
            family_id: new_tokens.family_id,
            access_id: new_tokens.access_token_id,
            expires_at: new_tokens.refresh_expires_at,
        };

        let result = AuthRepository::rotate_token(rm, &create_dto, token_entity.id).await;

        // 8. Handle happy path and rotation errors
        match result {
            Ok(_) => {
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
            Err(e) => {
                // Check if the error is our specific NotFound error from the race condition
                if let RepositoryError::NotFound { .. } = e {
                    tracing::warn!(
                        user_id = %token_entity.user_id,
                        family_id = %token_entity.family_id,
                        "Concurrent refresh detected. Rotation blocked."
                    );

                    // Return a custom auth error that translates to a 401 Unauthorized
                    // so the client knows they need to log in again or use the token
                    // from the other concurrent request.
                    Err(AuthError::ConcurrentRefresh.into())
                } else {
                    // If it's a different database error (e.g., connection dropped), bubble it up
                    Err(e.into())
                }
            }
        }
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

    /// Verify login credentials.
    async fn verify_login(
        rm: &RepositoryManager,
        email: &str,
        password: &str,
        pepper: &SecretString,
    ) -> Result<LoginCredentialsEntity> {
        // 1. Get login credentials
        let user = AuthRepository::get_login_credentials(rm, email)
            .await?
            .ok_or(AuthError::InvalidEmail {
                email: email.to_string(),
            })?;

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
                ControllerError::Internal(
                    "blocking thread for password verification failed to join".to_string(),
                )
            })?
        };

        if !is_valid {
            return Err(AuthError::InvalidPassword {
                user_id: user.id,
                email: email.to_string(),
            }
            .into());
        }

        debug!(
            user_id = %user.id,
            "Login verification successful",
        );

        Ok(user)
    }
}
