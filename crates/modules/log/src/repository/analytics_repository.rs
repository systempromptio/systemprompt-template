use anyhow::Context;
use serde_json::Value;
use sqlx::query;
use systemprompt_core_database::DbPool;
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

    pub async fn log_event(&self, event: &AnalyticsEvent) -> anyhow::Result<i64> {
        let metadata_str = event.metadata.to_string();

        let user_id_str = event.user_id.as_str();
        let session_id_str = event.session_id.as_str();
        let context_id_str = event.context_id.as_str();

        let agent_id_opt: Option<&str> = event.agent_id.as_ref().map(AgentId::as_str);
        let task_id_opt: Option<&str> = event.task_id.as_ref().map(TaskId::as_str);
        let endpoint_opt: Option<&str> = event.endpoint.as_deref();
        let message_opt: Option<&str> = event.message.as_deref();

        let pool = self
            .db_pool
            .pool_arc()
            .context("Failed to get database pool")?;

        let result = query(
            r"
            INSERT INTO analytics_events
            (user_id, session_id, context_id, event_type, event_category, severity,
             endpoint, error_code, response_time_ms, agent_id, task_id, message, metadata, timestamp)
            VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, CURRENT_TIMESTAMP)
            ",
        )
        .bind(user_id_str)
        .bind(session_id_str)
        .bind(context_id_str)
        .bind(&event.event_type)
        .bind(&event.event_category)
        .bind(&event.severity)
        .bind(endpoint_opt)
        .bind(event.error_code)
        .bind(event.response_time_ms)
        .bind(agent_id_opt)
        .bind(task_id_opt)
        .bind(message_opt)
        .bind(&metadata_str)
        .execute(pool.as_ref())
        .await
        .context("Failed to log analytics event")?;

        Ok(i64::try_from(result.rows_affected()).unwrap_or(i64::MAX))
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
