//! What a project's work went through and produced: the coding agents behind
//! its requests and the artifacts its tool calls left behind.
//!
//! Sibling of [`super::activity`]. Agents are `client_kind` on the request —
//! this instance ships no A2A agents, so the agents a project uses are the
//! Claude Code, Codex and desktop sessions that called the gateway. Both
//! halves read membership, the same attribution the projects listing uses.

use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::scope::ScopeQuery;

/// One coding agent as the project used it.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectAgentRow {
    pub client_kind: String,
    pub requests: i64,
    pub users: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub models: i64,
}

/// Artifacts of one type from one server, with the share that were errors.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectArtifactRow {
    pub server_name: String,
    pub artifact_type: String,
    pub artifacts: i64,
    pub errors: i64,
    pub users: i64,
    pub last_created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_project_agents(
    pool: &PgPool,
    q: &ScopeQuery<'_>,
) -> Result<Vec<ProjectAgentRow>, sqlx::Error> {
    let rows = crate::scoped_query!(
        r#"SELECT x.client_kind AS "client_kind!",
                  COUNT(x.id)::BIGINT AS "requests!",
                  COUNT(DISTINCT x.user_id)::BIGINT AS "users!",
                  COALESCE(SUM(COALESCE(x.tokens_used, COALESCE(x.input_tokens, 0) + COALESCE(x.output_tokens, 0) + COALESCE(x.cache_read_tokens, 0) + COALESCE(x.cache_creation_tokens, 0))), 0)::BIGINT AS "tokens!",
                  COALESCE(SUM(x.cost_microdollars), 0)::BIGINT AS "cost_microdollars!",
                  COUNT(DISTINCT x.model)::BIGINT AS "models!"
           FROM membership m
           JOIN ai_requests x ON x.user_id = m.user_id
            AND x.created_at >= NOW() - make_interval(days => $4)
           WHERE m.scope_id = $3
           GROUP BY x.client_kind
           ORDER BY COUNT(x.id) DESC, x.client_kind"#,
        q.kind().as_str(),
        q.attribution.is_exclusive(),
        q.id(),
        q.window_days
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ProjectAgentRow {
            client_kind: row.client_kind,
            requests: row.requests,
            users: row.users,
            tokens: row.tokens,
            cost_microdollars: row.cost_microdollars,
            models: row.models,
        })
        .collect())
}

pub async fn list_project_artifacts(
    pool: &PgPool,
    q: &ScopeQuery<'_>,
    limit: i64,
) -> Result<Vec<ProjectArtifactRow>, sqlx::Error> {
    let rows = crate::scoped_query!(
        r#"SELECT x.server_name AS "server_name!",
                  x.artifact_type AS "artifact_type!",
                  COUNT(*)::BIGINT AS "artifacts!",
                  COUNT(*) FILTER (WHERE x.is_error)::BIGINT AS "errors!",
                  COUNT(DISTINCT x.user_id)::BIGINT AS "users!",
                  MAX(x.created_at) AS "last_created_at!"
           FROM membership m
           JOIN mcp_artifacts x ON x.user_id = m.user_id
            AND x.created_at >= NOW() - make_interval(days => $4)
           WHERE m.scope_id = $3
           GROUP BY x.server_name, x.artifact_type
           ORDER BY COUNT(*) DESC, x.server_name, x.artifact_type
           LIMIT $5"#,
        q.kind().as_str(),
        q.attribution.is_exclusive(),
        q.id(),
        q.window_days,
        limit
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ProjectArtifactRow {
            server_name: row.server_name,
            artifact_type: row.artifact_type,
            artifacts: row.artifacts,
            errors: row.errors,
            users: row.users,
            last_created_at: row.last_created_at,
        })
        .collect())
}
