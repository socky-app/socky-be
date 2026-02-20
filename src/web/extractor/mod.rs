//! Extractor types to handle role access.
//! 
//! Use axum::middleware::from_extractor::<T>() to create middlewares when multiple routes
//! have the same permissions, or use the extractor directly on the route if more
//! granular access is required.

use crate::model::user::UserRole;

mod require_role;

pub type RequireStandard = require_role::RequireRole<{ UserRole::Standard as i16 }>;
pub type RequireSupport = require_role::RequireRole<{ UserRole::Support as i16 }>;
pub type RequireAdmin = require_role::RequireRole<{ UserRole::Admin as i16 }>;
