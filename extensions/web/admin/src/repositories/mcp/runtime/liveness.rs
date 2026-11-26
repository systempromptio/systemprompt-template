//! Who is attached to an MCP server, and under what identity.
//!
//! The heartbeat half of liveness lives in `repositories::overview::liveness`,
//! which the dashboard also reads — one query answers "when did this server
//! last speak" for both surfaces. What this module adds is the part only the
//! MCP pages need: the live sessions bound to a principal, and the
//! per-session detail behind the count.
//!
//! Whether a server is alive is not decided here either: that rule is
//! `repositories::overview::liveness::liveness_state`, and the MCP pages call
//! it with the heartbeat that module reads. One rule, one interval, two pages.

use sqlx::PgPool;
use systemprompt::identifiers::{McpServerId, SessionId, UserId};

#[derive(Debug, Clone)]
pub struct McpSessionPrincipalCount {
    pub server_id: McpServerId,
    pub bound_sessions: i64,
    pub distinct_users: i64,
}

// Why: The proxy binds each session to its server and principal at the edge
// (core mcp migration 021), so a server with open sessions and none bound is a
// server nothing has authenticated through.
pub async fn list_mcp_session_principal_counts(
    pool: &PgPool,
) -> Result<Vec<McpSessionPrincipalCount>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
             s.mcp_server_id AS "server_id!: McpServerId",
             COUNT(*) FILTER (
                 WHERE s.user_id IS NOT NULL AND s.status = 'active' AND s.expires_at > NOW()
             )::BIGINT AS "bound_sessions!",
             COUNT(DISTINCT s.user_id)::BIGINT AS "distinct_users!"
           FROM mcp_sessions s
          WHERE s.mcp_server_id IS NOT NULL
          GROUP BY s.mcp_server_id
          ORDER BY s.mcp_server_id"#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| McpSessionPrincipalCount {
            server_id: r.server_id,
            bound_sessions: r.bound_sessions,
            distinct_users: r.distinct_users,
        })
        .collect())
}

#[derive(Debug, Clone)]
pub struct McpSessionRow {
    pub session_id: SessionId,
    pub user_id: Option<UserId>,
    pub status: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

// Why: The most recently active sessions attached to one server.
pub async fn list_mcp_sessions_for_server(
    pool: &PgPool,
    server_id: &McpServerId,
    limit: i64,
) -> Result<Vec<McpSessionRow>, sqlx::Error> {
    sqlx::query_as!(
        McpSessionRow,
        r#"SELECT
             s.session_id AS "session_id!: SessionId",
             s.user_id AS "user_id?: UserId",
             s.status AS "status!",
             s.created_at AS "created_at!",
             s.last_activity_at AS "last_activity_at!",
             s.expires_at AS "expires_at!"
           FROM mcp_sessions s
          WHERE s.mcp_server_id = $1
          ORDER BY s.last_activity_at DESC
          LIMIT $2"#,
        server_id.as_str(),
        limit
    )
    .fetch_all(pool)
    .await
}
