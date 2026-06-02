//! Single-row cost KPI strip — spend, requests, token totals, and derived
//! throughput / unit-cost ratios over a time range.

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
