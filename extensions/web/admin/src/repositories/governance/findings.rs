//! The paged safety-finding log from `ai_safety_findings`.
//!
//! The second enforcement plane. Unlike the governance chain these findings run
//! in both directions, so `phase` is the direction — a request-phase finding
//! read what the caller sent, a response-phase one read what the model said.
//!
//! The table carries no identity of its own, so every query joins `ai_requests`
//! for the caller. That join is also what makes the rows scopeable: a project
//! manager's scope is a user-id list, and the finding inherits it from the
//! request it hangs off.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::repositories::scope::SubjectScope;
use crate::util::time_range::TimeRange;

/// One scanner finding, as the safety tab renders it.
#[derive(Debug, Clone)]
pub struct SafetyFindingLogRow {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub category: String,
    pub scanner: String,
    pub severity: String,
    pub phase: String,
    pub blocked: bool,
    pub excerpt: Option<String>,
    pub ai_request_id: String,
    pub user_id: Option<UserId>,
    pub model: Option<String>,
}

/// The safety KPIs: what was scanned, and what that scan actually refused.
#[derive(Debug, Clone, Copy, Default)]
pub struct SafetyStats {
    pub findings: i64,
    pub blocked: i64,
    pub audited: i64,
    pub categories: i64,
    pub inbound: i64,
    pub outbound: i64,
}

/// What the toolbar narrowed the findings to.
#[derive(Debug, Clone, Default)]
pub struct FindingFilter {
    pub category: Option<String>,
    pub blocked: Option<bool>,
}

#[expect(
    clippy::too_many_arguments,
    reason = "page query plumbing; splitting the parameters is tracked in docs/tech-debt.md"
)]
pub async fn list_safety_findings_paged(
    pool: &PgPool,
    range: TimeRange,
    scope: &SubjectScope,
    filter: &FindingFilter,
    limit: i64,
    offset: i64,
) -> Result<(Vec<SafetyFindingLogRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as!(
        SafetyFindingLogRow,
        r#"SELECT f.id, f.created_at, f.category, f.scanner, f.severity, f.phase,
                  f.blocked, f.excerpt, f.ai_request_id,
                  r.user_id AS "user_id: UserId", r.model
           FROM ai_safety_findings f
           LEFT JOIN ai_requests r ON r.id = f.ai_request_id
           WHERE f.created_at >= $1 AND f.created_at < $2
             AND ($3::TEXT[] IS NULL OR r.user_id = ANY($3))
             AND ($4::TEXT IS NULL OR f.category = $4)
             AND ($5::BOOLEAN IS NULL OR f.blocked = $5)
           ORDER BY f.created_at DESC, f.id
           LIMIT $6 OFFSET $7"#,
        range.from,
        range.to,
        scope.as_sql(),
        filter.category.as_deref(),
        filter.blocked,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*)::BIGINT AS "n!"
           FROM ai_safety_findings f
           LEFT JOIN ai_requests r ON r.id = f.ai_request_id
           WHERE f.created_at >= $1 AND f.created_at < $2
             AND ($3::TEXT[] IS NULL OR r.user_id = ANY($3))
             AND ($4::TEXT IS NULL OR f.category = $4)
             AND ($5::BOOLEAN IS NULL OR f.blocked = $5)"#,
        range.from,
        range.to,
        scope.as_sql(),
        filter.category.as_deref(),
        filter.blocked,
    )
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

// Why: `findings` and `blocked` are deliberately separate measures. Under warn
// mode a finding in a blocking category is recorded and the call proceeds, so a
// non-zero findings count beside a zero blocked count is the signal that the
// plane is absorbing rather than enforcing.
pub async fn get_safety_stats(
    pool: &PgPool,
    range: TimeRange,
    scope: &SubjectScope,
) -> Result<SafetyStats, sqlx::Error> {
    sqlx::query_as!(
        SafetyStats,
        r#"SELECT
             COUNT(*)::BIGINT AS "findings!",
             COUNT(*) FILTER (WHERE f.blocked)::BIGINT AS "blocked!",
             COUNT(*) FILTER (WHERE NOT f.blocked)::BIGINT AS "audited!",
             COUNT(DISTINCT f.category)::BIGINT AS "categories!",
             COUNT(*) FILTER (WHERE f.phase = 'request')::BIGINT AS "inbound!",
             COUNT(*) FILTER (WHERE f.phase <> 'request')::BIGINT AS "outbound!"
           FROM ai_safety_findings f
           LEFT JOIN ai_requests r ON r.id = f.ai_request_id
           WHERE f.created_at >= $1 AND f.created_at < $2
             AND ($3::TEXT[] IS NULL OR r.user_id = ANY($3))"#,
        range.from,
        range.to,
        scope.as_sql(),
    )
    .fetch_one(pool)
    .await
}

// Why: the select is built from what the window actually holds, not from a
// fixed list — a category nobody tripped is not an option worth offering.
pub async fn list_finding_categories(
    pool: &PgPool,
    range: TimeRange,
    scope: &SubjectScope,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT f.category AS "category!"
           FROM ai_safety_findings f
           LEFT JOIN ai_requests r ON r.id = f.ai_request_id
           WHERE f.created_at >= $1 AND f.created_at < $2
             AND ($3::TEXT[] IS NULL OR r.user_id = ANY($3))
           GROUP BY f.category
           ORDER BY COUNT(*) DESC, f.category"#,
        range.from,
        range.to,
        scope.as_sql(),
    )
    .fetch_all(pool)
    .await
}
