use anyhow::{anyhow, Result};

use super::postgres::PostgresProvider;
use super::postgres_helpers::bind_params;
use super::provider::DatabaseProviderExt;
use crate::models::{FromDatabaseRow, QuerySelector, ToDbValue};

impl DatabaseProviderExt for PostgresProvider {
    async fn fetch_typed_optional<T: FromDatabaseRow>(
        &self,
        query: &dyn QuerySelector,
        params: &[&dyn ToDbValue],
    ) -> Result<Option<T>> {
        let sql = query.select_query();
        let query_obj = sqlx::query(sql);
        let query_obj = bind_params(query_obj, params);

        let row = query_obj
            .fetch_optional(self.pool())
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
        let query_obj = bind_params(query_obj, params);

        let row = query_obj
            .fetch_one(self.pool())
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
        let query_obj = bind_params(query_obj, params);

        let rows = query_obj
            .fetch_all(self.pool())
            .await
            .map_err(|e| anyhow!("Query execution failed: {e}"))?;

        rows.iter().map(|r| T::from_postgres_row(r)).collect()
    }
}
