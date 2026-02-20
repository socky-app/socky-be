use thiserror::Error;

// TODO: rename to UserStatusError
#[derive(Debug, Clone, Error)]
pub enum UserStatusError {
    #[error("User is disabled")]
    Disabled,
    #[error("User is pending")]
    Pending,
    #[error("User is locked")]
    Locked,
}
#[derive(Debug, Clone, Error)]
#[error("Required role: {required}, current role: {current}")]
pub struct UserRoleError {
    pub required: i16,
    pub current: i16,
}
