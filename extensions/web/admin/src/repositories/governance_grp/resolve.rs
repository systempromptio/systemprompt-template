//! ID kind resolution for the global header search bar.
//!
//! Determines whether an opaque id refers to a request (open audit detail) or
//! a trace/session (open trace waterfall), and returns the canonical id to
//! route to.

use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ResolvedKind {
    Request,
    Trace,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedId {
    pub kind: ResolvedKind,
    /// Canonical id to route to: `ai_requests.id` for `Request`, `session_id` for `Trace`.
    pub id: String,
}

/// Resolve `id` against governance and request tables.
///
/// Order: an exact `ai_requests.id` / `ai_requests.request_id` hit, or a
/// `governance_decisions.id` hit (which we map to its session's primary
/// request) returns `Request`; a `trace_id` or bare `session_id` returns
/// `Trace`.
pub async fn resolve_id(pool: &PgPool, id: &str) -> Result<Option<ResolvedId>, sqlx::Error> {
    if let Some(row) = sqlx::query!(
        r#"SELECT id AS "id!" FROM ai_requests
           WHERE id = $1 OR request_id = $1 LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(ResolvedId { kind: ResolvedKind::Request, id: row.id }));
    }

    if let Some(row) = sqlx::query!(
        r#"SELECT ar.id AS "ar_id!"
           FROM governance_decisions g
           JOIN ai_requests ar ON ar.session_id = g.session_id
           WHERE g.id = $1
           ORDER BY ar.created_at ASC
           LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(ResolvedId { kind: ResolvedKind::Request, id: row.ar_id }));
    }

    if let Some(row) = sqlx::query!(
        r#"SELECT session_id FROM ai_requests
           WHERE trace_id = $1 AND session_id IS NOT NULL LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        if let Some(sid) = row.session_id {
            return Ok(Some(ResolvedId { kind: ResolvedKind::Trace, id: sid }));
        }
    }

    if let Some(row) = sqlx::query!(
        r#"SELECT session_id AS "session_id!" FROM ai_requests
           WHERE session_id = $1 LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(ResolvedId { kind: ResolvedKind::Trace, id: row.session_id }));
    }

    if let Some(row) = sqlx::query!(
        r#"SELECT session_id AS "session_id!" FROM governance_decisions
           WHERE session_id = $1 LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(ResolvedId { kind: ResolvedKind::Trace, id: row.session_id }));
    }

    Ok(None)
}
