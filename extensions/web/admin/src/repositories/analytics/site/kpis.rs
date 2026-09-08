//! Headline figures for the dashboard's KPI strip.
//!
//! The token total here must match the profile pane's exactly. The canonical
//! definition, and why it is what it is, lives in
//! [`crate::repositories::users::usage`]; the two are pinned together by
//! `tests/integration/admin-core/src/usage_reconciliation.rs`.

use sqlx::PgPool;

use crate::util::time_range::TimeRange;

use super::SiteScope;

#[derive(Debug, Default, Clone, Copy)]
pub struct SiteKpis {
    pub total_requests: i64,
    pub error_count: i64,
    pub total_cost_microdollars: i64,
    pub total_tokens: i64,
    // Why: reasoning is billed inside `output_tokens` and inside
    // `tokens_used`; it is reported beside them as a share, never added.
    pub reasoning_tokens: i64,
    pub output_tokens: i64,
    pub active_users: i64,
    // Why: Distinct users with at least one request in the trailing 7 days —
    // computed against `NOW()` regardless of the picked window, and labeled
    // that way on the page.
    pub weekly_active_users: i64,
    // Why: The same aggregates over the immediately preceding window of equal
    // width (`[from - (to-from), from)`), read in the same statement so both
    // windows see one snapshot and a delta can never be skewed by writes
    // landing between two queries.
    pub prev_total_requests: i64,
    pub prev_error_count: i64,
    pub prev_total_cost_microdollars: i64,
    pub prev_total_tokens: i64,
    pub prev_active_users: i64,
    // Why: WAU for the prior trailing week (`NOW()-14d .. NOW()-7d`), anchored to
    // the same `NOW()` as `weekly_active_users`.
    pub prev_weekly_active_users: i64,
}

