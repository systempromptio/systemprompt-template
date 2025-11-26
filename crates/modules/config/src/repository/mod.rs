pub mod modules;
pub mod variables;

pub use modules::ModuleRepository;
pub use systemprompt_models::repository::{McpServer, ServiceConfig, ServiceRepository};
pub use variables::VariablesRepository;

use anyhow::Context;
use std::sync::Arc;
use systemprompt_core_database::{DatabaseProvider, DatabaseQueryEnum};
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};

#[derive(Debug)]
pub struct ConfigRepository {
    db_pool: systemprompt_core_database::DbPool,
    db: Arc<dyn DatabaseProvider>,
}

impl RepositoryTrait for ConfigRepository {
    type Pool = systemprompt_core_database::DbPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db_pool
    }
}

impl ConfigRepository {
    pub fn new(db_pool: systemprompt_core_database::DbPool) -> Self {
        let db = db_pool.clone();
        Self { db_pool, db }
    }

    pub async fn is_config_table_available(&self) -> anyhow::Result<bool> {
        let query = DatabaseQueryEnum::CheckConfigTableExists.get(self.db.as_ref());
        let row = self
            .db
            .fetch_optional(&query, &[])
            .await
            .context("Failed to check if config table exists")?;

        Ok(row.is_some())
    }

    pub async fn list_configs(
        &self,
        _module_name: Option<&str>,
        _limit: Option<u32>,
        _offset: Option<u32>,
    ) -> anyhow::Result<Vec<ConfigRow>> {
        Ok(Vec::new())
    }
}

#[derive(Debug, Clone)]
pub struct ConfigRow {
    pub module_name: Option<String>,
}
