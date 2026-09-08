//! The human-approval queue: calls the `require_approval` policy parked.
//!
//! `approval_requests` is core's table and core's writer opens the rows; the
//! console is the reader and the decider. A row is a rendezvous point — the MCP
//! server that parked the call is blocked on it — so a decision here releases a
//! caller that is still waiting, which is why the queue leads with age.
//!
//! Expiry is a lapsed decision rather than a stored one: the writer never comes
//! back to restamp a row nobody answered, so a `pending` row past `expires_at`
//! is read as expired here and by the predicate the queue filters on.

use chrono::{DateTime, Utc};
// JSON: the tool arguments verbatim, as the enforcement point parked them.
// An approver authorises exactly the payload that will run, and `args_digest`
// binds the decision to it — re-rendering them through a typed struct would
// change what was authorised.
use serde_json::Value;
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

/// One held call, as the queue renders it.
#[derive(Debug, Clone)]
pub struct ApprovalRow {
    pub call_id: String,
    pub tool_name: String,
    pub server_name: String,
    pub arguments: Value,
    pub args_digest: String,
    pub requested_by: UserId,
    pub session_id: Option<SessionId>,
    pub trace_id: Option<String>,
    pub rule: String,
    pub status: String,
    pub approver_id: Option<String>,
    pub approver_username: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decision_note: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl ApprovalRow {
    // Why: The status a reader sees, with an unanswered expiry counted as expired.
    #[must_use]
    pub fn effective_status(&self) -> &str {
        if self.status == "pending" && self.expires_at <= Utc::now() {
            "expired"
        } else {
            self.status.as_str()
        }
    }

    #[must_use]
    pub fn is_actionable(&self) -> bool {
        self.effective_status() == "pending"
    }
}

/// The queue's KPI strip.
#[derive(Debug, Clone, Copy, Default)]
pub struct ApprovalStats {
    pub pending: i64,
    pub approved: i64,
    pub denied: i64,
    pub expired: i64,
    pub oldest_pending_minutes: i64,
}

pub async fn list_approvals_paged(
    pool: &PgPool,
    status: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<ApprovalRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as!(
        ApprovalRow,
        r#"SELECT a.call_id, a.tool_name, a.server_name, a.arguments, a.args_digest,
                  a.requested_by AS "requested_by!: UserId",
                  a.session_id AS "session_id: SessionId", a.trace_id,
                  a.rule, a.status, a.approver_id, a.approver_username, a.decided_at,
                  a.decision_note, a.expires_at, a.created_at
           FROM approval_requests a
           WHERE ($1::TEXT IS NULL
                  OR ($1 = 'pending' AND a.status = 'pending' AND a.expires_at > NOW())
                  OR ($1 = 'expired'
                      AND (a.status = 'expired'
                           OR (a.status = 'pending' AND a.expires_at <= NOW())))
                  OR ($1 NOT IN ('pending', 'expired') AND a.status = $1))
           ORDER BY (a.status = 'pending' AND a.expires_at > NOW()) DESC,
                    a.created_at DESC
           LIMIT $2 OFFSET $3"#,
        status,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*)::BIGINT AS "n!"
           FROM approval_requests a
           WHERE ($1::TEXT IS NULL
                  OR ($1 = 'pending' AND a.status = 'pending' AND a.expires_at > NOW())
                  OR ($1 = 'expired'
                      AND (a.status = 'expired'
                           OR (a.status = 'pending' AND a.expires_at <= NOW())))
                  OR ($1 NOT IN ('pending', 'expired') AND a.status = $1))"#,
        status,
    )
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

pub async fn find_approval(
    pool: &PgPool,
    call_id: &str,
) -> Result<Option<ApprovalRow>, sqlx::Error> {
    sqlx::query_as!(
        ApprovalRow,
        r#"SELECT a.call_id, a.tool_name, a.server_name, a.arguments, a.args_digest,
                  a.requested_by AS "requested_by!: UserId",
                  a.session_id AS "session_id: SessionId", a.trace_id,
                  a.rule, a.status, a.approver_id, a.approver_username, a.decided_at,
                  a.decision_note, a.expires_at, a.created_at
           FROM approval_requests a
           WHERE a.call_id = $1"#,
        call_id,
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_approval_stats(pool: &PgPool) -> Result<ApprovalStats, sqlx::Error> {
    sqlx::query_as!(
        ApprovalStats,
        r#"SELECT
             COUNT(*) FILTER (WHERE status = 'pending' AND expires_at > NOW())::BIGINT
               AS "pending!",
             COUNT(*) FILTER (WHERE status = 'approved')::BIGINT AS "approved!",
             COUNT(*) FILTER (WHERE status = 'denied')::BIGINT AS "denied!",
             COUNT(*) FILTER (WHERE status = 'expired'
                              OR (status = 'pending' AND expires_at <= NOW()))::BIGINT
               AS "expired!",
             COALESCE(
               EXTRACT(EPOCH FROM (NOW() - MIN(created_at)
                 FILTER (WHERE status = 'pending' AND expires_at > NOW()))) / 60,
               0)::BIGINT AS "oldest_pending_minutes!"
           FROM approval_requests"#,
    )
    .fetch_one(pool)
    .await
}

// Why: the `status = 'pending'` predicate is the concurrency control. Two
// approvers looking at the same queue is the normal case, and the second write
// must lose rather than overwrite the first decision — an approval that
// silently replaced a deny would be the one bug this table exists to prevent.
#[expect(
    clippy::too_many_arguments,
    reason = "page query plumbing; splitting the parameters is tracked in docs/tech-debt.md"
)]
pub async fn update_approval_decision(
    pool: &PgPool,
    call_id: &str,
    status: &str,
    approver: &UserId,
    approver_username: &str,
    note: Option<&str>,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query!(
        "UPDATE approval_requests
            SET status = $2, approver_id = $3, approver_username = $4,
                decided_at = NOW(), decision_note = $5
          WHERE call_id = $1 AND status = 'pending'",
        call_id,
        status,
        approver.as_str(),
        approver_username,
        note,
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
