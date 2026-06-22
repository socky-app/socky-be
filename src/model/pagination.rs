//! Pagination models and utilities.
//!
//! Pagination in this API is strictly **1-indexed**.
//! The first page is always `page = 1`.
//! If a client requests `page <= 0`, the controller layers will automatically coerce it to `1`.
use serde::{Deserialize, Serialize};

/// Returns the default starting page for paginated queries (1-indexed).
pub fn default_page() -> i64 {
    1
}

/// Returns the default limit (page size) for paginated queries.
pub fn default_limit() -> i64 {
    20
}

/// Calculates the SQL OFFSET safely from a 1-indexed page and a limit.
/// It coerces the page to be at least 1, and the limit to be at least 0.
pub fn calculate_offset(page: i64, limit: i64) -> i64 {
    (page.max(1) - 1) * limit.max(0)
}

/// Wrapper for paginated API responses.
///
/// Indicates the total amount of available items across all pages,
/// as well as the 1-indexed current page and computed total pages.
#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct PaginatedResponse<T> {
    /// The chunk of items for the current page.
    pub items: Vec<T>,
    /// Total absolute number of items in the database matching the query.
    pub total: i64,
    /// The current 1-indexed page.
    pub page: i64,
    /// The maximum number of items per page.
    pub limit: i64,
    /// The calculated total number of pages based on total / limit.
    pub total_pages: i64,
}

impl<T> PaginatedResponse<T> {
    /// Creates a new `PaginatedResponse` automatically calculating `total_pages`.
    /// `page` must be >= 1.
    pub fn new(items: Vec<T>, total: i64, page: i64, limit: i64) -> Self {
        let total_pages = if limit > 0 {
            (total + limit - 1) / limit
        } else {
            0
        };

        Self {
            items,
            total,
            page,
            limit,
            total_pages,
        }
    }

    /// Transforms the items inside the paginated response.
    pub fn map<U, F: FnMut(T) -> U>(self, f: F) -> PaginatedResponse<U> {
        PaginatedResponse {
            items: self.items.into_iter().map(f).collect(),
            total: self.total,
            page: self.page,
            limit: self.limit,
            total_pages: self.total_pages,
        }
    }
}
