use anyhow::{anyhow, Result};
use async_trait::async_trait;
use sqlx::{
    postgres::{PgConnectOptions, PgPool},
    Column, Executor, Row,
};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use super::provider::{DatabaseProvider, DatabaseProviderExt};
use crate::models::{
    ColumnInfo, DatabaseInfo, DatabaseTransaction, DbValue, FromDatabaseRow, JsonRow, QueryResult,
    QuerySelector, TableInfo, ToDbValue,
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

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    fn row_to_json(row: &sqlx::postgres::PgRow) -> HashMap<String, serde_json::Value> {
        let mut map = HashMap::new();

        for column in row.columns() {
            let name = column.name().to_string();

            if let Ok(val) = row.try_get::<Option<chrono::NaiveDateTime>, _>(column.ordinal()) {
                map.insert(
                    name,
                    val.map_or(serde_json::Value::Null, |v| {
                        serde_json::Value::String(v.and_utc().to_rfc3339())
                    }),
                );
            } else if let Ok(val) =
                row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>(column.ordinal())
            {
                map.insert(
                    name,
                    val.map_or(serde_json::Value::Null, |v| {
                        serde_json::Value::String(v.to_rfc3339())
                    }),
                );
            } else if let Ok(val) = row.try_get::<Option<String>, _>(column.ordinal()) {
                map.insert(
                    name,
                    val.map_or(serde_json::Value::Null, serde_json::Value::String),
                );
            } else if let Ok(val) = row.try_get::<Option<i64>, _>(column.ordinal()) {
                map.insert(
                    name,
                    val.map_or(serde_json::Value::Null, |v| {
                        serde_json::Value::Number(v.into())
                    }),
                );
            } else if let Ok(val) = row.try_get::<Option<i32>, _>(column.ordinal()) {
                map.insert(
                    name,
                    val.map_or(serde_json::Value::Null, |v| {
                        serde_json::Value::Number(i64::from(v).into())
                    }),
                );
            } else if let Ok(val) = row.try_get::<Option<f64>, _>(column.ordinal()) {
                map.insert(
                    name,
                    val.map_or(serde_json::Value::Null, |v| serde_json::json!(v)),
                );
            } else if let Ok(val) =
                row.try_get::<Option<rust_decimal::Decimal>, _>(column.ordinal())
            {
                map.insert(
                    name,
                    val.map_or(serde_json::Value::Null, |v| {
                        serde_json::json!(v.to_string().parse::<f64>().unwrap_or(0.0))
                    }),
                );
            } else if let Ok(val) = row.try_get::<Option<bool>, _>(column.ordinal()) {
                map.insert(
                    name,
                    val.map_or(serde_json::Value::Null, serde_json::Value::Bool),
                );
            } else if let Ok(val) = row.try_get::<Option<Vec<String>>, _>(column.ordinal()) {
                map.insert(
                    name,
                    val.map_or(serde_json::Value::Null, |v| {
                        serde_json::Value::Array(
                            v.into_iter().map(serde_json::Value::String).collect(),
                        )
                    }),
                );
            } else if let Ok(val) = row.try_get::<Option<serde_json::Value>, _>(column.ordinal()) {
                map.insert(name, val.unwrap_or(serde_json::Value::Null));
            } else if let Ok(val) = row.try_get::<Option<Vec<u8>>, _>(column.ordinal()) {
                map.insert(
                    name,
                    val.map_or(serde_json::Value::Null, |bytes| {
                        use base64::{engine::general_purpose::STANDARD, Engine};
                        serde_json::Value::String(STANDARD.encode(&bytes))
                    }),
                );
            } else {
                map.insert(name, serde_json::Value::Null);
            }
        }

        map
    }

    fn bind_params<'q>(
        mut query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
        params: &[&dyn ToDbValue],
    ) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
        for param in params {
            let value = param.to_db_value();
            query = match value {
                DbValue::String(s) => query.bind(s),
                DbValue::Int(i) => query.bind(i),
                DbValue::Float(f) => query.bind(f),
                DbValue::Bool(b) => query.bind(b),
                DbValue::Bytes(b) => query.bind(b),
                DbValue::Timestamp(dt) => query.bind(dt),
                DbValue::StringArray(arr) => query.bind(arr),
                DbValue::NullString => query.bind(None::<String>),
                DbValue::NullInt => query.bind(None::<i64>),
                DbValue::NullFloat => query.bind(None::<f64>),
                DbValue::NullBool => query.bind(None::<bool>),
                DbValue::NullBytes => query.bind(None::<Vec<u8>>),
                DbValue::NullTimestamp => query.bind(None::<chrono::DateTime<chrono::Utc>>),
                DbValue::NullStringArray => query.bind(None::<Vec<String>>),
            };
        }
        query
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
        let query_obj = Self::bind_params(query_obj, params);

        let result = query_obj
            .execute(&*self.pool)
            .await
            .map_err(|e| anyhow!("Query execution failed: {e}"))?;

        Ok(result.rows_affected())
    }

    async fn execute_raw(&self, sql: &str) -> Result<()> {
        // Use PostgreSQL's simple query protocol (not extended protocol)
        // This is critical for DDL operations with IF EXISTS/IF NOT EXISTS
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|e| anyhow!("Failed to acquire connection: {e}"))?;

        // conn.execute() uses simple protocol: send SQL string directly
        // No prepare/bind/execute cycle that causes metadata conflicts
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
        let query_obj = Self::bind_params(query_obj, params);

        let rows = query_obj
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| anyhow!("Query execution failed: {e}"))?;

        Ok(rows.iter().map(Self::row_to_json).collect())
    }

    async fn fetch_one(
        &self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<JsonRow> {
        let sql = query.select_query();
        let query_obj = sqlx::query(sql);
        let query_obj = Self::bind_params(query_obj, params);

        let row = query_obj
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| anyhow!("Query execution failed: {e}"))?;

        Ok(Self::row_to_json(&row))
    }

    async fn fetch_optional(
        &self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<Option<JsonRow>> {
        let sql = query.select_query();
        let query_obj = sqlx::query(sql);
        let query_obj = Self::bind_params(query_obj, params);

        let row = query_obj
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| anyhow!("Query execution failed: {e}"))?;

        Ok(row.map(|r| Self::row_to_json(&r)))
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
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    DbValue::Int(i)
                } else if let Some(f) = n.as_f64() {
                    DbValue::Float(f)
                } else {
                    DbValue::NullFloat
                }
            },
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
            "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' ORDER BY table_name"
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
                "SELECT column_name, data_type, is_nullable FROM information_schema.columns WHERE table_name = '{table_name}' ORDER BY ordinal_position"
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
            result_rows.push(Self::row_to_json(&row));
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

