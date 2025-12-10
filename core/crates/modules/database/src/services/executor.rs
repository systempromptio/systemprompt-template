use super::database::Database;
use crate::models::QueryResult;
use anyhow::{Context, Result};

#[derive(Debug, Copy, Clone)]
pub struct SqlExecutor;

impl SqlExecutor {
    /// Execute multiple SQL statements from a string
    pub async fn execute_statements(db: &Database, sql: &str) -> Result<()> {
        db.execute_batch(sql)
            .await
            .context("Failed to execute SQL statements")
    }

    /// Execute a single query and return results
    pub async fn execute_query(db: &Database, query: &str) -> Result<QueryResult> {
        db.query(&query).await.context("Failed to execute query")
    }

    /// Execute SQL from a file
    pub async fn execute_file(db: &Database, file_path: &str) -> Result<()> {
        let sql = std::fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read SQL file: {file_path}"))?;

        Self::execute_statements(db, &sql).await
    }

    /// Check if a table exists in `PostgreSQL`
    pub async fn table_exists(db: &Database, table_name: &str) -> Result<bool> {
        let query = format!(
            "SELECT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = \
             '{table_name}')"
        );

        let result = db.query(&query).await?;
        Ok(!result.rows.is_empty())
    }

    /// Check if a column exists in a `PostgreSQL` table
    pub async fn column_exists(db: &Database, table_name: &str, column_name: &str) -> Result<bool> {
        let query = format!(
            "SELECT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = \
             '{table_name}' AND column_name = '{column_name}')"
        );
        let result = db.query(&query).await?;

        for row in &result.rows {
            if let Some(exists) = row.get("exists").and_then(serde_json::Value::as_bool) {
                return Ok(exists);
            }
        }

        Ok(false)
    }
}
