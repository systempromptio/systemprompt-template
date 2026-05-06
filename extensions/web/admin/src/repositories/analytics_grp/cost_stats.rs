//! Cost-focused aggregations over `ai_requests` for `/admin/analytics/costs`.
//!
//! - [`fetch_cost_kpis`] — single-row KPI strip (spend, requests, tokens, throughput).
//! - [`fetch_cost_by_model`] — per-(provider, model) rollup ordered by spend.
//! - [`fetch_cost_by_provider`] — per-provider rollup ordered by spend.
//! - [`fetch_token_throughput_over_time`] — 24-bucket input/output token series.

use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::governance_grp::time_range::TimeRange;

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct CostKpis {
    pub requests: i64,
    pub total_cost_microdollars: i64,
    pub avg_cost_microdollars: f64,
    pub max_cost_microdollars: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
    pub distinct_models: i64,
    pub distinct_providers: i64,
    pub error_count: i64,
    pub tokens_per_minute: f64,
    pub cost_per_request_microdollars: f64,
    pub cost_per_1k_input_microdollars: f64,
    pub cost_per_1k_output_microdollars: f64,
}

pub async fn fetch_cost_kpis(pool: &PgPool, range: TimeRange) -> Result<CostKpis, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT
            COUNT(*)::bigint AS "requests!",
            COALESCE(SUM(cost_microdollars), 0)::bigint AS "total_cost!",
            COALESCE(AVG(cost_microdollars), 0)::float8 AS "avg_cost!",
            COALESCE(MAX(cost_microdollars), 0)::bigint AS "max_cost!",
            COALESCE(SUM(input_tokens), 0)::bigint AS "input_tokens!",
            COALESCE(SUM(output_tokens), 0)::bigint AS "output_tokens!",
            COUNT(DISTINCT model)::bigint AS "distinct_models!",
            COUNT(DISTINCT provider)::bigint AS "distinct_providers!",
            COUNT(*) FILTER (WHERE status NOT IN ('completed','pending','streaming'))::bigint
                AS "error_count!"
          FROM ai_requests
          WHERE created_at >= $1 AND created_at < $2"#,
        range.from,
        range.to,
    )
    .fetch_one(pool)
    .await?;

    let elapsed_minutes = (range.to - range.from).num_seconds() as f64 / 60.0;
    let total_tokens = row.input_tokens + row.output_tokens;
    let tokens_per_minute = if elapsed_minutes > 0.0 {
        total_tokens as f64 / elapsed_minutes
    } else {
        0.0
    };
    let cost_per_request_microdollars = if row.requests > 0 {
        row.total_cost as f64 / row.requests as f64
    } else {
        0.0
    };
    let cost_per_1k_input_microdollars = if row.input_tokens > 0 {
        row.total_cost as f64 * 1000.0 / row.input_tokens as f64
    } else {
        0.0
    };
    let cost_per_1k_output_microdollars = if row.output_tokens > 0 {
        row.total_cost as f64 * 1000.0 / row.output_tokens as f64
    } else {
        0.0
    };

    Ok(CostKpis {
        requests: row.requests,
        total_cost_microdollars: row.total_cost,
        avg_cost_microdollars: row.avg_cost,
        max_cost_microdollars: row.max_cost,
        input_tokens: row.input_tokens,
        output_tokens: row.output_tokens,
        total_tokens,
        distinct_models: row.distinct_models,
        distinct_providers: row.distinct_providers,
        error_count: row.error_count,
        tokens_per_minute,
        cost_per_request_microdollars,
        cost_per_1k_input_microdollars,
        cost_per_1k_output_microdollars,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelCostRow {
    pub provider: String,
    pub model: String,
    pub calls: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_cost_microdollars: i64,
    pub avg_cost_microdollars: f64,
    pub avg_latency_ms: f64,
    pub errors: i64,
}

pub async fn fetch_cost_by_model(
    pool: &PgPool,
    range: TimeRange,
) -> Result<Vec<ModelCostRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
            provider AS "provider!",
            model AS "model!",
            COUNT(*)::bigint AS "calls!",
            COALESCE(SUM(input_tokens), 0)::bigint AS "input_tokens!",
            COALESCE(SUM(output_tokens), 0)::bigint AS "output_tokens!",
            COALESCE(SUM(cost_microdollars), 0)::bigint AS "total_cost!",
            COALESCE(AVG(cost_microdollars), 0)::float8 AS "avg_cost!",
            COALESCE(AVG(latency_ms), 0)::float8 AS "avg_latency!",
            COUNT(*) FILTER (WHERE status NOT IN ('completed','pending','streaming'))::bigint
                AS "errors!"
          FROM ai_requests
          WHERE created_at >= $1 AND created_at < $2
          GROUP BY provider, model
          ORDER BY SUM(cost_microdollars) DESC NULLS LAST, COUNT(*) DESC"#,
        range.from,
        range.to,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ModelCostRow {
            provider: r.provider,
            model: r.model,
            calls: r.calls,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            total_cost_microdollars: r.total_cost,
            avg_cost_microdollars: r.avg_cost,
            avg_latency_ms: r.avg_latency,
            errors: r.errors,
        })
        .collect())
}

