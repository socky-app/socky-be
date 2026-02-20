use crate::model::user::UserRole;

mod require_role;

pub type RequireStandard = require_role::RequireRole<{ UserRole::Standard as i16 }>;
pub type RequireSupport = require_role::RequireRole<{ UserRole::Support as i16 }>;
pub type RequireAdmin = require_role::RequireRole<{ UserRole::Admin as i16 }>;
