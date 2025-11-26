mod delete;
mod disable;
mod enable;
mod get_all;
mod insert;
mod update;

pub use delete::delete_module;
pub use disable::disable_module;
pub use enable::enable_module;
pub use get_all::{get_all_modules, DatabaseModule};
pub use insert::insert_module;
pub use update::update_module;

use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};

#[derive(Debug)]
pub struct ModuleRepository {
    db: Arc<dyn systemprompt_core_database::DatabaseProvider>,
    db_pool: DbPool,
}

impl RepositoryTrait for ModuleRepository {
    type Pool = DbPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db_pool
    }
}

impl ModuleRepository {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            db: db_pool.clone(),
            db_pool,
        }
    }

    pub async fn insert_module_config(
        &self,
        module: &systemprompt_core_system::Module,
    ) -> Result<()> {
        insert_module(&*self.db, module).await
    }

    pub async fn update_module_version(
        &self,
        module: &systemprompt_core_system::Module,
    ) -> Result<()> {
        update_module(&*self.db, module).await
    }

    pub async fn delete_module_config(&self, module_name: &str) -> Result<()> {
        delete_module(&*self.db, module_name).await
    }

    pub async fn enable_module_config(&self, module_name: &str) -> Result<()> {
        enable_module(&*self.db, module_name).await
    }

    pub async fn disable_module_config(&self, module_name: &str) -> Result<()> {
        disable_module(&*self.db, module_name).await
    }

    pub async fn get_all(&self) -> Result<Vec<DatabaseModule>> {
        get_all_modules(&*self.db).await
    }
}
