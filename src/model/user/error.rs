use strum_macros::AsRefStr;
use thiserror::Error;

use crate::common::ErrorType;

#[derive(Debug, Clone, Error, AsRefStr)]
pub enum UserStatusError {
    #[error("User is disabled")]
    Disabled,
    #[error("User is pending")]
    Pending,
    #[error("User is locked")]
    Locked,
}

impl ErrorType for UserStatusError {}

#[derive(Debug, Clone, Error)]
#[error("Required role: {required}, current role: {current}")]
pub struct UserRoleError {
    pub required: i16,
    pub current: i16,
}

impl AsRef<str> for UserRoleError {
    fn as_ref(&self) -> &str {
        "UserRoleError"
    }
}

impl ErrorType for UserRoleError {}
