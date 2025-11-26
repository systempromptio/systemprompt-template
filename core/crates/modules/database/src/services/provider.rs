use crate::models::{
    DatabaseInfo, DatabaseTransaction, DbValue, FromDatabaseRow, JsonRow, QueryResult,
    QuerySelector, ToDbValue,
};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

/// Database provider abstraction for `PostgreSQL` operations.
///
/// This trait provides a consistent interface for database operations.
/// All implementations use `PostgreSQL` as the backend.
///
/// # Example
///
/// ```rust
/// use systemprompt_database::DatabaseProvider;
///
/// async fn example(db: &dyn DatabaseProvider) -> Result<()> {
///     // Fetch single row
///     let row = db.fetch_one("SELECT * FROM users WHERE id = $1", &[&"123"]).await?;
///
///     // Execute write operation
///     let rows_affected = db.execute(
///         "UPDATE users SET email = $1 WHERE id = $2",
///         &[&"new@email.com", &"123"]
///     ).await?;
///
///     Ok(())
/// }
/// ```
///
/// # Transaction Support
///
/// ```rust
/// async fn transfer(db: &dyn DatabaseProvider) -> Result<()> {
///     let mut tx = db.begin_transaction().await?;
///     tx.execute("UPDATE accounts SET balance = balance - $1 WHERE id = $2", &[&100, &"acc1"]).await?;
///     tx.execute("UPDATE accounts SET balance = balance + $1 WHERE id = $2", &[&100, &"acc2"]).await?;
///     tx.commit().await?;
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait DatabaseProvider: Send + Sync + std::fmt::Debug {
    /// Get `PostgreSQL` pool
    fn get_postgres_pool(&self) -> Option<Arc<sqlx::PgPool>> {
        None
    }

    /// Check if this database provider is `PostgreSQL` (always true - `SQLite` is no longer supported)
    fn is_postgres(&self) -> bool {
        true
    }

    /// Execute a write operation (INSERT, UPDATE, DELETE).
    ///
    /// Returns the number of rows affected by the operation.
    ///
    /// # Arguments
    ///
    /// * `query` - SQL query (string or `DatabaseQuery`) with `?` placeholders for parameters
    /// * `params` - Parameter values to bind to the query
    ///
    /// # Example
    ///
    /// ```rust
    /// // With plain string
    /// let rows = db.execute(
    ///     "UPDATE users SET email = ? WHERE id = ?",
    ///     &[&"new@email.com", &"123"]
    /// ).await?;
    ///
    /// // With DatabaseQuery
    /// const UPDATE_EMAIL: DatabaseQuery = database_query!("users/update_email");
    /// let rows = db.execute(&UPDATE_EMAIL, &[&"new@email.com", &"123"]).await?;
    /// ```
    async fn execute(&self, query: &dyn QuerySelector, params: &[&dyn ToDbValue]) -> Result<u64>;

    /// Execute raw SQL using `PostgreSQL`'s simple query protocol.
    ///
    /// This method is designed for DDL operations (CREATE, DROP, ALTER) that need to
    /// bypass `PostgreSQL`'s extended query protocol. The extended protocol evaluates
    /// conditions like `IF EXISTS` at bind-time, causing issues with idempotent
    /// migrations. The simple protocol evaluates them at parse-time.
    ///
    /// # Arguments
    ///
    /// * `sql` - Raw SQL string to execute (no parameter binding)
    ///
    /// # Example
    ///
    /// ```rust
    /// db.execute_raw("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)").await?;
    /// db.execute_raw("DROP VIEW IF EXISTS v_stats CASCADE").await?;
    /// ```
    async fn execute_raw(&self, sql: &str) -> Result<()>;

    /// Fetch all rows matching the query.
    ///
    /// Returns a vector of rows as JSON objects (`HashMap<String, Value>`).
    ///
    /// # Arguments
    ///
    /// * `query` - SQL query (string or `DatabaseQuery`) with `?` placeholders for parameters
    /// * `params` - Parameter values to bind to the query
    ///
    /// # Example
    ///
    /// ```rust
    /// let rows = db.fetch_all("SELECT * FROM users", &[]).await?;
    /// for row in rows {
    ///     let name = row.get("name").and_then(|v| v.as_str());
    /// }
    /// ```
    async fn fetch_all(
        &self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<Vec<JsonRow>>;

    /// Fetch exactly one row.
    ///
    /// Returns an error if no rows are found or if multiple rows are returned.
    ///
    /// # Arguments
    ///
    /// * `query` - SQL query (string or `DatabaseQuery`) with `?` placeholders for parameters
    /// * `params` - Parameter values to bind to the query
    ///
    /// # Example
    ///
    /// ```rust
    /// let row = db.fetch_one("SELECT * FROM users WHERE id = ?", &[&"123"]).await?;
    /// ```
    async fn fetch_one(
        &self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<JsonRow>;

    /// Fetch zero or one row.
    ///
    /// Returns `Ok(None)` if no rows are found, `Ok(Some(row))` if exactly one row
    /// is found, or an error if multiple rows are returned.
    ///
    /// # Arguments
    ///
    /// * `query` - SQL query (string or `DatabaseQuery`) with `?` placeholders for parameters
    /// * `params` - Parameter values to bind to the query
    ///
    /// # Example
    ///
    /// ```rust
    /// let row = db.fetch_optional("SELECT * FROM users WHERE email = ?", &[&"test@example.com"]).await?;
    /// if let Some(user) = row {
    ///     println!("Found user: {:?}", user);
    /// }
    /// ```
    async fn fetch_optional(
        &self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<Option<JsonRow>>;

    /// Fetch a single scalar value (count, sum, etc).
    ///
    /// Returns the first column of the first row as a `DbValue`.
    ///
    /// # Arguments
    ///
    /// * `query` - SQL query (string or `DatabaseQuery`) with `?` placeholders for parameters
    /// * `params` - Parameter values to bind to the query
    ///
    /// # Example
    ///
    /// ```rust
    /// let count = db.fetch_scalar_value("SELECT COUNT(*) FROM users", &[]).await?;
    /// if let DbValue::Integer(n) = count {
    ///     println!("Total users: {}", n);
    /// }
    /// ```
    async fn fetch_scalar_value(
        &self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<DbValue>;

    /// Begin a new database transaction.
    ///
    /// Returns a transaction object that can be used to execute multiple
    /// operations atomically. Call `commit()` to save changes or drop the
    /// transaction to rollback.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut tx = db.begin_transaction().await?;
    /// tx.execute("INSERT INTO users (id, name) VALUES (?, ?)", &[&"1", &"Alice"]).await?;
    /// tx.execute("INSERT INTO users (id, name) VALUES (?, ?)", &[&"2", &"Bob"]).await?;
    /// tx.commit().await?;
    /// ```
    async fn begin_transaction(&self) -> Result<Box<dyn DatabaseTransaction>>;

    /// Get information about the database connection.
    ///
    /// Returns metadata including database type, version, and connection details.
    async fn get_database_info(&self) -> Result<DatabaseInfo>;

    /// Test the database connection.
    ///
    /// Returns `Ok(())` if the connection is working, otherwise an error.
    async fn test_connection(&self) -> Result<()>;

    /// Execute a batch of SQL statements.
    ///
    /// Useful for running migrations or initialization scripts. Statements
    /// should be separated by semicolons.
    ///
    /// # Arguments
    ///
    /// * `sql` - Multiple SQL statements separated by semicolons
    async fn execute_batch(&self, sql: &str) -> Result<()>;

    /// Execute a raw query and return results as `QueryResult`.
    ///
    /// This is a lower-level API that returns results in a structured format
    /// including column metadata.
    ///
    /// # Arguments
    ///
    /// * `query` - SQL query (string or `DatabaseQuery`) to execute
    async fn query_raw(&self, query: &dyn QuerySelector) -> Result<QueryResult>;

    /// Execute a raw query with JSON parameters.
    ///
    /// This is a lower-level API that accepts parameters as JSON values and
    /// returns results in a structured format.
    ///
    /// # Arguments
    ///
    /// * `query` - SQL query (string or `DatabaseQuery`) with `?` placeholders
    /// * `params` - Parameter values as JSON
    async fn query_raw_with(
        &self,
        query: &dyn QuerySelector,
        params: Vec<serde_json::Value>,
    ) -> Result<QueryResult>;
}

/// Extension trait for typed database queries.
///
/// This trait provides generic methods for fetching strongly-typed rows from the database.
/// It is implemented for all types that implement `DatabaseProvider`, enabling compile-time
/// type safety without requiring manual JSON deserialization.
///
/// # Example
///
/// ```rust
/// use systemprompt_database::{DatabaseProvider, DatabaseProviderExt, FromDatabaseRow};
///
/// async fn get_user(db: &dyn DatabaseProvider, email: &str) -> Result<Option<User>> {
///     let query = DatabaseQueryEnum::GetUserByEmail.get(db);
///     db.as_typed()
///         .fetch_typed_optional::<User>(&query, &[&email])
///         .await
/// }
/// ```
#[allow(async_fn_in_trait)]
pub trait DatabaseProviderExt {
    /// Fetch zero or one typed row.
    ///
    /// Returns `Ok(None)` if no rows are found, `Ok(Some(T))` if exactly one row is found,
    /// or an error if multiple rows are returned or deserialization fails.
    async fn fetch_typed_optional<T: FromDatabaseRow>(
        &self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<Option<T>>;

    /// Fetch exactly one typed row.
    ///
    /// Returns an error if no rows are found, multiple rows are returned, or
    /// deserialization fails.
    async fn fetch_typed_one<T: FromDatabaseRow>(
        &self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<T>;

    /// Fetch all typed rows.
    ///
    /// Returns a vector of deserialized rows. Deserialization errors are propagated.
    async fn fetch_typed_all<T: FromDatabaseRow>(
        &self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<Vec<T>>;
}
