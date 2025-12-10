use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sqlx::postgres::{PgConnectOptions, PgPool};
use sqlx::{Column, Executor, Row};
use std::str::FromStr;
use std::sync::Arc;

use super::postgres_helpers::{bind_params, row_to_json};
use super::postgres_transaction::PostgresTransaction;
use super::provider::DatabaseProvider;
use crate::models::{
    ColumnInfo, DatabaseInfo, DatabaseTransaction, DbValue, JsonRow, QueryResult, QuerySelector,
    TableInfo, ToDbValue,
};

#[derive(Debug)]
pub struct PostgresProvider {
    pool: Arc<PgPool>,
}

impl PostgresProvider {
    pub async fn new(database_url: &str) -> Result<Self> {
        let mut connect_options = PgConnectOptions::from_str(database_url)
            .map_err(|e| anyhow!("Failed to parse database URL: {e}"))?;

        connect_options = connect_options
            .application_name("systemprompt")
            .statement_cache_capacity(0)
            .ssl_mode(sqlx::postgres::PgSslMode::Prefer);

        if let Some(ca_cert_path) = Self::get_cert_path() {
            connect_options = connect_options.ssl_root_cert(&ca_cert_path);
        }

        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(50)
            .min_connections(0)
            .max_lifetime(std::time::Duration::from_secs(1800))
            .acquire_timeout(std::time::Duration::from_secs(30))
            .idle_timeout(std::time::Duration::from_secs(300))
            .connect_with(connect_options)
            .await
            .map_err(|e| anyhow!("Failed to connect to PostgreSQL: {e}"))?;

        Ok(Self {
            pool: Arc::new(pool),
        })
    }

    fn get_cert_path() -> Option<std::path::PathBuf> {
        std::env::var("PGCA_CERT_PATH")
            .ok()
            .map(std::path::PathBuf::from)
    }

    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

#[async_trait]
impl DatabaseProvider for PostgresProvider {
    fn get_postgres_pool(&self) -> Option<Arc<PgPool>> {
        Some(Arc::clone(&self.pool))
    }

    async fn execute(&self, query: &dyn QuerySelector, params: &[&dyn ToDbValue]) -> Result<u64> {
        let sql = query.select_query();
        let query_obj = sqlx::query(sql);
        let query_obj = bind_params(query_obj, params);

        let result = query_obj
            .execute(&*self.pool)
            .await
            .map_err(|e| anyhow!("Query execution failed: {e}"))?;

        Ok(result.rows_affected())
    }

    async fn execute_raw(&self, sql: &str) -> Result<()> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| anyhow!("Failed to acquire connection: {e}"))?;

        conn.execute(sql)
            .await
            .map_err(|e| anyhow!("Raw query execution failed: {e}"))?;

