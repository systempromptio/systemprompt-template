use sqlx::PgPool;
use std::sync::Arc;
use systemprompt_traits::RepositoryError;

use crate::models::a2a::Message;

use super::parts::get_message_parts;

pub async fn get_messages_by_task(
    pool: &Arc<PgPool>,
    task_id: &str,
) -> Result<Vec<Message>, RepositoryError> {
    let message_rows: Vec<crate::models::TaskMessage> = sqlx::query_as!(
        crate::models::TaskMessage,
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

    let mut messages = Vec::new();

    for row in message_rows {
        let parts = get_message_parts(pool, &row.message_id).await?;

        let reference_task_ids = row.reference_task_ids.map(|ids| {
            ids.into_iter()
                .map(systemprompt_identifiers::TaskId::new)
                .collect()
        });

        messages.push(Message {
            role: row.role,
            message_id: row.message_id,
            task_id: Some(systemprompt_identifiers::TaskId::new(row.task_id)),
            context_id: systemprompt_identifiers::ContextId::new(
                row.context_id.unwrap_or_default(),
            ),
            kind: "message".to_string(),
            parts,
            metadata: row.metadata,
            extensions: None,
            reference_task_ids,
        });
    }

    Ok(messages)
}

pub async fn get_messages_by_context(
    pool: &Arc<PgPool>,
    context_id: &str,
) -> Result<Vec<Message>, RepositoryError> {
    let message_rows: Vec<crate::models::TaskMessage> = sqlx::query_as!(
        crate::models::TaskMessage,
        r#"SELECT
            m.id as "id!",
            m.task_id as "task_id!",
            m.message_id as "message_id!",
            m.client_message_id,
            m.role as "role!",
            m.context_id,
            m.user_id,
            m.session_id,
            m.trace_id,
            m.sequence_number as "sequence_number!",
            m.created_at as "created_at!",
            m.updated_at as "updated_at!",
            m.metadata,
            m.reference_task_ids
        FROM task_messages m
        JOIN agent_tasks t ON m.task_id = t.task_id
        WHERE t.context_id = $1
        ORDER BY m.created_at ASC"#,
        context_id
    )
    .fetch_all(pool.as_ref())
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    let mut messages = Vec::new();

    for row in message_rows {
        let parts = get_message_parts(pool, &row.message_id).await?;

        messages.push(Message {
            role: row.role,
            message_id: row.message_id,
            task_id: Some(systemprompt_identifiers::TaskId::new(row.task_id)),
            context_id: systemprompt_identifiers::ContextId::new(
                row.context_id.unwrap_or_else(|| context_id.to_string()),
            ),
            kind: "message".to_string(),
            parts,
            metadata: row.metadata,
            extensions: None,
            reference_task_ids: None,
        });
    }

    Ok(messages)
}

pub async fn get_next_sequence_number(
    pool: &Arc<PgPool>,
    task_id: &str,
) -> Result<i32, RepositoryError> {
    let row = sqlx::query!(
        r#"SELECT MAX(sequence_number) as "max_seq" FROM task_messages WHERE task_id = $1"#,
        task_id
    )
    .fetch_optional(pool.as_ref())
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    Ok(row.and_then(|r| r.max_seq).map(|s| s + 1).unwrap_or(0))
}

pub async fn get_next_sequence_number_sqlx(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    task_id: &str,
) -> Result<i32, RepositoryError> {
    let row = sqlx::query!(
        r#"SELECT MAX(sequence_number) as "max_seq" FROM task_messages WHERE task_id = $1"#,
        task_id
    )
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| RepositoryError::Database(e.to_string()))?;

    Ok(row.and_then(|r| r.max_seq).map(|s| s + 1).unwrap_or(0))
}

pub async fn get_next_sequence_number_in_tx(
    tx: &mut dyn systemprompt_core_database::DatabaseTransaction,
    task_id: &str,
) -> Result<i32, RepositoryError> {
    let query: &str =
        "SELECT MAX(sequence_number) as max_seq FROM task_messages WHERE task_id = $1";
    let row = tx.fetch_optional(&query, &[&task_id]).await?;

    let max_seq = if let Some(ref r) = row {
        r.get("max_seq").and_then(|v| v.as_i64()).map(|v| v as i32)
    } else {
        None
    };

    Ok(max_seq.map(|s| s + 1).unwrap_or(0))
}
