use crate::models::TaskRow;
use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_core_database::DbPool;
use systemprompt_traits::RepositoryError;

use crate::models::a2a::Task;
use crate::repository::task_constructor::TaskConstructor;

pub async fn get_task(
    pool: &Arc<PgPool>,
    db_pool: &DbPool,
    task_id: &str,
) -> Result<Option<Task>, RepositoryError> {
    let row = sqlx::query_as!(
        TaskRow,
        r#"SELECT
            task_id as "task_id!",
            context_id as "context_id!",
            status as "status!",
            status_timestamp,
            user_id,
            session_id,
            trace_id,
            agent_name,
            started_at,
            completed_at,
            execution_time_ms,
            metadata,
            created_at as "created_at!",
            updated_at as "updated_at!"
        FROM agent_tasks WHERE task_id = $1"#,
        task_id
    )
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    let Some(_row) = row else {
        return Ok(None);
    };

    let constructor = TaskConstructor::new(db_pool.clone());
    let task = constructor.construct_task_from_task_id(task_id).await?;

    Ok(Some(task))
}

pub async fn list_tasks_by_context(
    pool: &Arc<PgPool>,
    db_pool: &DbPool,
    context_id: &str,
) -> Result<Vec<Task>, RepositoryError> {
    let rows = sqlx::query_as!(
        TaskRow,
        r#"SELECT
            task_id as "task_id!",
            context_id as "context_id!",
            status as "status!",
            status_timestamp,
            user_id,
            session_id,
            trace_id,
            agent_name,
            started_at,
            completed_at,
            execution_time_ms,
            metadata,
            created_at as "created_at!",
            updated_at as "updated_at!"
        FROM agent_tasks WHERE context_id = $1 ORDER BY created_at ASC"#,
        context_id
    )
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    let constructor = TaskConstructor::new(db_pool.clone());
    let mut tasks = Vec::new();

    for row in rows {
        tasks.push(
            constructor
                .construct_task_from_task_id(&row.task_id)
                .await?,
        );
    }

    Ok(tasks)
}

pub async fn get_tasks_by_user_id(
    pool: &Arc<PgPool>,
    db_pool: &DbPool,
    user_id: &str,
    limit: Option<i32>,
    offset: Option<i32>,
) -> Result<Vec<Task>, RepositoryError> {
    let lim = limit.map(i64::from).unwrap_or(1000);
    let off = offset.map(i64::from).unwrap_or(0);

    let rows = sqlx::query_as!(
        TaskRow,
        r#"SELECT
            task_id as "task_id!",
            context_id as "context_id!",
            status as "status!",
            status_timestamp,
            user_id,
            session_id,
            trace_id,
            agent_name,
            started_at,
            completed_at,
            execution_time_ms,
            metadata,
            created_at as "created_at!",
            updated_at as "updated_at!"
        FROM agent_tasks WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
        user_id,
        lim,
        off
    )
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    let constructor = TaskConstructor::new(db_pool.clone());
    let mut tasks = Vec::new();

    for row in &rows {
        tasks.push(
            constructor
                .construct_task_from_task_id(&row.task_id)
                .await?,
        );
    }

    Ok(tasks)
}

#[derive(Debug, Clone)]
pub struct TaskContextInfo {
    pub context_id: String,
    pub user_id: String,
}

impl TaskContextInfo {
    pub fn context_id(&self) -> systemprompt_identifiers::ContextId {
        systemprompt_identifiers::ContextId::new(&self.context_id)
    }

    pub fn user_id(&self) -> systemprompt_identifiers::UserId {
        systemprompt_identifiers::UserId::new(&self.user_id)
    }
}

pub async fn get_task_context_info(
    pool: &Arc<PgPool>,
    task_id: &str,
) -> Result<Option<TaskContextInfo>, RepositoryError> {
    let row = sqlx::query!(
        r#"SELECT
            context_id as "context_id!",
            user_id
        FROM agent_tasks WHERE task_id = $1"#,
        task_id
    )
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    Ok(row.map(|r| TaskContextInfo {
        context_id: r.context_id,
        user_id: r.user_id.unwrap_or_default(),
    }))
}
