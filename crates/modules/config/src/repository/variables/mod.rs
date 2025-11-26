pub mod create;
pub mod delete;
pub mod get;
pub mod list;
pub mod update;

use std::sync::Arc;
use systemprompt_core_database::DatabaseProvider;
use systemprompt_core_system::DbPool;
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};

#[derive(Debug, Clone)]
pub struct VariablesRepository {
    db: Arc<dyn DatabaseProvider>,
    db_pool: DbPool,
}

impl RepositoryTrait for VariablesRepository {
    type Pool = DbPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db_pool
    }
}

impl VariablesRepository {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            db: db_pool.clone(),
            db_pool,
        }
    }
}
