use thiserror::Error;

// TODO: rename to UserStatusError
#[derive(Debug, Clone, Error)]
pub enum UserError {
    #[error("User is disabled")]
    UserIsDisabled,
    #[error("User is pending")]
    UserIsPending,
    #[error("User is locked")]
    UserIsLocked,
}
#[derive(Debug, Clone, Error)]
#[error("Required role: {required}, current role: {current}")]
pub struct UserRoleError {
    pub required: i16,
    pub current: i16,
}
