use anyhow::Result;
use serde_json::Value;
use systemprompt_core_database::{DatabaseProvider, DbPool};
use systemprompt_identifiers::{AgentId, ContextId, SessionId, TaskId, UserId};
use systemprompt_traits::{Repository as RepositoryTrait, RepositoryError};

#[derive(Debug, Clone)]
pub struct AnalyticsRepository {
    db_pool: DbPool,
}

impl RepositoryTrait for AnalyticsRepository {
    type Pool = DbPool;
    type Error = RepositoryError;

    fn pool(&self) -> &Self::Pool {
        &self.db_pool
    }
}

impl AnalyticsRepository {
    pub const fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub async fn log_event(&self, event: &AnalyticsEvent) -> Result<i64> {
        use systemprompt_core_database::DatabaseQueryEnum;

        let metadata_str = event.metadata.to_string();

        // Convert typed IDs to strings at the DB boundary
        let user_id_str = event.user_id.as_str();
        let session_id_str = event.session_id.as_str();
        let context_id_str = event.context_id.as_str();

        // Handle optional fields properly for NULL values
        let agent_id_opt: Option<&str> = event.agent_id.as_ref().map(AgentId::as_str);
        let task_id_opt: Option<&str> = event.task_id.as_ref().map(TaskId::as_str);
        let endpoint_opt: Option<&str> = event.endpoint.as_deref();
        let message_opt: Option<&str> = event.message.as_deref();

        let query = DatabaseQueryEnum::LogAnalyticsEvent.get(self.db_pool.as_ref());
        let rows_affected = self
            .db_pool
            .execute(
                &query,
                &[
                    &user_id_str,
                    &session_id_str,
                    &context_id_str,
                    &event.event_type.as_str(),
                    &event.event_category.as_str(),
                    &event.severity.as_str(),
                    &endpoint_opt,
                    &event.error_code,
                    &event.response_time_ms,
                    &agent_id_opt,
                    &task_id_opt,
                    &message_opt,
                    &metadata_str,
                ],
            )
            .await?;

        Ok(i64::try_from(rows_affected).unwrap_or(i64::MAX))
    }
}

#[derive(Debug, Clone)]
pub struct AnalyticsEvent {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub context_id: ContextId,
    pub event_type: String,
    pub event_category: String,
    pub severity: String,
    pub endpoint: Option<String>,
    pub error_code: Option<i32>,
    pub response_time_ms: Option<i32>,
    pub agent_id: Option<AgentId>,
    pub task_id: Option<TaskId>,
    pub message: Option<String>,
    pub metadata: Value,
}
