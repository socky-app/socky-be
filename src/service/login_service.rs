use thiserror::Error;

use crate::{
    common::password::PasswordUtils,
    model::user::{
        dto::LoginRequestDto,
        error::UserError,
        vo::{LoggedUserInfoVo, LoginVo},
        LoginCredentialsEntity, UserEntity, UserStatus,
    },
    repository::{ops::get::Get, user_repo::UserRepository, RepositoryError, RepositoryManager},
};

#[derive(Debug, Error)]
pub enum LoginServiceError {
    Repository(#[from] RepositoryError),
    NotFoundCredentials,
    InvalidCredentials,
    InvalidUser(#[from] UserError),
}

type Result<T> = core::result::Result<T, LoginServiceError>;

impl core::fmt::Display for LoginServiceError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

pub struct LoginService;

impl LoginService {
    // Login user.
    pub async fn login(rm: RepositoryManager, request: LoginRequestDto) -> Result<LoginVo> {
        let start = std::time::Instant::now();
        tracing::info!("Login attempt received for username: {}", request.username);

        // 1. Verify login credentials
        let credentials = Self::verify_login(rm.clone(), &request.username, &request.password)
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

        // TODO 2. Generate token
        // let token = jwt::generate_token(user.id, &request.username).map_err(|e| {
        //     tracing::error!("Failed to generate token for user_id={}: {:?}", user.id, e);
        //     ServiceError::TokenCreationFailed
        // })?;
        let token = "my_jwt".to_string();

        tracing::debug!(
            "JWT token generated successfully for user_id={}",
            credentials.id
        );

        // 3. Update last login time
        {
            let rm_clone = rm.clone();
            let user_id_clone = credentials.id;
            tokio::spawn(async move {
                let _ = UserRepository::update_last_login(rm_clone, user_id_clone).await;
            });
        }

        // 4. Get user info
        let user_info = Self::get_login_info(rm, credentials.id).await?;

        let total_time = start.elapsed();
        tracing::info!(
            "Login successful for username={}, user_id={}, total_time={:?}",
            &request.username,
            credentials.id,
            total_time
        );

        // 5. Return login VO
        Ok(LoginVo { token, user_info })
    }

    /// Verify login credentials.
    async fn verify_login(
        rm: RepositoryManager,
        username: &str,
        password: &str,
    ) -> Result<LoginCredentialsEntity> {
        tracing::info!("Starting login verification for username: {}", username);

        // 1. Get login credentials
        let user = UserRepository::get_login_credentials(rm, username)
            .await?
            .ok_or(LoginServiceError::NotFoundCredentials)?;

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
        if !PasswordUtils::verify_password(password, &user.password_hash) {
            tracing::warn!(
                "Invalid login attempt: password verification failed for username={}, user_id={}",
                username,
                user.id
            );
            return Err(LoginServiceError::InvalidCredentials);
        }

        tracing::info!(
            "Login verification successful for username={}, user_id={}",
            username,
            user.id
        );

        Ok(user)
    }

    async fn get_login_info(rm: RepositoryManager, user_id: i64) -> Result<LoggedUserInfoVo> {
        tracing::info!(user_id, "Starting to fetch comprehensive user info");

        // Get user basic info
        let user: UserEntity = UserRepository::get(&rm, user_id).await?;

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
