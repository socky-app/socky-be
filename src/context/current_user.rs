use crate::{
    model::user::{UserRole, UserStatus},
    utils::token::{AccessClaims, Claims},
};
use serde::{Deserialize, Serialize};

/// Current authenticated user info
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    pub id: i64,
    pub role: UserRole,
    pub status: UserStatus,
}

impl From<AccessClaims> for CurrentUser {
    fn from(value: AccessClaims) -> Self {
        Self {
            id: value.sub(),
            role: value.user_role(),
            status: value.user_status(),
        }
    }
}