#[derive(Debug, Clone, Serialize)]
pub struct ProviderCostRow {
    pub provider: String,
    pub calls: i64,
    pub total_cost_microdollars: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub distinct_models: i64,
}

pub async fn fetch_cost_by_provider(
    pool: &PgPool,
    range: TimeRange,
) -> Result<Vec<ProviderCostRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
            provider AS "provider!",
            COUNT(*)::bigint AS "calls!",
            COALESCE(SUM(cost_microdollars), 0)::bigint AS "total_cost!",
            COALESCE(SUM(input_tokens), 0)::bigint AS "input_tokens!",
            COALESCE(SUM(output_tokens), 0)::bigint AS "output_tokens!",
            COUNT(DISTINCT model)::bigint AS "distinct_models!"
          FROM ai_requests
          WHERE created_at >= $1 AND created_at < $2
          GROUP BY provider
          ORDER BY SUM(cost_microdollars) DESC NULLS LAST"#,
        range.from,
        range.to,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ProviderCostRow {
            provider: r.provider,
            calls: r.calls,
            total_cost_microdollars: r.total_cost,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            distinct_models: r.distinct_models,
        })
        .collect())
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ThroughputBucket {
    pub bucket_index: i32,
    pub bucket_start: chrono::DateTime<chrono::Utc>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
}

const THROUGHPUT_BUCKETS: i32 = 24;

pub async fn fetch_token_throughput_over_time(
    pool: &PgPool,
    range: TimeRange,
) -> Result<Vec<ThroughputBucket>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"WITH params AS (
            SELECT $1::timestamptz AS lo,
                   $2::timestamptz AS hi,
                   $3::int          AS n
          ),
          edges AS (
            SELECT generate_series(0, (SELECT n FROM params))::int AS i,
                   (SELECT lo FROM params)
                   + (((SELECT hi FROM params) - (SELECT lo FROM params))
                      * generate_series(0, (SELECT n FROM params))::float8
                      / (SELECT n FROM params)::float8) AS edge_ts
          ),
          bucketed AS (
            SELECT
              GREATEST(LEAST(
                width_bucket(
                  EXTRACT(EPOCH FROM created_at)::float8,
                  EXTRACT(EPOCH FROM (SELECT lo FROM params))::float8,
                  EXTRACT(EPOCH FROM (SELECT hi FROM params))::float8,
                  (SELECT n FROM params)
                ),
                (SELECT n FROM params)), 1)::int AS bucket_index,
              input_tokens,
              output_tokens
            FROM ai_requests
            WHERE created_at >= (SELECT lo FROM params)
              AND created_at <  (SELECT hi FROM params)
          ),
          summed AS (
            SELECT bucket_index,
                   COALESCE(SUM(input_tokens), 0)::bigint AS input_tokens,
                   COALESCE(SUM(output_tokens), 0)::bigint AS output_tokens
            FROM bucketed
            GROUP BY bucket_index
          )
        SELECT
          e.i AS "bucket_index!",
          e.edge_ts AS "bucket_start!",
          COALESCE(s.input_tokens, 0)::bigint AS "input_tokens!",
          COALESCE(s.output_tokens, 0)::bigint AS "output_tokens!"
        FROM edges e
        LEFT JOIN summed s ON s.bucket_index = e.i + 1
        WHERE e.i < (SELECT n FROM params)
        ORDER BY e.i"#,
        range.from,
        range.to,
        THROUGHPUT_BUCKETS,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ThroughputBucket {
            bucket_index: r.bucket_index,
            bucket_start: r.bucket_start,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            total_tokens: r.input_tokens + r.output_tokens,
        })
        .collect())
}

#[derive(Debug, Clone, Serialize)]
pub struct RecentRequest {
    pub id: String,
    pub user_id: String,
    pub trace_id: Option<String>,
    pub session_id: Option<String>,
    pub context_id: Option<String>,
    pub display_name: Option<String>,
    pub department: Option<String>,
    pub model: String,
    pub status: String,
    pub error_message: Option<String>,
    pub cost_microdollars: i64,
    pub latency_ms: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn fetch_recent_requests(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<RecentRequest>, sqlx::Error> {
    sqlx::query_as!(
        RecentRequest,
        r#"SELECT r.id, r.user_id, r.trace_id, r.session_id, r.context_id, r.model, r.status,
                  r.error_message, r.cost_microdollars, r.latency_ms, r.created_at,
                  u.display_name, u.department
           FROM ai_requests r
           LEFT JOIN users u ON u.id = r.user_id
           ORDER BY r.created_at DESC
           LIMIT $1"#,
        limit
    )
    .fetch_all(pool)
    .await
}