pub struct PostgresTransaction {
    tx: Option<sqlx::Transaction<'static, sqlx::Postgres>>,
}

impl std::fmt::Debug for PostgresTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PostgresTransaction")
            .field("tx", &self.tx.is_some())
            .finish()
    }
}

impl PostgresTransaction {
    const fn new(tx: sqlx::Transaction<'static, sqlx::Postgres>) -> Self {
        Self { tx: Some(tx) }
    }
}

fn row_to_json_tx(row: &sqlx::postgres::PgRow) -> HashMap<String, serde_json::Value> {
    let mut map = HashMap::new();

    for column in row.columns() {
        let name = column.name().to_string();

        if let Ok(val) = row.try_get::<Option<chrono::NaiveDateTime>, _>(column.ordinal()) {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, |v| {
                    serde_json::Value::String(v.and_utc().to_rfc3339())
                }),
            );
        } else if let Ok(val) =
            row.try_get::<Option<chrono::DateTime<chrono::Utc>>, _>(column.ordinal())
        {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, |v| {
                    serde_json::Value::String(v.to_rfc3339())
                }),
            );
        } else if let Ok(val) = row.try_get::<Option<String>, _>(column.ordinal()) {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, serde_json::Value::String),
            );
        } else if let Ok(val) = row.try_get::<Option<i64>, _>(column.ordinal()) {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, |v| {
                    serde_json::Value::Number(v.into())
                }),
            );
        } else if let Ok(val) = row.try_get::<Option<i32>, _>(column.ordinal()) {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, |v| {
                    serde_json::Value::Number(i64::from(v).into())
                }),
            );
        } else if let Ok(val) = row.try_get::<Option<f64>, _>(column.ordinal()) {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, |v| serde_json::json!(v)),
            );
        } else if let Ok(val) = row.try_get::<Option<bool>, _>(column.ordinal()) {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, serde_json::Value::Bool),
            );
        } else if let Ok(val) = row.try_get::<Option<Vec<String>>, _>(column.ordinal()) {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, |v| {
                    serde_json::Value::Array(v.into_iter().map(serde_json::Value::String).collect())
                }),
            );
        } else if let Ok(val) = row.try_get::<Option<serde_json::Value>, _>(column.ordinal()) {
            map.insert(name, val.unwrap_or(serde_json::Value::Null));
        } else if let Ok(val) = row.try_get::<Option<Vec<u8>>, _>(column.ordinal()) {
            map.insert(
                name,
                val.map_or(serde_json::Value::Null, |bytes| {
                    use base64::{engine::general_purpose::STANDARD, Engine};
                    serde_json::Value::String(STANDARD.encode(&bytes))
                }),
            );
        } else {
            map.insert(name, serde_json::Value::Null);
        }
    }

    map
}

