//! Identifier resolution to a `session_id`.

use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

pub(super) async fn resolve_session_id(
    pool: &PgPool,
    id: &str,
) -> Result<Option<SessionId>, sqlx::Error> {
    if let Some(row) = sqlx::query!(
        r#"SELECT session_id as "session_id!: SessionId" FROM governance_decisions WHERE id = $1 LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(row.session_id));
    }

    #[cfg(feature = "gateway")]
    if let Some(row) = sqlx::query!(
        r#"SELECT session_id as "session_id: SessionId" FROM ai_requests
          WHERE id = $1 OR request_id = $1 OR trace_id = $1
          LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
        && let Some(sid) = row.session_id
    {
        return Ok(Some(sid));
    }

    if let Some(row) = sqlx::query!(
        r#"SELECT session_id as "session_id!: SessionId" FROM plugin_usage_events WHERE session_id = $1 LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(row.session_id));
    }

    // Why: a session id is only reachable through `plugin_usage_events` above,
    // and the hook spine is empty for gateway-only runs — so a session that has
    // decisions and requests but no plugin events resolved to nothing at all.
    if let Some(row) = sqlx::query!(
        r#"SELECT session_id as "session_id!: SessionId" FROM governance_decisions
           WHERE session_id = $1 AND session_id <> '' LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(row.session_id));
    }

    #[cfg(feature = "gateway")]
    if let Some(row) = sqlx::query!(
        r#"SELECT session_id as "session_id: SessionId" FROM ai_requests
           WHERE session_id = $1 LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
        && let Some(sid) = row.session_id
    {
        return Ok(Some(sid));
    }

    Ok(None)
}
