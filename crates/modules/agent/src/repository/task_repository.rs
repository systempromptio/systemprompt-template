use crate::models::a2a::{Message, Part, Task, TaskState};
use crate::repository::message::{
    get_message_parts, get_messages_by_context, get_messages_by_task, get_next_sequence_number,
    get_next_sequence_number_in_tx, get_next_sequence_number_sqlx, persist_message_sqlx,
    persist_message_with_tx,
};
use crate::repository::task::{
    create_task, get_task, get_task_context_info, get_tasks_by_user_id, list_tasks_by_context,
    task_state_to_db_string, track_agent_in_context, update_task_state, TaskContextInfo,
};
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_core_system::repository::AnalyticsSessionRepository;
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};

pub use crate::repository::task::TaskContextInfo as TaskContextInfoExport;

#[derive(Debug, Clone)]
pub struct TaskRepository {
    db_pool: DbPool,
    analytics_session_repo: AnalyticsSessionRepository,
}

impl TaskRepository {
    #[must_use]
    pub fn new(db_pool: DbPool) -> Self {
        let analytics_session_repo = AnalyticsSessionRepository::new(db_pool.clone());
        Self {
            db_pool,
            analytics_session_repo,
        }
    }

    fn get_pg_pool(&self) -> Result<Arc<PgPool>, RepositoryError> {
        self.db_pool
            .as_ref()
            .get_postgres_pool()
            .ok_or_else(|| RepositoryError::Database("PostgreSQL pool not available".to_string()))
    }

    pub async fn create_task(
        &self,
        task: &Task,
        user_id: &systemprompt_identifiers::UserId,
        session_id: &systemprompt_identifiers::SessionId,
        trace_id: &systemprompt_identifiers::TraceId,
        agent_name: &str,
    ) -> Result<String, RepositoryError> {
        let pool = self.get_pg_pool()?;
        let result = create_task(&pool, task, user_id, session_id, trace_id, agent_name).await?;

        self.analytics_session_repo
            .increment_task_count(session_id.as_ref())
            .await
            .ok();

        Ok(result)
    }

    pub async fn get_task(&self, task_id: &str) -> Result<Option<Task>, RepositoryError> {
        let pool = self.get_pg_pool()?;
        get_task(&pool, &self.db_pool, task_id).await
    }

    pub async fn list_tasks_by_context(
        &self,
        context_id: &str,
    ) -> Result<Vec<Task>, RepositoryError> {
        let pool = self.get_pg_pool()?;
        list_tasks_by_context(&pool, &self.db_pool, context_id).await
    }

    pub async fn get_tasks_by_user_id(
        &self,
        user_id: &str,
        limit: Option<i32>,
        offset: Option<i32>,
    ) -> Result<Vec<Task>, RepositoryError> {
        let pool = self.get_pg_pool()?;
        get_tasks_by_user_id(&pool, &self.db_pool, user_id, limit, offset).await
    }

    pub async fn track_agent_in_context(
        &self,
        context_id: &str,
        agent_name: &str,
    ) -> Result<(), RepositoryError> {
        let pool = self.get_pg_pool()?;
        track_agent_in_context(&pool, context_id, agent_name).await
    }

    pub async fn update_task_state(
        &self,
        task_id: &str,
        state: TaskState,
        timestamp: &chrono::DateTime<chrono::Utc>,
    ) -> Result<(), RepositoryError> {
        let pool = self.get_pg_pool()?;
        update_task_state(&pool, task_id, state, timestamp).await
    }

