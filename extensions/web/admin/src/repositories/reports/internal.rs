//! The operator's provider cost for one calendar month.
//!
//! By supplier and by model, plus the daily series behind the chart. Every
//! figure is microdollars, the unit every price in the system is already
//! accounted in.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt_web_shared::error::MarketplaceError;

/// What we owe one upstream, or spent on one model, for the month.
#[derive(Debug, Clone)]
pub struct SupplierMonthCost {
    pub key: String,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
}

// Why: The supplier bill, by provider. Rejected requests never reached an
// upstream and carry no provider, so they are excluded rather than grouped as
// blank.
pub async fn list_provider_month_costs(
    pool: &PgPool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<SupplierMonthCost>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            r.provider AS "key!",
            COUNT(*)::BIGINT AS "requests!",
            COALESCE(SUM(r.input_tokens + r.output_tokens), 0)::BIGINT AS "tokens!",
            COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost!"
        FROM ai_requests r
        WHERE r.created_at >= $1 AND r.created_at < $2
          AND r.provider IS NOT NULL
        GROUP BY r.provider
        ORDER BY 4 DESC
        "#,
        from,
        to,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| SupplierMonthCost {
            key: r.key,
            requests: r.requests,
            tokens: r.tokens,
            cost_microdollars: r.cost,
        })
        .collect())
}

pub async fn list_model_month_costs(
    pool: &PgPool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<SupplierMonthCost>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            r.model AS "key!",
            COUNT(*)::BIGINT AS "requests!",
            COALESCE(SUM(r.input_tokens + r.output_tokens), 0)::BIGINT AS "tokens!",
            COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost!"
        FROM ai_requests r
        WHERE r.created_at >= $1 AND r.created_at < $2
          AND r.model IS NOT NULL
        GROUP BY r.model
        ORDER BY 4 DESC
        LIMIT 20
        "#,
        from,
        to,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| SupplierMonthCost {
            key: r.key,
            requests: r.requests,
            tokens: r.tokens,
            cost_microdollars: r.cost,
        })
        .collect())
}

/// Platform cost per month for the trailing `months`, oldest first, so the
/// trend chart reads left to right.
#[derive(Debug, Clone, Copy)]
pub struct PlatformMonthPoint {
    pub month_start: DateTime<Utc>,
    pub cost_microdollars: i64,
    pub requests: i64,
}

pub async fn list_platform_month_series(
    pool: &PgPool,
    months: i32,
) -> Result<Vec<PlatformMonthPoint>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            DATE_TRUNC('month', r.created_at) AS "month_start!",
            COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost!",
            COUNT(*)::BIGINT AS "requests!"
        FROM ai_requests r
        WHERE r.created_at >= DATE_TRUNC('month', NOW()) - ($1::INT * INTERVAL '1 month')
        GROUP BY 1
        ORDER BY 1
        "#,
        months,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| PlatformMonthPoint {
            month_start: r.month_start,
            cost_microdollars: r.cost,
            requests: r.requests,
        })
        .collect())
}
