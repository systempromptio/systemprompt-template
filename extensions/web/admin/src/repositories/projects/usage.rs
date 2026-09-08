//! The projects listing's rollup: one row per project, read in one pass.
//!
//! Every statement here is built on the shared membership CTE, so a project's
//! numbers are attributed by the same rule the rest of the console counts by
//! and the page never restates what membership means. Totals are read under
//! [`Attribution::Exclusive`](crate::repositories::scope::Attribution) so the
//! projects partition the instance; the per-member breakdown is the one place
//! the pages ask for member attribution, and it says so on the screen.

use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::scope::{Attribution, ScopeKind};

// Why: the listing reads every project in one pass and orders in Rust, so the
// cap is what keeps that honest. An instance past it has outgrown a flat
// listing and wants a filter, not a taller page.
pub const LISTING_CAP: i64 = 500;

/// One project's row on the listing: membership, the groups feeding it, and
/// the window's traffic, tool health and skill spread.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectRollup {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub member_count: i64,
    pub attributed_members: i64,
    pub group_count: i64,
    pub active_members: i64,
    pub requests: i64,
    pub cost_microdollars: i64,
    pub tool_calls: i64,
    pub tool_success: i64,
    pub skills_used: i64,
}

pub async fn list_project_rollups(
    pool: &PgPool,
    window_days: i32,
    limit: i64,
) -> Result<Vec<ProjectRollup>, sqlx::Error> {
    let rows = crate::scoped_query!(
        r#"SELECT p.id AS "id!",
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
                  COALESCE(r.cost_microdollars, 0)::BIGINT AS "cost_microdollars!",
                  COALESCE(t.calls, 0)::BIGINT AS "tool_calls!",
                  COALESCE(t.ok, 0)::BIGINT AS "tool_success!",
                  COALESCE(s.skills, 0)::BIGINT AS "skills_used!"
           FROM projects p
           LEFT JOIN (
               SELECT m.scope_id, COUNT(x.id) AS requests,
                      COUNT(DISTINCT x.user_id) AS active_members,
                      COALESCE(SUM(x.cost_microdollars), 0) AS cost_microdollars
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
           ORDER BY p.name
           LIMIT $4"#,
        ScopeKind::Project.as_str(),
        Attribution::Exclusive.is_exclusive(),
        window_days,
        limit
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ProjectRollup {
            id: row.id,
            name: row.name,
            description: row.description,
            member_count: row.member_count,
            attributed_members: row.attributed_members,
            group_count: row.group_count,
            active_members: row.active_members,
            requests: row.requests,
            cost_microdollars: row.cost_microdollars,
            tool_calls: row.tool_calls,
            tool_success: row.tool_success,
            skills_used: row.skills_used,
        })
        .collect())
}
