//! Per-dimension `ai_requests` rollups for the Models / Providers / Status
//! tabs.
//!
//! Each takes a [`SubjectScope`] for the same reason the request log does:
//! these are aggregates over every user's traffic, narrowed to the callers's
//! visible users.
//!
//! One row per distinct model, provider, or status in the window, carrying the
//! same measures the KPI strip reports so a reader can attribute traffic,
//! spend, latency, and failures without leaving the tab. The error predicate
//! matches `view::is_error_status`, so a status row's `error_count` and the
//! table's danger badge can never disagree.
//!
//! `sqlx::query_as!` needs static SQL, so the grouping column cannot be a bind
//! parameter — hence three functions over one shared shape rather than a
//! `GROUP BY $1`.

use sqlx::PgPool;

use crate::repositories::scope::SubjectScope;
use crate::util::time_range::TimeRange;

#[derive(Debug, Clone)]
pub struct BreakdownRow {
    pub key: String,
    pub requests: i64,
    pub error_count: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost_microdollars: i64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
}

pub async fn list_requests_by_model(
    pool: &PgPool,
    range: TimeRange,
    scope: &SubjectScope,
) -> Result<Vec<BreakdownRow>, sqlx::Error> {
    sqlx::query_as!(
        BreakdownRow,
        r#"SELECT
             model AS "key!",
             COUNT(*)::bigint AS "requests!",
             COUNT(*) FILTER (WHERE status NOT IN ('completed', 'pending', 'streaming'))::bigint
               AS "error_count!",
             COALESCE(SUM(input_tokens), 0)::bigint AS "input_tokens!",
             COALESCE(SUM(output_tokens), 0)::bigint AS "output_tokens!",
             COALESCE(SUM(cost_microdollars), 0)::bigint AS "cost_microdollars!",
             COALESCE(percentile_cont(0.50) WITHIN GROUP (ORDER BY latency_ms), 0)::float8
               AS "p50_latency_ms!",
             COALESCE(percentile_cont(0.95) WITHIN GROUP (ORDER BY latency_ms), 0)::float8
               AS "p95_latency_ms!"
           FROM ai_requests ar
           WHERE ar.created_at >= $1 AND ar.created_at < $2
             AND ($3::TEXT[] IS NULL OR ar.user_id = ANY($3))
             AND model IS NOT NULL AND model <> ''
           GROUP BY model
           ORDER BY COUNT(*) DESC, model"#,
        range.from,
        range.to,
        scope.as_sql(),
    )
    .fetch_all(pool)
    .await
}

pub async fn list_requests_by_provider(
    pool: &PgPool,
    range: TimeRange,
    scope: &SubjectScope,
) -> Result<Vec<BreakdownRow>, sqlx::Error> {
    sqlx::query_as!(
        BreakdownRow,
        r#"SELECT
             provider AS "key!",
             COUNT(*)::bigint AS "requests!",
             COUNT(*) FILTER (WHERE status NOT IN ('completed', 'pending', 'streaming'))::bigint
               AS "error_count!",
             COALESCE(SUM(input_tokens), 0)::bigint AS "input_tokens!",
             COALESCE(SUM(output_tokens), 0)::bigint AS "output_tokens!",
             COALESCE(SUM(cost_microdollars), 0)::bigint AS "cost_microdollars!",
             COALESCE(percentile_cont(0.50) WITHIN GROUP (ORDER BY latency_ms), 0)::float8
               AS "p50_latency_ms!",
             COALESCE(percentile_cont(0.95) WITHIN GROUP (ORDER BY latency_ms), 0)::float8
               AS "p95_latency_ms!"
           FROM ai_requests ar
           WHERE ar.created_at >= $1 AND ar.created_at < $2
             AND ($3::TEXT[] IS NULL OR ar.user_id = ANY($3))
             AND provider IS NOT NULL AND provider <> ''
           GROUP BY provider
           ORDER BY COUNT(*) DESC, provider"#,
        range.from,
        range.to,
        scope.as_sql(),
    )
    .fetch_all(pool)
    .await
}

pub async fn list_requests_by_status(
    pool: &PgPool,
    range: TimeRange,
    scope: &SubjectScope,
) -> Result<Vec<BreakdownRow>, sqlx::Error> {
    sqlx::query_as!(
        BreakdownRow,
        r#"SELECT
             status AS "key!",
             COUNT(*)::bigint AS "requests!",
             COUNT(*) FILTER (WHERE status NOT IN ('completed', 'pending', 'streaming'))::bigint
               AS "error_count!",
             COALESCE(SUM(input_tokens), 0)::bigint AS "input_tokens!",
             COALESCE(SUM(output_tokens), 0)::bigint AS "output_tokens!",
             COALESCE(SUM(cost_microdollars), 0)::bigint AS "cost_microdollars!",
             COALESCE(percentile_cont(0.50) WITHIN GROUP (ORDER BY latency_ms), 0)::float8
               AS "p50_latency_ms!",
             COALESCE(percentile_cont(0.95) WITHIN GROUP (ORDER BY latency_ms), 0)::float8
               AS "p95_latency_ms!"
           FROM ai_requests ar
           WHERE ar.created_at >= $1 AND ar.created_at < $2
             AND ($3::TEXT[] IS NULL OR ar.user_id = ANY($3))
             AND status IS NOT NULL AND status <> ''
           GROUP BY status
           ORDER BY COUNT(*) DESC, status"#,
        range.from,
        range.to,
        scope.as_sql(),
    )
    .fetch_all(pool)
    .await
}
