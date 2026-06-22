use crate::{
    controller::{auth_controller::AuthError, ControllerError, Result},
    model::{
        pagination::PaginatedResponse,
        user::{
            dto::{UpdateProfileDto, UpdateUserStatusDto, UpdateUsernameDto, UserQueryDto},
            vo::{RegisteredUserVo, UserVo},
            UserRole, UserStatus,
        },
    },
    repository::{auth_repo::AuthRepository, user_repo::UserRepository, RepositoryManager},
};

pub struct UserController;

impl UserController {
    /// Returns a single user by ID.
    pub async fn get_user(rm: &RepositoryManager, user_id: i64) -> Result<UserVo> {
        let entity = UserRepository::get(rm, user_id).await?;
        Ok(UserVo::from(entity))
    }

    /// Returns a registered user by ID.
    pub async fn get_registered_user(
        rm: &RepositoryManager,
        user_id: i64,
    ) -> Result<RegisteredUserVo> {
        match Self::get_user(rm, user_id).await? {
            UserVo::Registered(r) => Ok(r),
            UserVo::Ghost(_) => Err(ControllerError::Internal(format!(
                "User {} is a ghost user, not a registered user",
                user_id
            ))),
        }
    }

    /// Lists all users with optional filters and pagination.
    pub async fn list_users(
        rm: &RepositoryManager,
        query: &UserQueryDto,
    ) -> Result<PaginatedResponse<UserVo>> {
        let response = UserRepository::list_all(rm, query).await?;
        Ok(response.map(UserVo::from))
    }

    /// Updates the user's public profile fields.
    pub async fn update_profile(
        rm: &RepositoryManager,
        user_id: i64,
        dto: UpdateProfileDto,
    ) -> Result<()> {
        UserRepository::update_profile(rm, user_id, &dto).await?;
        Ok(())
    }

    /// Updates the user's username handle.
    pub async fn update_username(
        rm: &RepositoryManager,
        user_id: i64,
        dto: UpdateUsernameDto,
    ) -> Result<()> {
        // Validate uniqueness
        if UserRepository::username_exists(rm, &dto.username).await? {
            return Err(ControllerError::UsernameAlreadyExists {
                username: dto.username,
            });
        }

        UserRepository::update_username(rm, user_id, &dto.username).await?;
        Ok(())
    }

    /// Updates the user's status.
    pub async fn update_status(
        rm: &RepositoryManager,
        user_id: i64,
        dto: UpdateUserStatusDto,
    ) -> Result<()> {
        UserRepository::update_status(rm, user_id, &dto).await?;
        Ok(())
    }

    /// Soft-deletes the user account.
    pub async fn delete_user(rm: &RepositoryManager, user_id: i64) -> Result<()> {
        UserRepository::soft_delete(rm, user_id).await?;
        Ok(())
    }

    /// Admin deletes a user account (anonymization and revokes all tokens).
    pub async fn admin_delete_user(rm: &RepositoryManager, user_id: i64) -> Result<()> {
        AuthRepository::anonymize_user(rm, user_id).await?;
        Ok(())
    }
}
