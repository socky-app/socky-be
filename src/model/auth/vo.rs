use serde::Serialize;

use crate::model::user::vo::RegisteredUserVo;

/// Response payload for autheticated user.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct AuthResponseVo {
    /// Token for authenticating subsequent requests
    pub access_token: String,
    /// Token for refreshing the user credentials
    pub refresh_token: String,
    /// User information
    pub user_info: RegisteredUserVo,
}
