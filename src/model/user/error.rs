use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum UserStatusError {
    #[error("user is disabled")]
    Disabled,
    #[error("user is pending")]
    Pending,
    #[error("user is locked")]
    Locked,
}

#[derive(Debug, Clone, Error)]
#[error("required role ({required}) is greater than current role ({current})")]
pub struct UserRoleError {
    pub required: i16,
    pub current: i16,
}

impl AsRef<str> for UserRoleError {
    fn as_ref(&self) -> &str {
        "UserRoleError"
    }
}