        Ok(())
    }

    async fn fetch_all(
        &self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<Vec<JsonRow>> {
        let sql = query.select_query();
        let query_obj = sqlx::query(sql);
        let query_obj = bind_params(query_obj, params);

        let rows = query_obj
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| anyhow!("Query execution failed: {e}"))?;

        Ok(rows.iter().map(row_to_json).collect())
    }

    async fn fetch_one(
        &self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<JsonRow> {
        let sql = query.select_query();
        let query_obj = sqlx::query(sql);
        let query_obj = bind_params(query_obj, params);

        let row = query_obj
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| anyhow!("Query execution failed: {e}"))?;

        Ok(row_to_json(&row))
    }

    async fn fetch_optional(
        &self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<Option<JsonRow>> {
        let sql = query.select_query();
        let query_obj = sqlx::query(sql);
        let query_obj = bind_params(query_obj, params);

        let row = query_obj
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| anyhow!("Query execution failed: {e}"))?;

        Ok(row.map(|r| row_to_json(&r)))
    }

    async fn fetch_scalar_value(
        &self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<DbValue> {
        let row = self.fetch_one(query, params).await?;

        let first_value = row
            .values()
            .next()
            .ok_or_else(|| anyhow!("No columns in result"))?;

        let db_value = match first_value {
            serde_json::Value::String(s) => DbValue::String(s.clone()),
            serde_json::Value::Number(n) => n
                .as_i64()
                .map(DbValue::Int)
                .or_else(|| n.as_f64().map(DbValue::Float))
                .unwrap_or(DbValue::NullFloat),
            serde_json::Value::Bool(b) => DbValue::Bool(*b),
            serde_json::Value::Null => DbValue::NullString,
            _ => return Err(anyhow!("Unsupported value type")),
        };

        Ok(db_value)
    }

    async fn begin_transaction(&self) -> Result<Box<dyn DatabaseTransaction>> {
        let tx = self
            .pool
            .begin()
            .await
            .map_err(|e| anyhow!("Failed to begin transaction: {e}"))?;

        Ok(Box::new(PostgresTransaction::new(tx)))
    }

    async fn get_database_info(&self) -> Result<DatabaseInfo> {
        let version_row = sqlx::query("SELECT version() as version")
            .fetch_one(&*self.pool)
            .await?;
        let version: String = version_row.try_get("version")?;

        let table_rows = sqlx::query(
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' ORDER \
             BY table_name",
        )
        .fetch_all(&*self.pool)
        .await?;

        let mut tables = Vec::new();
        for table_row in table_rows {
            let table_name: String = table_row.try_get("table_name")?;

            let count_query = format!("SELECT COUNT(*) as count FROM {table_name}");
            let count_row = sqlx::query(&count_query).fetch_one(&*self.pool).await?;
            let row_count: i64 = count_row.try_get("count")?;

            let columns_query = format!(
                "SELECT column_name, data_type, is_nullable FROM information_schema.columns WHERE \
                 table_name = '{table_name}' ORDER BY ordinal_position"
            );
            let column_rows = sqlx::query(&columns_query).fetch_all(&*self.pool).await?;

            let mut columns = Vec::new();
            for col_row in column_rows {
                let col_name: String = col_row.try_get("column_name")?;
                let col_type: String = col_row.try_get("data_type")?;
                let is_nullable: String = col_row.try_get("is_nullable")?;

                columns.push(ColumnInfo {
                    name: col_name,
                    data_type: col_type,
                    nullable: is_nullable == "YES",
                    primary_key: false,
                    default: None,
                });
            }

            tables.push(TableInfo {
                name: table_name,
                row_count,
                columns,
            });
        }

        Ok(DatabaseInfo {
            path: "PostgreSQL".to_string(),
            size: 0,
            version,
            tables,
        })
    }

    async fn test_connection(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| anyhow!("Connection test failed: {e}"))?;
        Ok(())
    }

    async fn execute_batch(&self, sql: &str) -> Result<()> {
        for statement in sql.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed)
                    .execute(&*self.pool)
                    .await
                    .map_err(|e| anyhow!("Batch execution failed: {e}"))?;
            }
        }
        Ok(())
    }

    async fn query_raw(&self, query: &dyn QuerySelector) -> Result<QueryResult> {
        let sql = query.select_query();
        let start = std::time::Instant::now();

        let rows = sqlx::query(sql)
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| anyhow!("Query execution failed: {e}"))?;

        let mut columns = Vec::new();
        let mut result_rows = Vec::new();

        if let Some(first_row) = rows.first() {
            columns = first_row
                .columns()
                .iter()
                .map(|c| c.name().to_string())
                .collect();
        }

        for row in rows {
            result_rows.push(row_to_json(&row));
        }

        let row_count = result_rows.len();

        #[allow(clippy::cast_possible_truncation)]
        let execution_time_ms = start.elapsed().as_millis() as u64;

        Ok(QueryResult {
            columns,
            rows: result_rows,
            row_count,
            execution_time_ms,
        })
    }

    async fn query_raw_with(
        &self,
        query: &dyn QuerySelector,
        _params: Vec<serde_json::Value>,
    ) -> Result<QueryResult> {
        self.query_raw(query).await
    }
}