fn bind_params_tx<'q>(
    mut query: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
    params: &[&dyn ToDbValue],
) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
    for param in params {
        let value = param.to_db_value();
        query = match value {
            DbValue::String(s) => query.bind(s),
            DbValue::Int(i) => query.bind(i),
            DbValue::Float(f) => query.bind(f),
            DbValue::Bool(b) => query.bind(b),
            DbValue::Bytes(b) => query.bind(b),
            DbValue::Timestamp(dt) => query.bind(dt),
            DbValue::StringArray(arr) => query.bind(arr),
            DbValue::NullString => query.bind(None::<String>),
            DbValue::NullInt => query.bind(None::<i64>),
            DbValue::NullFloat => query.bind(None::<f64>),
            DbValue::NullBool => query.bind(None::<bool>),
            DbValue::NullBytes => query.bind(None::<Vec<u8>>),
            DbValue::NullTimestamp => query.bind(None::<chrono::DateTime<chrono::Utc>>),
            DbValue::NullStringArray => query.bind(None::<Vec<String>>),
        };
    }
    query
}

#[async_trait]
impl DatabaseTransaction for PostgresTransaction {
    async fn execute(
        &mut self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<u64> {
        let sql = query.select_query();
        let tx = self
            .tx
            .as_mut()
            .ok_or_else(|| anyhow!("Transaction already consumed"))?;

        let query_obj = sqlx::query(sql);
        let query_obj = bind_params_tx(query_obj, params);

        let result = query_obj
            .execute(&mut **tx)
            .await
            .map_err(|e| anyhow!("Query execution failed: {e}"))?;

        Ok(result.rows_affected())
    }

    async fn fetch_all(
        &mut self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<Vec<JsonRow>> {
        let sql = query.select_query();
        let tx = self
            .tx
            .as_mut()
            .ok_or_else(|| anyhow!("Transaction already consumed"))?;

        let query_obj = sqlx::query(sql);
        let query_obj = bind_params_tx(query_obj, params);

        let rows = query_obj
            .fetch_all(&mut **tx)
            .await
            .map_err(|e| anyhow!("Query execution failed: {e}"))?;

        Ok(rows.iter().map(row_to_json_tx).collect())
    }

    async fn fetch_one(
        &mut self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<JsonRow> {
        let sql = query.select_query();
        let tx = self
            .tx
            .as_mut()
            .ok_or_else(|| anyhow!("Transaction already consumed"))?;

        let query_obj = sqlx::query(sql);
        let query_obj = bind_params_tx(query_obj, params);

        let row = query_obj
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| anyhow!("Query execution failed: {e}"))?;

        Ok(row_to_json_tx(&row))
    }

    async fn fetch_optional(
        &mut self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<Option<JsonRow>> {
        let sql = query.select_query();
        let tx = self
            .tx
            .as_mut()
            .ok_or_else(|| anyhow!("Transaction already consumed"))?;

        let query_obj = sqlx::query(sql);
        let query_obj = bind_params_tx(query_obj, params);

        let row = query_obj
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| anyhow!("Query execution failed: {e}"))?;

        Ok(row.map(|r| row_to_json_tx(&r)))
    }

    async fn commit(mut self: Box<Self>) -> Result<()> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| anyhow!("Transaction already consumed"))?;

        tx.commit()
            .await
            .map_err(|e| anyhow!("Transaction commit failed: {e}"))?;

        Ok(())
    }

    async fn rollback(mut self: Box<Self>) -> Result<()> {
        let tx = self
            .tx
            .take()
            .ok_or_else(|| anyhow!("Transaction already consumed"))?;

        tx.rollback()
            .await
            .map_err(|e| anyhow!("Transaction rollback failed: {e}"))?;

        Ok(())
    }
}

impl DatabaseProviderExt for PostgresProvider {
    async fn fetch_typed_optional<T: FromDatabaseRow>(
        &self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<Option<T>> {
        let sql = query.select_query();
        let query_obj = sqlx::query(sql);
        let query_obj = Self::bind_params(query_obj, params);

        let row = query_obj
            .fetch_optional(&*self.pool)
            .await
            .map_err(|e| anyhow!("Query execution failed: {e}"))?;

        match row {
            Some(r) => Ok(Some(T::from_postgres_row(&r)?)),
            None => Ok(None),
        }
    }

    async fn fetch_typed_one<T: FromDatabaseRow>(
        &self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<T> {
        let sql = query.select_query();
        let query_obj = sqlx::query(sql);
        let query_obj = Self::bind_params(query_obj, params);

        let row = query_obj
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| anyhow!("Query execution failed: {e}"))?;

        T::from_postgres_row(&row)
    }

    async fn fetch_typed_all<T: FromDatabaseRow>(
        &self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<Vec<T>> {
        let sql = query.select_query();
        let query_obj = sqlx::query(sql);
        let query_obj = Self::bind_params(query_obj, params);

        let rows = query_obj
            .fetch_all(&*self.pool)
            .await
            .map_err(|e| anyhow!("Query execution failed: {e}"))?;

        rows.iter().map(|r| T::from_postgres_row(r)).collect()
    }
}