    pub async fn update_task_and_save_messages(
        &self,
        task: &Task,
        user_message: &Message,
        agent_message: &Message,
        user_id: Option<&str>,
        session_id: &str,
        trace_id: &str,
    ) -> Result<Task, RepositoryError> {
        let pool = self.get_pg_pool()?;
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        let status = task_state_to_db_string(task.status.state.clone());
        let metadata_json = task
            .metadata
            .as_ref()
            .map(|m| serde_json::to_value(m).unwrap_or_default())
            .unwrap_or_else(|| serde_json::json!({}));

        let task_id_str = task.id.as_str();
        let is_completed = task.status.state == TaskState::Completed;

        let result = if is_completed {
            sqlx::query!(
                r#"UPDATE agent_tasks SET
                    status = $1,
                    status_timestamp = $2,
                    metadata = $3,
                    updated_at = CURRENT_TIMESTAMP,
                    completed_at = CURRENT_TIMESTAMP,
                    started_at = COALESCE(started_at, CURRENT_TIMESTAMP),
                    execution_time_ms = EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - COALESCE(started_at, CURRENT_TIMESTAMP))) * 1000
                WHERE task_id = $4"#,
                status,
                task.status.timestamp,
                metadata_json,
                task_id_str
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?
        } else {
            sqlx::query!(
                r#"UPDATE agent_tasks SET status = $1, status_timestamp = $2, metadata = $3, updated_at = CURRENT_TIMESTAMP WHERE task_id = $4"#,
                status,
                task.status.timestamp,
                metadata_json,
                task_id_str
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?
        };

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Task not found for update: {}",
                task.id
            )));
        }

        let user_seq = get_next_sequence_number_sqlx(&mut tx, task.id.as_str()).await?;
        persist_message_sqlx(
            &mut tx,
            user_message,
            task.id.as_str(),
            task.context_id.as_str(),
            user_seq,
            user_id,
            session_id,
            trace_id,
        )
        .await?;

        let agent_seq = get_next_sequence_number_sqlx(&mut tx, task.id.as_str()).await?;
        persist_message_sqlx(
            &mut tx,
            agent_message,
            task.id.as_str(),
            task.context_id.as_str(),
            agent_seq,
            user_id,
            session_id,
            trace_id,
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        self.analytics_session_repo
            .increment_message_count(session_id)
            .await
            .ok();
        self.analytics_session_repo
            .increment_message_count(session_id)
            .await
            .ok();

        let updated_task = self.get_task(task.id.as_str()).await?.ok_or_else(|| {
            RepositoryError::NotFound(format!("Task not found after update: {}", task.id))
        })?;

        Ok(updated_task)
    }

    pub async fn get_next_sequence_number(&self, task_id: &str) -> Result<i32, RepositoryError> {
        let pool = self.get_pg_pool()?;
        get_next_sequence_number(&pool, task_id).await
    }

    pub async fn get_messages_by_task(
        &self,
        task_id: &str,
    ) -> Result<Vec<Message>, RepositoryError> {
        let pool = self.get_pg_pool()?;
        get_messages_by_task(&pool, task_id).await
    }

    pub async fn get_message_parts(&self, message_id: &str) -> Result<Vec<Part>, RepositoryError> {
        let pool = self.get_pg_pool()?;
        get_message_parts(&pool, message_id).await
    }

    pub async fn get_messages_by_context(
        &self,
        context_id: &str,
    ) -> Result<Vec<Message>, RepositoryError> {
        let pool = self.get_pg_pool()?;
        get_messages_by_context(&pool, context_id).await
    }

    pub async fn get_next_sequence_number_in_tx(
        &self,
        tx: &mut dyn systemprompt_core_database::DatabaseTransaction,
        task_id: &str,
    ) -> Result<i32, RepositoryError> {
        get_next_sequence_number_in_tx(tx, task_id).await
    }

    pub async fn persist_message_with_tx(
        &self,
        tx: &mut dyn systemprompt_core_database::DatabaseTransaction,
        message: &Message,
        task_id: &str,
        context_id: &str,
        sequence_number: i32,
        user_id: Option<&str>,
        session_id: &str,
        trace_id: &str,
    ) -> Result<(), RepositoryError> {
        persist_message_with_tx(
            tx,
            message,
            task_id,
            context_id,
            sequence_number,
            user_id,
            session_id,
            trace_id,
        )
        .await
    }

    pub async fn get_task_context_info(
        &self,
        task_id: &str,
    ) -> Result<Option<TaskContextInfo>, RepositoryError> {
        let pool = self.get_pg_pool()?;
        get_task_context_info(&pool, task_id).await
    }
}

impl RepositoryTrait for TaskRepository {
    type Pool = DbPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db_pool
    }
}
