//! Sign-in sessions belonging to a user, and revoking them.
//!
//! Named `signin_*` throughout because `users::queries::list_user_sessions`
//! already exists and means something else entirely — a Claude Code work
//! session out of `plugin_session_summaries`. These are `user_sessions` rows:
//! credentials, not activity.
//!
//! `user_sessions` is core-owned and already carries `revoked_at` plus its
//! partial index; nothing in this fork wrote the column until now, so ending a
//! session was a CLI-only act. The admin surface is expected to need no shell
//! access (REQ-001), which is what these functions close.
//!
//! Revoking is an `UPDATE`, never a `DELETE`: the row carries that session's
//! request counts, cost, and traffic attribution, and the analytics rollups
//! read it. Ending a session must not rewrite history.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

#[derive(Debug, Clone, serde::Serialize)]
pub struct SigninSessionRow {
    pub session_id: SessionId,
    pub started_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    // Why: `web`, `bridge`, `cli`, `api`, `oauth`, `mcp` — which door this session
    // came in through, so an admin can tell a browser login from a bridge.
    pub session_source: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_count: i32,
}

// Why: Every session recorded for a user, newest first — revoked ones included,
// so the page can show that an action took effect rather than a row vanishing.
pub async fn list_signin_sessions(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<SigninSessionRow>, sqlx::Error> {
    sqlx::query_as!(
        SigninSessionRow,
        r#"SELECT
               s.session_id AS "session_id!: SessionId",
               s.started_at,
               s.last_activity_at,
               s.expires_at,
               s.revoked_at,
               s.session_source,
               s.ip_address,
               s.user_agent,
               s.request_count
           FROM user_sessions s
           WHERE s.user_id = $1
           ORDER BY s.last_activity_at DESC
           LIMIT 200"#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

// Why: Revoke one session. Returns whether a row moved — a session already
// revoked, or belonging to another user, reports `false` rather than erroring,
// so the handler can answer 404 without a second lookup.
//
// The `user_id` predicate is not redundant with the primary key: it is what
// stops a session id lifted from one user's page ending another user's
// session.
pub async fn revoke_signin_session(
    pool: &PgPool,
    user_id: &UserId,
    session_id: &SessionId,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "UPDATE user_sessions SET revoked_at = NOW(), ended_at = COALESCE(ended_at, NOW())
         WHERE session_id = $1 AND user_id = $2 AND revoked_at IS NULL",
        session_id.as_str(),
        user_id.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn revoke_all_signin_sessions(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "UPDATE user_sessions SET revoked_at = NOW(), ended_at = COALESCE(ended_at, NOW())
         WHERE user_id = $1 AND revoked_at IS NULL",
        user_id.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
