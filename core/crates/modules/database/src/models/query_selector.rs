use super::types::DatabaseQuery;

/// Trait for types that can be used as database queries.
///
/// This trait enables the [`DatabaseProvider`] to accept both plain SQL strings
/// and [`DatabaseQuery`] structs. All queries are `PostgreSQL` queries only.
///
/// # Implementations
///
/// - `str` - Returns the string as-is (`PostgreSQL` query)
/// - `String` - Returns the string as-is (`PostgreSQL` query)
/// - [`DatabaseQuery`] - Returns the `PostgreSQL` query variant
///
/// # Example
///
/// ```rust
/// use systemprompt_database::{DatabaseQuery, QuerySelector};
///
/// fn example() {
///     let plain_query = "SELECT 1";
///     let pg_query = DatabaseQuery::new("SELECT CURRENT_TIMESTAMP");
///
///     let sql1: &dyn QuerySelector = &plain_query;
///     let sql2: &dyn QuerySelector = &pg_query;
///
///     let selected1 = sql1.select_query();
///     let selected2 = sql2.select_query();
/// }
/// ```
pub trait QuerySelector: Sync {
    fn select_query(&self) -> &str;
}

impl QuerySelector for &str {
    fn select_query(&self) -> &str {
        self
    }
}

impl QuerySelector for String {
    fn select_query(&self) -> &str {
        self.as_str()
    }
}

impl QuerySelector for DatabaseQuery {
    fn select_query(&self) -> &str {
        self.postgres()
    }
}
