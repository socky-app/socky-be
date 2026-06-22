use crate::{
    controller::Result,
    model::user::vo::PublicProfileVo,
    repository::{user_repo::UserRepository, RepositoryManager},
};

pub struct ProfileController;

impl ProfileController {
    /// Searches for public profiles.
    pub async fn search(rm: &RepositoryManager, query: &str) -> Result<Vec<PublicProfileVo>> {
        let entities = UserRepository::search_active(rm, query).await?;
        let vos = entities.into_iter().map(PublicProfileVo::from).collect();
        Ok(vos)
    }

    /// Checks if a username is available.
    pub async fn check_username(rm: &RepositoryManager, username: &str) -> Result<bool> {
        let exists = UserRepository::username_exists(rm, username).await?;
        Ok(!exists)
    }

    /// Gets a public profile by ID.
    pub async fn get_profile(rm: &RepositoryManager, id: i64) -> Result<PublicProfileVo> {
        let entity = UserRepository::get_active(rm, id).await?;
        Ok(PublicProfileVo::from(entity))
    }
}
