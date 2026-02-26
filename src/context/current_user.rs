use crate::{
    model::user::UserRole,
    utils::token::{AccessClaims, Claims},
};
use serde::{Deserialize, Serialize};

/// Current authenticated user info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUser {
    pub id: i64,
    pub role: UserRole,
}

impl CurrentUser {
    pub fn new(id: i64, role: UserRole) -> Self {
        Self {
            id,
            role,
        }
    }
}

impl From<AccessClaims> for CurrentUser {
    fn from(value: AccessClaims) -> Self {
        Self {
            id: value.user_id(),
            role: value.user_role(),
        }
    }
}
