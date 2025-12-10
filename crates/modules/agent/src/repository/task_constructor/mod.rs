mod batch;
mod batch_queries;
mod converters;
mod single;

use crate::models::a2a::Task;
use crate::repository::artifact_repository::ArtifactRepository;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_traits::RepositoryError;

#[derive(Debug, Clone)]
pub struct TaskConstructor {
    db_pool: DbPool,
    artifact_repo: ArtifactRepository,
}

impl TaskConstructor {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            artifact_repo: ArtifactRepository::new(db_pool.clone()),
            db_pool,
        }
    }

    pub(crate) fn get_pg_pool(&self) -> Result<Arc<PgPool>, RepositoryError> {
        self.db_pool
            .as_ref()
            .get_postgres_pool()
            .ok_or_else(|| RepositoryError::Database("PostgreSQL pool not available".to_string()))
    }

    pub(crate) const fn artifact_repo(&self) -> &ArtifactRepository {
        &self.artifact_repo
    }

    pub(crate) const fn db_pool(&self) -> &DbPool {
        &self.db_pool
    }

    pub async fn construct_task_from_task_id(
        &self,
        task_id: &str,
    ) -> Result<Task, RepositoryError> {
        single::construct_task_from_task_id(self, task_id).await
    }

    pub async fn construct_tasks_batch(
        &self,
        task_ids: &[String],
    ) -> Result<Vec<Task>, RepositoryError> {
        batch::construct_tasks_batch(self, task_ids).await
    }
}
