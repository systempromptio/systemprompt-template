//! ID kind resolution for the global header search bar.
//!
//! Determines whether an opaque id refers to a request, trace, session, or
//! context, and returns the canonical id for the matching detail page.

use serde::Serialize;
use sqlx::PgPool;

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ResolvedKind {
    Request,
    Trace,
    Session,
    Context,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedId {
    pub kind: ResolvedKind,
    /// Canonical id for the detail page route.
    pub id: String,
}

/// Resolve `id` against the AI request, governance, and context tables.
///
/// Lookup order — most specific first:
///   1. `ai_requests.id` / `ai_requests.request_id`           → Request
///   2. `governance_decisions.id`                              → Request (oldest in chain)
///   3. `ai_requests.trace_id`                                 → Trace
///   4. `ai_requests.context_id` or `user_contexts.context_id` → Context
///   5. `ai_requests.session_id` / `governance_decisions.session_id` /
///      `user_contexts.session_id`                             → Session
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
        r#"SELECT trace_id AS "trace_id!" FROM ai_requests
           WHERE trace_id = $1 LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(ResolvedId { kind: ResolvedKind::Trace, id: row.trace_id }));
    }

    if let Some(row) = sqlx::query!(
        r#"SELECT context_id AS "context_id!" FROM user_contexts
           WHERE context_id = $1 LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(ResolvedId { kind: ResolvedKind::Context, id: row.context_id }));
    }

    if let Some(row) = sqlx::query!(
        r#"SELECT context_id AS "context_id!" FROM ai_requests
           WHERE context_id = $1 LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(ResolvedId { kind: ResolvedKind::Context, id: row.context_id }));
    }

    if let Some(row) = sqlx::query!(
        r#"SELECT session_id AS "session_id!" FROM ai_requests
           WHERE session_id = $1 LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(ResolvedId { kind: ResolvedKind::Session, id: row.session_id }));
    }

    if let Some(row) = sqlx::query!(
        r#"SELECT session_id AS "session_id!" FROM governance_decisions
           WHERE session_id = $1 LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(ResolvedId { kind: ResolvedKind::Session, id: row.session_id }));
    }

    if let Some(row) = sqlx::query!(
        r#"SELECT session_id AS "session_id!" FROM user_contexts
           WHERE session_id = $1 LIMIT 1"#,
        id,
    )
    .fetch_optional(pool)
    .await?
    {
        return Ok(Some(ResolvedId { kind: ResolvedKind::Session, id: row.session_id }));
    }

    Ok(None)
}
