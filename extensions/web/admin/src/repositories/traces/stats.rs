//! Per-session percentile stats over the same window as the trace list.

use sqlx::PgPool;

use super::TraceStats;
use crate::util::time_range::TimeRange;

// Why: `subject_ids` is the caller's `SubjectScope::as_sql()`; every source of
// a session (tool events, governance decisions, requests) carries a user id, so
// the same predicate scopes all three.
pub async fn get_trace_stats(
    pool: &PgPool,
    range: TimeRange,
    subject_ids: Option<&[String]>,
) -> Result<TraceStats, sqlx::Error> {
    let row = sqlx::query_file!(
        "src/repositories/traces/stats.sql",
        range.from,
        range.to,
        subject_ids,
    )
    .fetch_one(pool)
    .await?;

    Ok(TraceStats {
        total_traces: row.total_traces,
        error_count: row.error_count,
        deny_count: row.deny_count,
        p50_active_ms: row.p50,
        p95_active_ms: row.p95,
        p99_active_ms: row.p99,
        total_cost_microdollars: row.total_cost,
        total_tokens: row.total_tokens,
    })
}
