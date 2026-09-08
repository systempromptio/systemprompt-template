//! What a project's window was spent on, section by section: MCP tool health,
//! skill effectiveness, the sessions that carried the work and the commits
//! that came out of it.
//!
//! Sibling of [`super::usage`], which owns the listing rollup. Both are built
//! on the shared membership CTE, so every figure here is attributed by the
//! same rule the rest of the console counts by.

use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

use crate::repositories::scope::ScopeQuery;

/// One MCP tool as the project used it, with the share that failed.
#[derive(Debug, Clone, Serialize)]
pub struct ToolHealthRow {
    pub server_name: String,
    pub tool_name: String,
    pub calls: i64,
    pub failures: i64,
    pub users: i64,
    pub p95_ms: i64,
}

/// One skill, how much the project leant on it, and what its users rated it.
#[derive(Debug, Clone, Serialize)]
pub struct SkillEffectivenessRow {
    pub skill: String,
    pub invocations: i64,
    pub users: i64,
    pub rating_count: i64,
    pub rating_avg: Option<f64>,
}

/// One session the project's people ran, summarised from its requests.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectSessionRow {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub requests: i64,
    pub cost_microdollars: i64,
    pub models: i64,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
}

/// One commit a project member landed inside the window.
#[derive(Debug, Clone, Serialize)]
pub struct ProjectCommitRow {
    pub commit_hash: String,
    pub user_id: UserId,
    pub branch: Option<String>,
    pub message: String,
    pub files_changed: Option<i32>,
    pub insertions: Option<i32>,
    pub deletions: Option<i32>,
    pub committed_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_project_tool_health(
    pool: &PgPool,
    q: &ScopeQuery<'_>,
    limit: i64,
) -> Result<Vec<ToolHealthRow>, sqlx::Error> {
    let rows = crate::scoped_query!(
        r#"SELECT x.server_name AS "server_name!",
                  x.tool_name AS "tool_name!",
                  COUNT(*)::BIGINT AS "calls!",
                  COUNT(*) FILTER (WHERE x.status <> 'success')::BIGINT AS "failures!",
                  COUNT(DISTINCT x.user_id)::BIGINT AS "users!",
                  COALESCE(
                      PERCENTILE_DISC(0.95) WITHIN GROUP (
                          ORDER BY COALESCE(x.execution_time_ms, 0)), 0)::BIGINT AS "p95_ms!"
           FROM membership m
           JOIN mcp_tool_executions x ON x.user_id = m.user_id
           WHERE m.scope_id = $3
             AND x.started_at >= NOW() - make_interval(days => $4)
           GROUP BY 1, 2
           ORDER BY 4 DESC, 3 DESC, 2
           LIMIT $5"#,
        q.kind.as_str(),
        q.attribution.is_exclusive(),
        q.id,
        q.window_days,
        limit
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ToolHealthRow {
            server_name: row.server_name,
            tool_name: row.tool_name,
            calls: row.calls,
            failures: row.failures,
            users: row.users,
            p95_ms: row.p95_ms,
        })
        .collect())
}

pub async fn list_project_skill_effectiveness(
    pool: &PgPool,
    q: &ScopeQuery<'_>,
    limit: i64,
) -> Result<Vec<SkillEffectivenessRow>, sqlx::Error> {
    let rows = crate::scoped_query!(
        r#"SELECT e.skill AS "skill!",
                  COUNT(*)::BIGINT AS "invocations!",
                  COUNT(DISTINCT e.user_id)::BIGINT AS "users!",
                  (SELECT COUNT(*) FROM skill_ratings sr
                    JOIN membership m2 ON m2.user_id = sr.user_id
                   WHERE m2.scope_id = $3 AND sr.skill_name = e.skill)::BIGINT AS "rating_count!",
                  (SELECT AVG(sr.rating)::FLOAT8 FROM skill_ratings sr
                    JOIN membership m2 ON m2.user_id = sr.user_id
                   WHERE m2.scope_id = $3 AND sr.skill_name = e.skill) AS "rating_avg?"
           FROM membership m
           JOIN skill_invocation_events e ON e.user_id = m.user_id
           WHERE m.scope_id = $3
             AND e.skill IS NOT NULL
             AND e.invoked_at >= NOW() - make_interval(days => $4)
           GROUP BY 1
           ORDER BY 2 DESC, 1
           LIMIT $5"#,
        q.kind.as_str(),
        q.attribution.is_exclusive(),
        q.id,
        q.window_days,
        limit
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| SkillEffectivenessRow {
            skill: row.skill,
            invocations: row.invocations,
            users: row.users,
            rating_count: row.rating_count,
            rating_avg: row.rating_avg,
        })
        .collect())
}

pub async fn list_project_sessions(
    pool: &PgPool,
    q: &ScopeQuery<'_>,
    limit: i64,
) -> Result<Vec<ProjectSessionRow>, sqlx::Error> {
    let rows = crate::scoped_query!(
        r#"SELECT s.session_id AS "session_id!: SessionId",
                  m.user_id AS "user_id!: UserId",
                  COUNT(x.id)::BIGINT AS "requests!",
                  COALESCE(SUM(x.cost_microdollars), 0)::BIGINT AS "cost_microdollars!",
                  COUNT(DISTINCT x.model)::BIGINT AS "models!",
                  s.started_at AS "started_at!",
                  s.last_activity_at AS "last_activity_at!"
           FROM membership m
           JOIN user_sessions s ON s.user_id = m.user_id
           LEFT JOIN ai_requests x ON x.session_id = s.session_id
           WHERE m.scope_id = $3
             AND s.started_at >= NOW() - make_interval(days => $4)
           GROUP BY s.session_id, m.user_id, s.started_at, s.last_activity_at
           ORDER BY s.last_activity_at DESC
           LIMIT $5"#,
        q.kind.as_str(),
        q.attribution.is_exclusive(),
        q.id,
        q.window_days,
        limit
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ProjectSessionRow {
            session_id: row.session_id,
            user_id: row.user_id,
            requests: row.requests,
            cost_microdollars: row.cost_microdollars,
            models: row.models,
            started_at: row.started_at,
            last_activity_at: row.last_activity_at,
        })
        .collect())
}

pub async fn list_project_commits(
    pool: &PgPool,
    q: &ScopeQuery<'_>,
    limit: i64,
) -> Result<Vec<ProjectCommitRow>, sqlx::Error> {
    let rows = crate::scoped_query!(
        r#"SELECT c.commit_hash AS "commit_hash!",
                  c.user_id AS "user_id!: UserId",
                  c.branch AS "branch?",
                  c.message AS "message!",
                  c.files_changed AS "files_changed?",
                  c.insertions AS "insertions?",
                  c.deletions AS "deletions?",
                  c.committed_at AS "committed_at!"
           FROM membership m
           JOIN user_commits c ON c.user_id = m.user_id
           WHERE m.scope_id = $3
             AND c.committed_at >= NOW() - make_interval(days => $4)
           ORDER BY c.committed_at DESC
           LIMIT $5"#,
        q.kind.as_str(),
        q.attribution.is_exclusive(),
        q.id,
        q.window_days,
        limit
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| ProjectCommitRow {
            commit_hash: row.commit_hash,
            user_id: row.user_id,
            branch: row.branch,
            message: row.message,
            files_changed: row.files_changed,
            insertions: row.insertions,
            deletions: row.deletions,
            committed_at: row.committed_at,
        })
        .collect())
}
