use crate::models::{DatabaseInfo, TableInfo};
use crate::services::Database;
use anyhow::Result;
use std::sync::Arc;
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};

#[derive(Debug)]
pub struct DatabaseInfoRepository {
    db: Arc<Database>,
}

impl RepositoryTrait for DatabaseInfoRepository {
    type Pool = Arc<Database>;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db
    }
}

impl DatabaseInfoRepository {
    pub const fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn get_database_info(&self) -> Result<DatabaseInfo> {
        self.db.get_info().await
    }

    pub async fn list_tables(&self) -> Result<Vec<TableInfo>> {
        let info = self.db.get_info().await?;
        Ok(info.tables)
    }

    /// Get information for a specific table including schema and row count
    pub async fn get_table_info(&self, table_name: &str) -> Result<Option<TableInfo>> {
        let info = self.db.get_info().await?;
        Ok(info.tables.into_iter().find(|t| t.name == table_name))
    }
}
