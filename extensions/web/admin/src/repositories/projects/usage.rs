//! The projects listing's rollup: one row per project, read in one pass.
//!
//! Every statement here is built on the shared membership CTE, so a project's
//! numbers are attributed by the same rule the rest of the console counts by
//! and the page never restates what membership means. Totals are read under
//! [`Attribution::Exclusive`](crate::repositories::scope::Attribution) so the
//! projects partition the instance; the per-member breakdown is the one place
//! the pages ask for member attribution, and it says so on the screen.
//! Artifacts, like tool calls, are hook-plane rows keyed on the person and
//! read membership too.
//!
//! "Clients" is the coding agent that made the request (`client_kind`) —
//! this instance ships no A2A agents, so the agents a project uses are its
//! Claude Code, Codex and desktop sessions, and the rollup names the busiest.

use serde::Serialize;
use sqlx::PgPool;
use systemprompt_web_shared::ProjectId;

use crate::repositories::scope::{Attribution, ScopeKind};

// Why: the listing reads every project in one pass and orders in Rust, so the
// cap is what keeps that honest. An instance past it has outgrown a flat
// listing and wants a filter, not a taller page.
pub const LISTING_CAP: i64 = 500;

/// One project's row on the listing: membership, the groups feeding it, and
/// the window's traffic, tool health and skill spread.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectRollup {
    pub id: ProjectId,
    pub name: String,
    pub description: Option<String>,
    pub member_count: i64,
    pub attributed_members: i64,
    pub group_count: i64,
    pub active_members: i64,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub models_used: i64,
    pub top_model: Option<String>,
    pub clients_used: i64,
    pub top_client: Option<String>,
    pub tool_calls: i64,
    pub tool_success: i64,
    pub skills_used: i64,
    pub artifacts: i64,
}

pub async fn list_project_rollups(
    pool: &PgPool,
    window_days: i32,
    limit: i64,
) -> Result<Vec<ProjectRollup>, sqlx::Error> {
    crate::scoped_query!(@as ProjectRollup,
        r#"SELECT p.id AS "id!: ProjectId",
                  p.name AS "name!",
                  p.description AS "description?",
                  (SELECT COUNT(DISTINCT pm.user_id) FROM project_members pm
                    WHERE pm.project_id = p.id)::BIGINT AS "member_count!",
                  (SELECT COUNT(DISTINCT ug.group_id)
                     FROM project_members pm
                     JOIN user_groups ug ON ug.user_id = pm.user_id
                    WHERE pm.project_id = p.id)::BIGINT AS "group_count!",
                  (SELECT COUNT(DISTINCT m.user_id) FROM membership m
                    WHERE m.scope_id = p.id)::BIGINT AS "attributed_members!",
                  COALESCE(r.requests, 0)::BIGINT AS "requests!",
                  COALESCE(r.active_members, 0)::BIGINT AS "active_members!",
                  COALESCE(r.tokens, 0)::BIGINT AS "tokens!",
                  COALESCE(r.cost_microdollars, 0)::BIGINT AS "cost_microdollars!",
                  COALESCE(r.models_used, 0)::BIGINT AS "models_used!",
                  r.top_model AS "top_model?",
                  COALESCE(r.clients_used, 0)::BIGINT AS "clients_used!",
                  r.top_client AS "top_client?",
                  COALESCE(a.artifacts, 0)::BIGINT AS "artifacts!",
                  COALESCE(t.calls, 0)::BIGINT AS "tool_calls!",
                  COALESCE(t.ok, 0)::BIGINT AS "tool_success!",
                  COALESCE(s.skills, 0)::BIGINT AS "skills_used!"
           FROM projects p
           LEFT JOIN (
               SELECT m.scope_id, COUNT(x.id) AS requests,
                      COUNT(DISTINCT x.user_id) AS active_members,
                      COALESCE(SUM(COALESCE(x.tokens_used, COALESCE(x.input_tokens, 0) + COALESCE(x.output_tokens, 0) + COALESCE(x.cache_read_tokens, 0) + COALESCE(x.cache_creation_tokens, 0))), 0) AS tokens,
                      COALESCE(SUM(x.cost_microdollars), 0) AS cost_microdollars,
                      COUNT(DISTINCT x.model) AS models_used,
                      MODE() WITHIN GROUP (ORDER BY x.model) AS top_model,
                      COUNT(DISTINCT x.client_kind) FILTER (WHERE x.client_kind NOT IN ('unknown', 'internal')) AS clients_used,
                      MODE() WITHIN GROUP (ORDER BY x.client_kind) FILTER (WHERE x.client_kind NOT IN ('unknown', 'internal')) AS top_client
               FROM membership m
               JOIN ai_requests x ON x.user_id = m.user_id
                AND x.created_at >= NOW() - make_interval(days => $3)
               GROUP BY m.scope_id
           ) r ON r.scope_id = p.id
           LEFT JOIN (
               SELECT m.scope_id, COUNT(*) AS calls,
                      COUNT(*) FILTER (WHERE x.status = 'success') AS ok
               FROM membership m
               JOIN mcp_tool_executions x ON x.user_id = m.user_id
                AND x.started_at >= NOW() - make_interval(days => $3)
               GROUP BY m.scope_id
           ) t ON t.scope_id = p.id
           LEFT JOIN (
               SELECT m.scope_id, COUNT(DISTINCT e.skill) AS skills
               FROM membership m
               JOIN skill_invocation_events e ON e.user_id = m.user_id
                AND e.invoked_at >= NOW() - make_interval(days => $3)
               WHERE e.skill IS NOT NULL GROUP BY m.scope_id
           ) s ON s.scope_id = p.id
           LEFT JOIN (
               SELECT m.scope_id, COUNT(*) AS artifacts
               FROM membership m
               JOIN mcp_artifacts x ON x.user_id = m.user_id
                AND x.created_at >= NOW() - make_interval(days => $3)
               GROUP BY m.scope_id
           ) a ON a.scope_id = p.id
           ORDER BY p.name
           LIMIT $4"#,
        ScopeKind::Project.as_str(),
        Attribution::Exclusive.is_exclusive(),
        window_days,
        limit
    )
    .fetch_all(pool)
    .await
}
