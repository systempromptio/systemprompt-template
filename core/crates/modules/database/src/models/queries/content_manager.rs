use super::super::DatabaseQueryEnum;

/// Maps [`DatabaseQueryEnum`] variants for Content Manager module to SQL file paths.
///
/// Returns Some(&'static str) if this variant belongs to the module,
/// None otherwise.
pub const fn get_query(_variant: DatabaseQueryEnum) -> Option<&'static str> {
    None
}
