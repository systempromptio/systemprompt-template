//! Identifier resolution to a `session_id`.

use sqlx::PgPool;

/// Resolve `id` (`decision_id`, `request_id`, `trace_id`, or `session_id`) to a
/// `session_id`.
pub(super) async fn resolve_session_id(
    pool: &PgPool,
    id: &str,
) -> Result<Option<String>, sqlx::Error> {
    if let Some(row) = sqlx::query!(
        r#"SELECT session_id as "session_id!" FROM governance_decisions WHERE id = $1 LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(row.session_id));
    }

    if let Some(row) = sqlx::query!(
        r"SELECT session_id FROM ai_requests
          WHERE id = $1 OR request_id = $1 OR trace_id = $1
          LIMIT 1",
        id,
    )
    .fetch_optional(pool)
    .await?
        && let Some(sid) = row.session_id
    {
        return Ok(Some(sid));
    }

    if let Some(row) = sqlx::query!(
        r#"SELECT session_id as "session_id!" FROM plugin_usage_events WHERE session_id = $1 LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(row.session_id));
    }

    Ok(None)
}
