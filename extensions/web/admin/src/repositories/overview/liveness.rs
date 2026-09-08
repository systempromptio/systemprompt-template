//! Which MCP servers are alive, read from the session heartbeat.
//!
//! An MCP server does not report its own health; the sessions it holds do.
//! `mcp_sessions.last_activity_at` moves whenever a client talks to a server,
//! so the most recent activity across a server's sessions is the closest thing
//! to a heartbeat the instance has.
//!
//! The rule is deliberately generous: alive within two heartbeat intervals,
//! because one missed beat is a slow response and two is a pattern. Anything
//! older is reported as stale rather than dead — a server nobody has used
//! today is not the same claim as a server that has fallen over, and the page
//! must not make the second claim from the first's evidence.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

// Why: how often a busy MCP session is expected to touch its row. Not a
// configured value — nothing in `services/mcp/*.yaml` declares a heartbeat, so
// this is the interval the rule is stated in and the one the unit test pins. A
// server YAML gaining a heartbeat field would replace it.
pub const HEARTBEAT_INTERVAL_SECS: i64 = 300;

/// What the heartbeat says about one server: beat within two intervals, beat
/// longer ago than that, or never beat at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Liveness {
    Alive,
    Stale,
    Silent,
}

impl Liveness {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Alive => "Alive",
            Self::Stale => "Stale",
            Self::Silent => "No sessions",
        }
    }

    // Why: the badge tone the strip paints this state in, kept beside the
    // label so the two cannot describe different states.
    #[must_use]
    pub const fn tone(self) -> &'static str {
        match self {
            Self::Alive => "ok",
            Self::Stale => "warn",
            Self::Silent => "muted",
        }
    }

    #[must_use]
    pub const fn is_alive(self) -> bool {
        matches!(self, Self::Alive)
    }
}

// Why: a beat in the future is alive, not impossible — clock skew between the
// server writing the row and the console reading it is not evidence of a
// problem, and a negative age must not read as silence.
#[must_use]
pub fn liveness_state(
    now: DateTime<Utc>,
    last_heartbeat: Option<DateTime<Utc>>,
    interval_secs: i64,
) -> Liveness {
    let Some(last) = last_heartbeat else {
        return Liveness::Silent;
    };
    if (now - last).num_seconds() <= interval_secs.saturating_mul(2) {
        Liveness::Alive
    } else {
        Liveness::Stale
    }
}

/// One server's heartbeat, keyed by the id used in `services/mcp/*.yaml`.
#[derive(Debug, Clone)]
pub struct McpHeartbeatRow {
    pub server_id: String,
    pub last_heartbeat: Option<DateTime<Utc>>,
    pub active_sessions: i64,
}

pub async fn list_mcp_server_liveness(pool: &PgPool) -> Result<Vec<McpHeartbeatRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT s.mcp_server_id AS "server_id!",
               MAX(s.last_activity_at) AS "last_heartbeat?",
               COUNT(*) FILTER (WHERE s.status = 'active')::BIGINT AS "active_sessions!"
        FROM mcp_sessions s
        WHERE s.mcp_server_id IS NOT NULL
        GROUP BY s.mcp_server_id
        ORDER BY s.mcp_server_id
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|row| McpHeartbeatRow {
            server_id: row.server_id,
            last_heartbeat: row.last_heartbeat,
            active_sessions: row.active_sessions,
        })
        .collect())
}
