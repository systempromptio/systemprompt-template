use super::postgres::PostgresProvider;
use super::provider::DatabaseProvider;
use crate::models::{DatabaseInfo, QueryResult};
use anyhow::Result;
use std::sync::Arc;

pub struct Database {
    provider: Arc<dyn DatabaseProvider>,
}

impl std::fmt::Debug for Database {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Database")
            .field("backend", &"PostgreSQL")
            .finish()
    }
}

impl Database {
    pub async fn new_postgres(url: &str) -> Result<Self> {
        let provider = PostgresProvider::new(url).await?;
        Ok(Self {
            provider: Arc::new(provider),
        })
    }

    pub async fn from_config(db_type: &str, url: &str) -> Result<Self> {
        match db_type.to_lowercase().as_str() {
            "postgres" | "postgresql" | "" => Self::new_postgres(url).await,
            other => Err(anyhow::anyhow!(
                "Unsupported database type: {other}. Only PostgreSQL is supported."
            )),
        }
    }

    pub fn get_postgres_pool_arc(&self) -> Result<Arc<sqlx::PgPool>> {
        self.provider
            .get_postgres_pool()
            .ok_or_else(|| anyhow::anyhow!("Database is not PostgreSQL"))
    }

    pub async fn query(&self, sql: &dyn crate::models::QuerySelector) -> Result<QueryResult> {
        self.provider.query_raw(sql).await
    }

    pub async fn query_with(
        &self,
        sql: &dyn crate::models::QuerySelector,
        params: Vec<serde_json::Value>,
    ) -> Result<QueryResult> {
        self.provider.query_raw_with(sql, params).await
    }

    pub async fn execute_batch(&self, sql: &str) -> Result<()> {
        self.provider.execute_batch(sql).await
    }

    pub async fn get_info(&self) -> Result<DatabaseInfo> {
        self.provider.get_database_info().await
    }

    pub async fn test_connection(&self) -> Result<()> {
        self.provider.test_connection().await
    }

    #[must_use]
    pub fn get_postgres_pool(&self) -> Option<Arc<sqlx::PgPool>> {
        self.provider.get_postgres_pool()
    }

    pub fn pool_arc(&self) -> Result<Arc<sqlx::PgPool>> {
        self.get_postgres_pool_arc()
    }

    #[must_use]
    pub fn pool(&self) -> Option<Arc<sqlx::PgPool>> {
        self.get_postgres_pool()
    }

    pub async fn begin(&self) -> Result<sqlx::Transaction<'_, sqlx::Postgres>> {
        let pool = self.pool_arc()?;
        pool.begin().await.map_err(Into::into)
    }
}

pub type DbPool = Arc<Database>;

pub trait DatabaseExt {
    fn database(&self) -> Arc<Database>;
}

impl DatabaseExt for Arc<Database> {
    fn database(&self) -> Arc<Database> {
        Self::clone(self)
    }
}

#[async_trait::async_trait]
impl DatabaseProvider for Database {
    fn get_postgres_pool(&self) -> Option<Arc<sqlx::PgPool>> {
        self.provider.get_postgres_pool()
    }

    async fn execute(
        &self,
        query: &dyn crate::models::QuerySelector,
        params: &[&dyn crate::models::ToDbValue],
    ) -> Result<u64> {
        self.provider.execute(query, params).await
    }

    async fn execute_raw(&self, sql: &str) -> Result<()> {
        self.provider.execute_raw(sql).await
    }

    async fn fetch_all(
        &self,
        query: &dyn crate::models::QuerySelector,
        params: &[&dyn crate::models::ToDbValue],
    ) -> Result<Vec<crate::models::JsonRow>> {
        self.provider.fetch_all(query, params).await
    }

    async fn fetch_one(
        &self,
        query: &dyn crate::models::QuerySelector,
        params: &[&dyn crate::models::ToDbValue],
    ) -> Result<crate::models::JsonRow> {
        self.provider.fetch_one(query, params).await
    }

    async fn fetch_optional(
        &self,
        query: &dyn crate::models::QuerySelector,
        params: &[&dyn crate::models::ToDbValue],
    ) -> Result<Option<crate::models::JsonRow>> {
        self.provider.fetch_optional(query, params).await
    }

    async fn fetch_scalar_value(
        &self,
        query: &dyn crate::models::QuerySelector,
        params: &[&dyn crate::models::ToDbValue],
    ) -> Result<crate::models::DbValue> {
        self.provider.fetch_scalar_value(query, params).await
    }

    async fn begin_transaction(&self) -> Result<Box<dyn crate::models::DatabaseTransaction>> {
        self.provider.begin_transaction().await
    }

    async fn get_database_info(&self) -> Result<DatabaseInfo> {
        self.provider.get_database_info().await
    }

    async fn test_connection(&self) -> Result<()> {
        self.provider.test_connection().await
    }

    async fn execute_batch(&self, sql: &str) -> Result<()> {
        self.provider.execute_batch(sql).await
    }

    async fn query_raw(&self, query: &dyn crate::models::QuerySelector) -> Result<QueryResult> {
        self.provider.query_raw(query).await
    }

    async fn query_raw_with(
        &self,
        query: &dyn crate::models::QuerySelector,
        params: Vec<serde_json::Value>,
    ) -> Result<QueryResult> {
        self.provider.query_raw_with(query, params).await
    }
}
