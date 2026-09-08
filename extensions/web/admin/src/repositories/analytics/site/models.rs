//! Per-model gateway statistics for the Models tab.
//!
//! One row per model actually served, plus one `unrouted` row standing for
//! every request the gateway rejected before a route was chosen — those carry
//! a NULL `model` and a NULL `provider`, and dropping them would hide exactly
//! the traffic an operator most wants to see.
//!
//! `requested_model` is what the client asked for and `model` is what the
//! route resolved to, so the two differing is a redirect. The count is
//! reported per model rather than summed, because a redirect only means
//! anything beside the model it landed on.

use sqlx::PgPool;

use crate::util::time_range::TimeRange;

use super::SiteScope;

// Why: the statuses that mean the request produced no answer. `success` is a
// completed request under the MCP-side naming, so excluding it here is what
// stops two thirds of healthy traffic being counted as failures.
pub const ERROR_STATUSES: [&str; 4] = ["failed", "rejected", "timeout", "error"];

#[derive(Debug, Clone)]
pub struct ModelStatsRow {
    pub model: String,
    pub provider: Option<String>,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_tokens: i64,
    pub reasoning_tokens: i64,
    pub cost_microdollars: i64,
    pub p50_latency_ms: Option<f64>,
    pub p95_latency_ms: Option<f64>,
    pub errors: i64,
    // Why: requests whose `requested_model` differs from the model that served
    // them — the gateway route rewrote the target.
    pub redirected: i64,
    // Why: the bucket for requests with no model at all, which are the ones
    // rejected before a route was chosen.
    pub is_unrouted: bool,
}

pub async fn list_model_stats(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<Vec<ModelStatsRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            COALESCE(r.model, 'unrouted') AS "model!",
            MIN(r.provider) AS provider,
            COUNT(*)::BIGINT AS "requests!",
            COALESCE(SUM(r.input_tokens), 0)::BIGINT AS "input_tokens!",
            COALESCE(SUM(r.output_tokens), 0)::BIGINT AS "output_tokens!",
            COALESCE(SUM(COALESCE(r.cache_read_tokens, 0)
                       + COALESCE(r.cache_creation_tokens, 0)), 0)::BIGINT AS "cache_tokens!",
            COALESCE(SUM(r.reasoning_tokens), 0)::BIGINT AS "reasoning_tokens!",
            COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost!",
            percentile_cont(0.5) WITHIN GROUP (ORDER BY r.latency_ms) AS p50,
            percentile_cont(0.95) WITHIN GROUP (ORDER BY r.latency_ms) AS p95,
            COUNT(*) FILTER (WHERE r.status = ANY($5::TEXT[]))::BIGINT AS "errors!",
            COUNT(*) FILTER (
                WHERE r.requested_model IS NOT NULL
                  AND r.model IS NOT NULL
                  AND r.requested_model <> r.model
            )::BIGINT AS "redirected!",
            bool_and(r.model IS NULL) AS "is_unrouted!"
        FROM ai_requests r
        WHERE r.created_at >= $1 AND r.created_at < $2
          AND NOT r.synthetic
          AND ($3::TEXT[] IS NULL OR r.user_id = ANY($3))
          AND ($4::TEXT IS NULL OR r.user_id = $4)
        GROUP BY COALESCE(r.model, 'unrouted')
        ORDER BY COUNT(*) DESC
        LIMIT 200
        "#,
        range.from,
        range.to,
        scope.scope.as_sql(),
        scope.user_id_str(),
        &ERROR_STATUSES.map(str::to_owned)[..],
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ModelStatsRow {
            model: r.model,
            provider: r.provider,
            requests: r.requests,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            cache_tokens: r.cache_tokens,
            reasoning_tokens: r.reasoning_tokens,
            cost_microdollars: r.cost,
            p50_latency_ms: r.p50,
            p95_latency_ms: r.p95,
            errors: r.errors,
            redirected: r.redirected,
            is_unrouted: r.is_unrouted,
        })
        .collect())
}

/// Every requested-model → served-model pair the window contains, so the page
/// can name what a redirect actually did rather than only counting it.
#[derive(Debug, Clone)]
pub struct ModelRedirectRow {
    pub requested_model: String,
    pub served_model: String,
    pub requests: i64,
}

pub async fn list_model_redirects(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<Vec<ModelRedirectRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            r.requested_model AS "requested_model!",
            r.model AS "served_model!",
            COUNT(*)::BIGINT AS "requests!"
        FROM ai_requests r
        WHERE r.created_at >= $1 AND r.created_at < $2
          AND NOT r.synthetic
          AND r.requested_model IS NOT NULL
          AND r.model IS NOT NULL
          AND r.requested_model <> r.model
          AND ($3::TEXT[] IS NULL OR r.user_id = ANY($3))
          AND ($4::TEXT IS NULL OR r.user_id = $4)
        GROUP BY r.requested_model, r.model
        ORDER BY COUNT(*) DESC
        LIMIT 50
        "#,
        range.from,
        range.to,
        scope.scope.as_sql(),
        scope.user_id_str(),
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ModelRedirectRow {
            requested_model: r.requested_model,
            served_model: r.served_model,
            requests: r.requests,
        })
        .collect())
}
