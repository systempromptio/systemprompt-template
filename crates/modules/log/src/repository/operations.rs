#![allow(clippy::print_stdout)]

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use systemprompt_core_database::DbPool;
use systemprompt_identifiers::{ClientId, ContextId, SessionId, TaskId, TraceId, UserId};

use crate::models::{LogEntry, LogLevel, LoggingError};

pub async fn create_log(db_pool: &DbPool, entry: &LogEntry) -> Result<(), LoggingError> {
    let metadata_json = entry
        .metadata
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .context("Failed to serialize log metadata")?;

    let level_str = entry.level.to_string();
    let pool = db_pool.pool_arc().context("Failed to get database pool")?;

    let user_id = entry.user_id.as_str();
    let session_id = entry.session_id.as_str();
    let task_id = entry.task_id.as_ref().map(TaskId::as_str);
    let trace_id = entry.trace_id.as_str();
    let context_id = entry.context_id.as_ref().map(ContextId::as_str);
    let client_id = entry.client_id.as_ref().map(ClientId::as_str);

    sqlx::query!(
        r"
        INSERT INTO logs (id, timestamp, level, module, message, metadata, user_id, session_id, task_id, trace_id, context_id, client_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        ",
        entry.id,
        entry.timestamp,
        level_str,
        entry.module,
        entry.message,
        metadata_json,
        user_id,
        session_id,
        task_id,
        trace_id,
        context_id,
        client_id
    )
    .execute(pool.as_ref())
    .await
    .context("Failed to create log entry")?;

    Ok(())
}

pub async fn get_log(db_pool: &DbPool, id: &str) -> Result<Option<LogEntry>, LoggingError> {
    let pool = db_pool.pool_arc().context("Failed to get database pool")?;

    let row = sqlx::query!(
        r#"
        SELECT
            id as "id!",
            timestamp as "timestamp!",
            level as "level!",
            module as "module!",
            message as "message!",
            metadata,
            user_id as "user_id!",
            session_id as "session_id!",
            task_id,
            trace_id as "trace_id!",
            context_id,
            client_id
        FROM logs
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool.as_ref())
    .await
    .context("Failed to get log by id")?;

    Ok(row.map(|r| {
        let metadata = r
            .metadata
            .as_ref()
            .and_then(|m| serde_json::from_str(m).ok());

        LogEntry {
            id: r.id.clone(),
            timestamp: r.timestamp,
            level: r.level.parse().unwrap_or(LogLevel::Info),
            module: r.module.clone(),
            message: r.message.clone(),
            metadata,
            user_id: UserId::new(r.user_id.clone()),
            session_id: SessionId::new(r.session_id.clone()),
            task_id: r.task_id.clone().map(TaskId::new),
            trace_id: TraceId::new(r.trace_id.clone()),
            context_id: r.context_id.clone().map(ContextId::new),
            client_id: r.client_id.clone().map(ClientId::new),
        }
    }))
}

pub async fn list_logs(db_pool: &DbPool, limit: i64) -> Result<Vec<LogEntry>, LoggingError> {
    let pool = db_pool.pool_arc().context("Failed to get database pool")?;

    let rows = sqlx::query!(
        r#"
        SELECT
            id as "id!",
            timestamp as "timestamp!",
            level as "level!",
            module as "module!",
            message as "message!",
            metadata,
            user_id as "user_id!",
            session_id as "session_id!",
            task_id,
            trace_id as "trace_id!",
            context_id,
            client_id
        FROM logs
        ORDER BY timestamp DESC
        LIMIT $1
        "#,
        limit
    )
    .fetch_all(pool.as_ref())
    .await
    .context("Failed to list logs")?;

    Ok(rows
        .into_iter()
        .map(|r| {
            let metadata = r
                .metadata
                .as_ref()
                .and_then(|m| serde_json::from_str(m).ok());

            LogEntry {
                id: r.id.clone(),
                timestamp: r.timestamp,
                level: r.level.parse().unwrap_or(LogLevel::Info),
                module: r.module.clone(),
                message: r.message.clone(),
                metadata,
                user_id: UserId::new(r.user_id.clone()),
                session_id: SessionId::new(r.session_id.clone()),
                task_id: r.task_id.clone().map(TaskId::new),
                trace_id: TraceId::new(r.trace_id.clone()),
                context_id: r.context_id.clone().map(ContextId::new),
                client_id: r.client_id.clone().map(ClientId::new),
            }
        })
        .collect())
}

pub async fn list_logs_paginated(
    db_pool: &DbPool,
    page: i32,
    per_page: i32,
    level_filter: Option<&str>,
    module_filter: Option<&str>,
    message_filter: Option<&str>,
) -> Result<(Vec<LogEntry>, i64), LoggingError> {
    let pool = db_pool.pool_arc().context("Failed to get database pool")?;
    let offset = i64::from(page.saturating_sub(1).saturating_mul(per_page));
    let per_page = i64::from(per_page);
    let message_pattern = message_filter.map(|m| format!("%{m}%"));

    let rows = sqlx::query!(
        r#"
        SELECT
            id as "id!",
            timestamp as "timestamp!",
            level as "level!",
            module as "module!",
            message as "message!",
            metadata,
            user_id as "user_id!",
            session_id as "session_id!",
            task_id,
            trace_id as "trace_id!",
            context_id,
            client_id
        FROM logs
        WHERE ($1::VARCHAR IS NULL OR level = $1)
        AND ($2::VARCHAR IS NULL OR module = $2)
        AND ($3::VARCHAR IS NULL OR message LIKE $3)
        ORDER BY timestamp DESC
        LIMIT $4 OFFSET $5
        "#,
        level_filter,
        module_filter,
        message_pattern,
        per_page,
        offset
    )
    .fetch_all(pool.as_ref())
    .await
    .context("Failed to get paginated logs")?;

    let count_row = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*) as "count!" FROM logs
        WHERE ($1::VARCHAR IS NULL OR level = $1)
        AND ($2::VARCHAR IS NULL OR module = $2)
        AND ($3::VARCHAR IS NULL OR message LIKE $3)
        "#,
        level_filter,
        module_filter,
        message_pattern
    )
    .fetch_one(pool.as_ref())
    .await
    .context("Failed to count logs")?;

    let entries: Vec<LogEntry> = rows
        .into_iter()
        .map(|r| {
            let metadata = r
                .metadata
                .as_ref()
                .and_then(|m| serde_json::from_str(m).ok());

            LogEntry {
                id: r.id.clone(),
                timestamp: r.timestamp,
                level: r.level.parse().unwrap_or(LogLevel::Info),
                module: r.module.clone(),
                message: r.message.clone(),
                metadata,
                user_id: UserId::new(r.user_id.clone()),
                session_id: SessionId::new(r.session_id.clone()),
                task_id: r.task_id.clone().map(TaskId::new),
                trace_id: TraceId::new(r.trace_id.clone()),
                context_id: r.context_id.clone().map(ContextId::new),
                client_id: r.client_id.clone().map(ClientId::new),
            }
        })
        .collect();

    Ok((entries, count_row))
}

