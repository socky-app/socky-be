/// Trait representing the deletability capabilities of a table.
pub trait Deletability {
    /// If true, queries will automatically append `AND deleted_at IS NULL`
    const SUPPORTS_SOFT_DELETE: bool = false;
}

/// Marker for tables that DO NOT use soft delete.
pub struct HardDeleteStrategy;
impl Deletability for HardDeleteStrategy {
    const SUPPORTS_SOFT_DELETE: bool = false;
}

/// Marker for tables that DO use soft delete.
pub struct SoftDeleteStrategy;
impl Deletability for SoftDeleteStrategy {
    const SUPPORTS_SOFT_DELETE: bool = true;
}