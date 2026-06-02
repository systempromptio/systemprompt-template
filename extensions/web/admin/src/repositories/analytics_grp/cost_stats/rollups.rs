//! Spend rollups — per-(provider, model) and per-provider, both ordered by
//! total spend.

use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::governance_grp::time_range::TimeRange;

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