pub async fn update_log(
    db_pool: &DbPool,
    id: &str,
    entry: &LogEntry,
) -> Result<bool, LoggingError> {
    let metadata_json = entry
        .metadata
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .context("Failed to serialize log metadata")?;

    let level_str = entry.level.to_string();
    let pool = db_pool.pool_arc().context("Failed to get database pool")?;

    let result = sqlx::query!(
        r"
        UPDATE logs
        SET level = $1, module = $2, message = $3, metadata = $4
        WHERE id = $5
        ",
        level_str,
        entry.module,
        entry.message,
        metadata_json,
        id
    )
    .execute(pool.as_ref())
    .await
    .context("Failed to update log entry")?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_log(db_pool: &DbPool, id: &str) -> Result<bool, LoggingError> {
    let pool = db_pool.pool_arc().context("Failed to get database pool")?;

    let result = sqlx::query!("DELETE FROM logs WHERE id = $1", id)
        .execute(pool.as_ref())
        .await
        .context("Failed to delete log entry")?;

    Ok(result.rows_affected() > 0)
}

pub async fn delete_logs_multiple(db_pool: &DbPool, ids: &[String]) -> Result<u64, LoggingError> {
    if ids.is_empty() {
        return Ok(0);
    }

    let pool = db_pool.pool_arc().context("Failed to get database pool")?;

    let result = sqlx::query!("DELETE FROM logs WHERE id = ANY($1)", ids)
        .execute(pool.as_ref())
        .await
        .context("Failed to delete multiple log entries")?;

    Ok(result.rows_affected())
}

pub async fn clear_all_logs(db_pool: &DbPool) -> Result<u64, LoggingError> {
    let pool = db_pool.pool_arc().context("Failed to get database pool")?;

    let result = sqlx::query!("DELETE FROM logs")
        .execute(pool.as_ref())
        .await
        .context("Failed to clear all logs")?;

    Ok(result.rows_affected())
}

pub async fn cleanup_logs_before(
    db_pool: &DbPool,
    cutoff: DateTime<Utc>,
) -> Result<u64, LoggingError> {
    let pool = db_pool.pool_arc().context("Failed to get database pool")?;

    let result = sqlx::query!("DELETE FROM logs WHERE timestamp < $1", cutoff)
        .execute(pool.as_ref())
        .await
        .context("Failed to cleanup old logs")?;

    Ok(result.rows_affected())
}
