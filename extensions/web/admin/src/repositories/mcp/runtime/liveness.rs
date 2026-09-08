//! Who is attached to an MCP server, and under what identity.
//!
//! The heartbeat half of liveness lives in `repositories::overview::liveness`,
//! which the dashboard also reads — one query answers "when did this server
//! last speak" for both surfaces. What this module adds is the part only the
//! MCP pages need: the proxy identities a live session is acting as, and the
//! per-session detail behind the count.
//!
//! Whether a server is alive is not decided here either: that rule is
//! `repositories::overview::liveness::liveness_state`, and the MCP pages call
//! it with the heartbeat that module reads. One rule, one interval, two pages.

use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

#[derive(Debug, Clone)]
pub struct McpProxyIdentityCount {
    pub server_id: String,
    pub identities: i64,
    pub distinct_users: i64,
}

// Why: Unexpired proxy identities per server, with the people behind them.
//
// Keyed on the session the identity was minted for, so a server with open
// sessions and no identities is a server nothing has authenticated through —
// which is a different state from having no sessions at all.
pub async fn list_mcp_proxy_identity_counts(
    pool: &PgPool,
) -> Result<Vec<McpProxyIdentityCount>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
             s.mcp_server_id AS "server_id!",
             COUNT(p.session_id) FILTER (WHERE p.expires_at > NOW())::BIGINT AS "identities!",
             COUNT(DISTINCT s.user_id)::BIGINT AS "distinct_users!"
           FROM mcp_sessions s
           LEFT JOIN mcp_proxy_identities p ON p.session_id = s.session_id
          WHERE s.mcp_server_id IS NOT NULL
          GROUP BY s.mcp_server_id
          ORDER BY s.mcp_server_id"#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| McpProxyIdentityCount {
            server_id: r.server_id,
            identities: r.identities,
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
    pub has_proxy_identity: bool,
    pub proxy_user_type: Option<String>,
}

// Why: The most recently active sessions attached to one server.
pub async fn list_mcp_sessions_for_server(
    pool: &PgPool,
    server_id: &str,
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
             s.expires_at AS "expires_at!",
             (p.session_id IS NOT NULL AND p.expires_at > NOW()) AS "has_proxy_identity!",
             p.user_type AS "proxy_user_type?"
           FROM mcp_sessions s
           LEFT JOIN mcp_proxy_identities p ON p.session_id = s.session_id
          WHERE s.mcp_server_id = $1
          ORDER BY s.last_activity_at DESC
          LIMIT $2"#,
        server_id,
        limit
    )
    .fetch_all(pool)
    .await
}
