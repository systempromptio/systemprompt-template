use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_traits::RepositoryError;

use crate::models::a2a::{Task, TaskState};

pub const fn task_state_to_db_string(state: TaskState) -> &'static str {
    match state {
        TaskState::Pending => "submitted",
        TaskState::Submitted => "submitted",
        TaskState::Working => "working",
        TaskState::InputRequired => "input-required",
        TaskState::Completed => "completed",
        TaskState::Canceled => "canceled",
        TaskState::Failed => "failed",
        TaskState::Rejected => "rejected",
        TaskState::AuthRequired => "auth-required",
        TaskState::Unknown => "unknown",
    }
}

pub async fn create_task(
    pool: &Arc<PgPool>,
    task: &Task,
    user_id: &systemprompt_identifiers::UserId,
    session_id: &systemprompt_identifiers::SessionId,
    trace_id: &systemprompt_identifiers::TraceId,
    agent_name: &str,
) -> Result<String, RepositoryError> {
    let metadata_json = task
        .metadata
        .as_ref()
        .map(|m| serde_json::to_value(m).unwrap_or_default())
        .unwrap_or_else(|| serde_json::json!({}));

    let status = task_state_to_db_string(task.status.state.clone());
    let task_id_str = task.id.as_str();
    let context_id_str = task.context_id.as_str();
    let user_id_str = user_id.as_ref();
    let session_id_str = session_id.as_ref();
    let trace_id_str = trace_id.as_ref();

    sqlx::query!(
        r#"INSERT INTO agent_tasks (task_id, context_id, status, status_timestamp, user_id, session_id, trace_id, metadata, agent_name)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
        task_id_str,
        context_id_str,
        status,
        task.status.timestamp,
        user_id_str,
        session_id_str,
        trace_id_str,
        metadata_json,
        agent_name
    )
    .execute(pool.as_ref())
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    Ok(task.id.to_string())
}

pub async fn track_agent_in_context(
    pool: &Arc<PgPool>,
    context_id: &str,
    agent_name: &str,
) -> Result<(), RepositoryError> {
    sqlx::query!(
        r#"INSERT INTO context_agents (context_id, agent_name) VALUES ($1, $2)
        ON CONFLICT (context_id, agent_name) DO NOTHING"#,
        context_id,
        agent_name
    )
    .execute(pool.as_ref())
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    Ok(())
}

pub async fn update_task_state(
    pool: &Arc<PgPool>,
    task_id: &str,
    state: TaskState,
    timestamp: &chrono::DateTime<chrono::Utc>,
) -> Result<(), RepositoryError> {
    let status = task_state_to_db_string(state);

    if state == TaskState::Completed {
        sqlx::query!(
            r#"UPDATE agent_tasks SET status = $1, status_timestamp = $2, updated_at = CURRENT_TIMESTAMP,
            completed_at = CURRENT_TIMESTAMP,
            started_at = COALESCE(started_at, CURRENT_TIMESTAMP),
            execution_time_ms = EXTRACT(EPOCH FROM (CURRENT_TIMESTAMP - COALESCE(started_at, CURRENT_TIMESTAMP))) * 1000
            WHERE task_id = $3"#,
            status,
            timestamp,
            task_id
        )
        .execute(pool.as_ref())
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;
    } else if state == TaskState::Working {
        sqlx::query!(
            r#"UPDATE agent_tasks SET status = $1, status_timestamp = $2, updated_at = CURRENT_TIMESTAMP,
            started_at = COALESCE(started_at, CURRENT_TIMESTAMP)
            WHERE task_id = $3"#,
            status,
            timestamp,
            task_id
        )
        .execute(pool.as_ref())
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;
    } else {
        sqlx::query!(
            r#"UPDATE agent_tasks SET status = $1, status_timestamp = $2, updated_at = CURRENT_TIMESTAMP WHERE task_id = $3"#,
            status,
            timestamp,
            task_id
        )
        .execute(pool.as_ref())
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;
    }

    Ok(())
}
