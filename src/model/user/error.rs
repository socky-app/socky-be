use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum UserError {
    #[error("Invalid user status")]
    InvalidUserStatus,
    #[error("User is disabled")]
    UserIsDisabled,
    #[error("User is pending")]
    UserIsPending,
    #[error("User is locked")]
    UserIsLocked,
}
