/// Create a [`DatabaseQuery`] from a query file path.
///
/// This macro automatically loads `PostgreSQL` query using [`include_str!`] at compile time,
/// ensuring missing query files result in compilation errors rather than runtime failures.
///
/// # File Structure
///
/// Given a path like `"users/create"`, the macro looks for:
/// - `queries/postgres/users/create.sql` (`PostgreSQL` query)
///
/// # Example
///
/// ```rust
/// use systemprompt_database::{DatabaseQuery, database_query};
///
/// const CREATE_USER: DatabaseQuery = database_query!("users/create");
/// const GET_USER: DatabaseQuery = database_query!("users/get_by_id");
/// ```
///
/// # Benefits
///
/// - **Compile-time validation**: Missing query files cause compile errors
/// - **Type-safe**: Returns [`DatabaseQuery`] struct
/// - **Clean syntax**: Single line per query
/// - **`PostgreSQL`-only**: All queries use `PostgreSQL` syntax
#[macro_export]
macro_rules! database_query {
    ($path:literal) => {
        $crate::DatabaseQuery::new(include_str!(concat!("../queries/postgres/", $path, ".sql")))
    };
}
