use crate::models::a2a::{Artifact, Message, Part, Task, TaskStatus};
use crate::models::{MessagePart, TaskMessage, TaskRow};
use crate::repository::ExecutionStepRepository;
use systemprompt_models::ExecutionStep;
use systemprompt_traits::RepositoryError;

use super::{converters, TaskConstructor};

pub async fn construct_task_from_task_id(
    constructor: &TaskConstructor,
    task_id: &str,
) -> Result<Task, RepositoryError> {
    let row = fetch_task_row(constructor, task_id).await?;
    construct_task_from_row(constructor, &row).await
}

async fn fetch_task_row(
    constructor: &TaskConstructor,
    task_id: &str,
) -> Result<TaskRow, RepositoryError> {
    let pool = constructor.get_pg_pool()?;

    sqlx::query_as!(
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
    .fetch_one(pool.as_ref())
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            RepositoryError::NotFound(format!("Task {} not found", task_id))
        },
        _ => RepositoryError::Database(e.to_string()),
    })
}

async fn construct_task_from_row(
    constructor: &TaskConstructor,
    row: &TaskRow,
) -> Result<Task, RepositoryError> {
    let task_id = &row.task_id;

    let history = load_task_messages(constructor, task_id).await?;
    let artifacts = load_task_artifacts(constructor, task_id).await?;
    let execution_steps = load_execution_steps(constructor, task_id).await?;

    let mut metadata = converters::construct_metadata(row)?;
    if let Some(steps) = execution_steps {
        if let Some(ref mut meta) = metadata {
            meta.execution_steps = Some(steps);
        }
    }

    let task_state = converters::parse_task_state(&row.status)
        .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;

    Ok(Task {
        id: task_id.clone().into(),
        context_id: row.context_id.clone().into(),
        kind: "task".to_string(),
        status: TaskStatus {
            state: task_state,
            message: None,
            timestamp: row.status_timestamp,
        },
        history,
        artifacts,
        metadata,
    })
}

async fn load_task_messages(
    constructor: &TaskConstructor,
    task_id: &str,
) -> Result<Option<Vec<Message>>, RepositoryError> {
    let pool = constructor.get_pg_pool()?;

    let message_rows: Vec<TaskMessage> = sqlx::query_as!(
        TaskMessage,
        r#"SELECT
            id as "id!",
            task_id as "task_id!",
            message_id as "message_id!",
            client_message_id,
            role as "role!",
            context_id,
            user_id,
            session_id,
            trace_id,
            sequence_number as "sequence_number!",
            created_at as "created_at!",
            updated_at as "updated_at!",
            metadata,
            reference_task_ids
        FROM task_messages WHERE task_id = $1 ORDER BY sequence_number ASC"#,
        task_id
    )
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    if message_rows.is_empty() {
        return Ok(None);
    }

    let mut messages = Vec::new();
    for msg_row in message_rows {
        let parts = load_message_parts(constructor, &msg_row.message_id, task_id).await?;
        let message = build_message_from_row(msg_row, parts);
        messages.push(message);
    }

    Ok(Some(messages))
}

async fn load_message_parts(
    constructor: &TaskConstructor,
    message_id: &str,
    task_id: &str,
) -> Result<Vec<Part>, RepositoryError> {
    let pool = constructor.get_pg_pool()?;

    let part_rows: Vec<MessagePart> = sqlx::query_as!(
        MessagePart,
        r#"SELECT
            id as "id!",
            message_id as "message_id!",
            task_id as "task_id!",
            part_kind as "part_kind!",
            sequence_number as "sequence_number!",
            text_content,
            file_name,
            file_mime_type,
            file_uri,
            file_bytes,
            data_content,
            metadata
        FROM message_parts WHERE message_id = $1 AND task_id = $2 ORDER BY sequence_number ASC"#,
        message_id,
        task_id
    )
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    converters::build_parts_from_rows(&part_rows)
}

async fn load_task_artifacts(
    constructor: &TaskConstructor,
    task_id: &str,
) -> Result<Option<Vec<Artifact>>, RepositoryError> {
    let artifacts = constructor
        .artifact_repo()
        .get_artifacts_by_task(task_id)
        .await
        .map_err(|e| RepositoryError::InvalidData(e.to_string()))?;

    if artifacts.is_empty() {
        Ok(None)
    } else {
        Ok(Some(artifacts))
    }
}

async fn load_execution_steps(
    constructor: &TaskConstructor,
    task_id: &str,
) -> Result<Option<Vec<ExecutionStep>>, RepositoryError> {
    let step_repo = ExecutionStepRepository::new(constructor.db_pool().clone());

    let steps = step_repo
        .list_by_task(task_id)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

    if steps.is_empty() {
        Ok(None)
    } else {
        Ok(Some(steps))
    }
}

fn build_message_from_row(msg_row: TaskMessage, parts: Vec<Part>) -> Message {
    let reference_task_ids = msg_row
        .reference_task_ids
        .map(|ids| ids.into_iter().map(|id| id.into()).collect());

    let mut final_metadata = msg_row.metadata.unwrap_or_else(|| serde_json::json!({}));
    if let Some(client_id) = &msg_row.client_message_id {
        if let Some(obj) = final_metadata.as_object_mut() {
            obj.insert(
                "clientMessageId".to_string(),
                serde_json::Value::String(client_id.clone()),
            );
        }
    }

    Message {
        role: msg_row.role,
        parts,
        message_id: msg_row.message_id,
        task_id: Some(msg_row.task_id.into()),
        context_id: msg_row.context_id.unwrap_or_default().into(),
        kind: "message".to_string(),
        metadata: if final_metadata == serde_json::json!({}) {
            None
        } else {
            Some(final_metadata)
        },
        extensions: None,
        reference_task_ids,
    }
}
