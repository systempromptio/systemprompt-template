use anyhow::Result;
use sqlx::postgres::PgRow;

/// Trait for converting `PostgreSQL` rows to typed models.
///
/// This trait enables compile-time type safety when deserializing database results,
/// eliminating the need for manual JSON conversion through [`JsonRow`].
///
/// Implementations should handle conversion from `PostgreSQL` row types
/// and perform any necessary type conversions.
///
/// # Example
///
/// ```rust
/// use systemprompt_database::FromDatabaseRow;
/// use sqlx::Row;
///
/// struct User {
///     id: String,
///     name: String,
/// }
///
/// impl FromDatabaseRow for User {
///     fn from_postgres_row(row: &PgRow) -> Result<Self> {
///         Ok(Self {
///             id: row.get("id"),
///             name: row.get("name"),
///         })
///     }
/// }
/// ```
pub trait FromDatabaseRow: Sized {
    /// Convert a `PostgreSQL` row to this type.
    fn from_postgres_row(row: &PgRow) -> Result<Self>;
}
