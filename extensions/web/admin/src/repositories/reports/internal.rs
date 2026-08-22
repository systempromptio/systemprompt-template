//! The operator's profit-and-loss for one calendar month.
//!
//! Revenue is the plan's monthly licence fee; cost is what the organization's
//! members actually spent at the providers. Both are microdollars, the unit
//! every price in the system is already accounted in, so the subtraction needs
//! no conversion step that could round a margin into existence.
//!
//! This mirrors `organizations::metrics::list_organization_metrics`, which
//! answers the same question over a rolling thirty days for the live console.
//! The duplication is deliberate: a dashboard wants "recently", a report wants
//! "in March", and collapsing the two would make one of them lie.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt_web_shared::error::MarketplaceError;

/// One organization's month, with the commercial terms it was served under.
#[derive(Debug, Clone)]
pub struct OrganizationMonthPnl {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub status: String,
    // Why: The operator's own tenant. Its spend is real and belongs in the cost
    // total, but it bills nobody, so it must not dilute the margin.
    pub is_platform: bool,
    pub plan_name: Option<String>,
    pub revenue_microdollars: i64,
    pub cap_microdollars: Option<i64>,
    pub seat_limit: Option<i32>,
    pub seats_used: i64,
    pub active_users: i64,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
}

impl OrganizationMonthPnl {
    #[must_use]
    pub const fn margin_microdollars(&self) -> i64 {
        self.revenue_microdollars - self.cost_microdollars
    }

    // Why: Margin as a percentage of revenue. `None` on a non-billed plan, where
    // the ratio is undefined rather than zero.
    #[must_use]
    pub const fn margin_pct(&self) -> Option<i64> {
        if self.revenue_microdollars <= 0 {
            return None;
        }
        Some(self.margin_microdollars().saturating_mul(100) / self.revenue_microdollars)
    }

    // Why: What one seat cost to serve. The figure that says whether a per-seat
    // price is holding as a customer grows into their plan.
    #[must_use]
    pub const fn cost_per_seat_microdollars(&self) -> i64 {
        if self.seats_used <= 0 {
            return 0;
        }
        self.cost_microdollars / self.seats_used
    }

    // Why: Month spend against the plan's cap. `None` is an uncapped plan, which
    // must not render as 0% — that reads as headroom rather than as N/A.
    #[must_use]
    pub fn budget_used_pct(&self) -> Option<i64> {
        let cap = self.cap_microdollars.filter(|c| *c > 0)?;
        Some(self.cost_microdollars.saturating_mul(100) / cap)
    }
}

pub async fn list_organization_month_pnl(
    pool: &PgPool,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<OrganizationMonthPnl>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            o.id AS "id!",
            o.slug AS "slug!",
            o.name AS "name!",
            o.status AS "status!",
            o.is_platform AS "is_platform!",
            p.name AS "plan_name?",
            COALESCE(p.monthly_price_microdollars, 0) AS "revenue!",
            p.monthly_cost_cap_microdollars AS "cap?",
            COALESCE(o.seat_limit_override, p.seat_limit) AS "seat_limit?",
            (SELECT COUNT(*) FROM organization_members m
               JOIN users u ON u.id = m.user_id
              WHERE m.org_id = o.id AND u.status = 'active') AS "seats_used!",
            usage.active_users AS "active_users!",
            usage.requests AS "requests!",
            usage.tokens AS "tokens!",
            usage.cost AS "cost!"
        FROM organizations o
        LEFT JOIN plans p ON p.id = o.plan_id
        LEFT JOIN LATERAL (
            SELECT
                COUNT(*)::BIGINT AS requests,
                COUNT(DISTINCT r.user_id)::BIGINT AS active_users,
                COALESCE(SUM(r.input_tokens + r.output_tokens), 0)::BIGINT AS tokens,
                COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS cost
            FROM ai_requests r
            JOIN organization_members m ON m.user_id = r.user_id
            WHERE m.org_id = o.id
              AND r.created_at >= $1 AND r.created_at < $2
        ) usage ON TRUE
        ORDER BY o.is_platform DESC, o.name
        "#,
        from,
        to,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| OrganizationMonthPnl {
            id: r.id,
            slug: r.slug,
            name: r.name,
            status: r.status,
            is_platform: r.is_platform,
            plan_name: r.plan_name,
            revenue_microdollars: r.revenue,
            cap_microdollars: r.cap,
            seat_limit: r.seat_limit,
            seats_used: r.seats_used,
            active_users: r.active_users,
            requests: r.requests,
            tokens: r.tokens,
            cost_microdollars: r.cost,
        })
        .collect())
}

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
