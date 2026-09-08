//! The seven headline figures of the overview, and the same seven over the
//! immediately preceding window of equal width.
//!
//! Both windows are counted in one statement with `FILTER` clauses, so they
//! see one snapshot of `ai_requests` and a request landing mid-read cannot be
//! counted in one window and missed in the other — the failure that makes a
//! delta lie.
//!
//! `errors` and `denied` do not overlap, and both name a status rather than
//! excluding a list of them. A denial is a request the gateway refused before
//! routing (`rejected`); an error is one that was routed and failed
//! (`failed`). Naming the status positively is what lets each tile link to the
//! request log filtered by exactly the rows it counted — and it is why a
//! legacy status nobody writes any more cannot silently inflate the error
//! rate, which excluding a list of "healthy" statuses does.

use sqlx::PgPool;

use crate::util::time_range::TimeRange;

/// Both windows of the KPI strip. `prev_*` covers `[from - width, from)`.
#[derive(Debug, Default, Clone, Copy)]
pub struct OverviewKpis {
    pub requests: i64,
    pub prev_requests: i64,
    pub cost_microdollars: i64,
    pub prev_cost_microdollars: i64,
    pub active_users: i64,
    pub prev_active_users: i64,
    pub p50_latency_ms: i64,
    pub prev_p50_latency_ms: i64,
    pub p95_latency_ms: i64,
    pub prev_p95_latency_ms: i64,
    pub errors: i64,
    pub prev_errors: i64,
    pub denied: i64,
    pub prev_denied: i64,
}

impl OverviewKpis {
    // Why: tenths of a percent rather than a float, so the tile and its delta
    // are derived from one integer and cannot round apart.
    #[must_use]
    pub const fn error_rate_tenths(&self) -> i64 {
        rate_tenths(self.errors, self.requests)
    }

    #[must_use]
    pub const fn prev_error_rate_tenths(&self) -> i64 {
        rate_tenths(self.prev_errors, self.prev_requests)
    }
}

// Why: an empty window has no rate rather than a rate of zero, and the caller
// renders that as an em dash. Zero would claim a measurement nobody took.
const fn rate_tenths(part: i64, whole: i64) -> i64 {
    if whole <= 0 {
        return 0;
    }
    (part * 1000) / whole
}

pub async fn get_overview_kpis(
    pool: &PgPool,
    range: TimeRange,
) -> Result<OverviewKpis, sqlx::Error> {
    // Why: the previous window's edge is computed here rather than in SQL so
    // the two windows are the same width to the microsecond.
    let prev_from = range.from - (range.to - range.from);
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE r.created_at >= $1 AND r.created_at < $2)::BIGINT
                AS "requests!",
            COUNT(*) FILTER (WHERE r.created_at >= $3 AND r.created_at < $1)::BIGINT
                AS "prev_requests!",
            COALESCE(SUM(r.cost_microdollars)
                FILTER (WHERE r.created_at >= $1 AND r.created_at < $2), 0)::BIGINT
                AS "cost!",
            COALESCE(SUM(r.cost_microdollars)
                FILTER (WHERE r.created_at >= $3 AND r.created_at < $1), 0)::BIGINT
                AS "prev_cost!",
            COUNT(DISTINCT r.user_id) FILTER (WHERE r.created_at >= $1 AND r.created_at < $2)::BIGINT
                AS "active_users!",
            COUNT(DISTINCT r.user_id) FILTER (WHERE r.created_at >= $3 AND r.created_at < $1)::BIGINT
                AS "prev_active_users!",
            COALESCE(percentile_cont(0.5) WITHIN GROUP (ORDER BY r.latency_ms)
                FILTER (WHERE r.created_at >= $1 AND r.created_at < $2
                          AND r.latency_ms IS NOT NULL), 0)::BIGINT
                AS "p50!",
            COALESCE(percentile_cont(0.5) WITHIN GROUP (ORDER BY r.latency_ms)
                FILTER (WHERE r.created_at >= $3 AND r.created_at < $1
                          AND r.latency_ms IS NOT NULL), 0)::BIGINT
                AS "prev_p50!",
            COALESCE(percentile_cont(0.95) WITHIN GROUP (ORDER BY r.latency_ms)
                FILTER (WHERE r.created_at >= $1 AND r.created_at < $2
                          AND r.latency_ms IS NOT NULL), 0)::BIGINT
                AS "p95!",
            COALESCE(percentile_cont(0.95) WITHIN GROUP (ORDER BY r.latency_ms)
                FILTER (WHERE r.created_at >= $3 AND r.created_at < $1
                          AND r.latency_ms IS NOT NULL), 0)::BIGINT
                AS "prev_p95!",
            COUNT(*) FILTER (WHERE r.created_at >= $1 AND r.created_at < $2
                AND r.status = 'failed')::BIGINT
                AS "errors!",
            COUNT(*) FILTER (WHERE r.created_at >= $3 AND r.created_at < $1
                AND r.status = 'failed')::BIGINT
                AS "prev_errors!",
            COUNT(*) FILTER (WHERE r.created_at >= $1 AND r.created_at < $2
                AND r.status = 'rejected')::BIGINT
                AS "denied!",
            COUNT(*) FILTER (WHERE r.created_at >= $3 AND r.created_at < $1
                AND r.status = 'rejected')::BIGINT
                AS "prev_denied!"
        FROM ai_requests r
        WHERE NOT r.synthetic
          AND r.created_at >= $3
          AND r.created_at < $2
        "#,
        range.from,
        range.to,
        prev_from,
    )
    .fetch_one(pool)
    .await?;

    Ok(OverviewKpis {
        requests: row.requests,
        prev_requests: row.prev_requests,
        cost_microdollars: row.cost,
        prev_cost_microdollars: row.prev_cost,
        active_users: row.active_users,
        prev_active_users: row.prev_active_users,
        p50_latency_ms: row.p50,
        prev_p50_latency_ms: row.prev_p50,
        p95_latency_ms: row.p95,
        prev_p95_latency_ms: row.prev_p95,
        errors: row.errors,
        prev_errors: row.prev_errors,
        denied: row.denied,
        prev_denied: row.prev_denied,
    })
}