// Why: current and previous window are counted in one pass with `FILTER`
// clauses rather than two round trips, so both totals see exactly the same
// snapshot of `ai_requests` and a request landing mid-read cannot be counted
// in one window and missed in the other.
#[expect(
    clippy::too_many_lines,
    reason = "one `sqlx::query!` statement: both windows must be counted in a \
              single pass, and splitting it would split the snapshot"
)]
pub async fn get_site_kpis(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<SiteKpis, sqlx::Error> {
    // Why: the previous window's edge is computed here, not in SQL, so the
    // two windows are guaranteed the same width to the microsecond.
    let prev_from = range.from - (range.to - range.from);
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE r.created_at >= $1 AND r.created_at < $2)::BIGINT
                AS "total!",
            COUNT(*) FILTER (WHERE r.created_at >= $1 AND r.created_at < $2
                AND r.status NOT IN ('completed', 'success', 'pending', 'streaming'))::BIGINT
                AS "errors!",
            COALESCE(SUM(r.cost_microdollars)
                FILTER (WHERE r.created_at >= $1 AND r.created_at < $2), 0)::BIGINT
                AS "cost!",
            COALESCE(SUM(r.tokens_used)
                FILTER (WHERE r.created_at >= $1 AND r.created_at < $2), 0)::BIGINT
                AS "tokens!",
            COALESCE(SUM(r.reasoning_tokens)
                FILTER (WHERE r.created_at >= $1 AND r.created_at < $2), 0)::BIGINT
                AS "reasoning_tokens!",
            COALESCE(SUM(r.output_tokens)
                FILTER (WHERE r.created_at >= $1 AND r.created_at < $2), 0)::BIGINT
                AS "output_tokens!",
            COUNT(DISTINCT r.user_id)
                FILTER (WHERE r.created_at >= $1 AND r.created_at < $2)::BIGINT
                AS "active_users!",
            COUNT(DISTINCT r.user_id)
                FILTER (WHERE r.created_at >= NOW() - INTERVAL '7 days')::BIGINT
                AS "weekly_active_users!",
            COUNT(*) FILTER (WHERE r.created_at >= $5 AND r.created_at < $1)::BIGINT
                AS "prev_total!",
            COUNT(*) FILTER (WHERE r.created_at >= $5 AND r.created_at < $1
                AND r.status NOT IN ('completed', 'success', 'pending', 'streaming'))::BIGINT
                AS "prev_errors!",
            COALESCE(SUM(r.cost_microdollars)
                FILTER (WHERE r.created_at >= $5 AND r.created_at < $1), 0)::BIGINT
                AS "prev_cost!",
            COALESCE(SUM(r.tokens_used)
                FILTER (WHERE r.created_at >= $5 AND r.created_at < $1), 0)::BIGINT
                AS "prev_tokens!",
            COUNT(DISTINCT r.user_id)
                FILTER (WHERE r.created_at >= $5 AND r.created_at < $1)::BIGINT
                AS "prev_active_users!",
            COUNT(DISTINCT r.user_id)
                FILTER (WHERE r.created_at >= NOW() - INTERVAL '14 days'
                          AND r.created_at < NOW() - INTERVAL '7 days')::BIGINT
                AS "prev_weekly_active_users!"
        FROM ai_requests r
        WHERE NOT r.synthetic
          AND (r.created_at >= $5 OR r.created_at >= NOW() - INTERVAL '14 days')
          AND ($3::TEXT[] IS NULL OR r.user_id = ANY($3))
          AND ($4::TEXT IS NULL OR r.user_id = $4)
        "#,
        range.from,
        range.to,
        scope.scope.as_sql(),
        scope.user_id_str(),
        prev_from,
    )
    .fetch_one(pool)
    .await?;

    Ok(SiteKpis {
        total_requests: row.total,
        error_count: row.errors,
        total_cost_microdollars: row.cost,
        total_tokens: row.tokens,
        reasoning_tokens: row.reasoning_tokens,
        output_tokens: row.output_tokens,
        active_users: row.active_users,
        weekly_active_users: row.weekly_active_users,
        prev_total_requests: row.prev_total,
        prev_error_count: row.prev_errors,
        prev_total_cost_microdollars: row.prev_cost,
        prev_total_tokens: row.prev_tokens,
        prev_active_users: row.prev_active_users,
        prev_weekly_active_users: row.prev_weekly_active_users,
    })
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PermissionGrantStats {
    pub requests: i64,
    // Why: Each request owns the interval up to its own successor, so a
    // re-prompted identical call is attributed to its own request rather than
    // to an earlier one. Still an observable proxy: the hook stream reports
    // that a tool ran, not that a human clicked allow.
    pub granted: i64,
}

pub async fn get_permission_grant_stats(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<PermissionGrantStats, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*)::BIGINT AS "requests!",
            COUNT(*) FILTER (WHERE EXISTS (
                SELECT 1 FROM plugin_usage_events g
                WHERE g.session_id = p.session_id
                  AND g.event_type IN ('PostToolUse', 'PostToolUseFailure')
                  AND g.tool_name IS NOT DISTINCT FROM p.tool_name
                  AND g.created_at >= p.created_at
                  AND (p.next_request_at IS NULL OR g.created_at < p.next_request_at)
            ))::BIGINT AS "granted!"
        FROM (
            SELECT e.*,
                   LEAD(e.created_at) OVER (
                       PARTITION BY e.session_id, e.tool_name ORDER BY e.created_at
                   ) AS next_request_at
            FROM plugin_usage_events e
            WHERE e.event_type = 'PermissionRequest'
        ) p
        WHERE p.created_at >= $1 AND p.created_at < $2
          AND ($3::TEXT[] IS NULL OR p.user_id = ANY($3))
          AND ($4::TEXT IS NULL OR p.user_id = $4)
        "#,
        range.from,
        range.to,
        scope.scope.as_sql(),
        scope.user_id_str(),
    )
    .fetch_one(pool)
    .await?;

    Ok(PermissionGrantStats {
        requests: row.requests,
        granted: row.granted,
    })
}
