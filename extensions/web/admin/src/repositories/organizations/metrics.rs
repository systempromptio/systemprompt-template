//! Per-organization headline figures: seats, footprint, spend, and margin.
//!
//! The enterprise console leads with a profit-and-loss line, so cost and
//! revenue are read together rather than from two places that could disagree.
//! Cost is `ai_requests.cost_microdollars` summed over the organization's
//! members; revenue is the plan's monthly licence fee. Both are microdollars,
//! which is the unit every price in the system is already accounted in, so the
//! subtraction needs no conversion and cannot lose a rounding step.
//!
//! Two windows are reported, and they answer different questions. The rolling
//! 30 days is the trend an operator reads across customers. Month-to-date is
//! what the budget guard enforces against the plan's cap, so the percentage
//! shown on screen is the same number that will produce the 429 — a dashboard
//! that disagreed with the enforcement point would be worse than no dashboard.

use sqlx::PgPool;
use systemprompt_web_shared::error::MarketplaceError;

use crate::authz::organization::organization_rule_type;

#[derive(Debug, Clone)]
pub struct OrganizationMetrics {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub plan_id: Option<String>,
    pub plan_name: Option<String>,
    pub status: String,
    pub is_platform: bool,
    pub seats_used: i64,
    /// The org override when set, otherwise the plan's; `None` is unlimited.
    pub seat_limit: Option<i32>,
    pub departments: i64,
    /// Allow rules the plan projected for this organization.
    pub entitlements: i64,
    pub requests_30d: i64,
    pub tokens_30d: i64,
    pub cost_microdollars_30d: i64,
    /// Spend since the start of the calendar month — what the budget guard
    /// compares against `cap_microdollars`.
    pub cost_microdollars_mtd: i64,
    /// `None` is an uncapped plan.
    pub cap_microdollars: Option<i64>,
    /// The plan's soft warning threshold; `None` is no warning configured.
    pub warn_microdollars: Option<i64>,
    /// The monthly licence fee. Zero is a non-billed plan.
    pub revenue_microdollars: i64,
}

impl OrganizationMetrics {
    /// Licence revenue less inference cost for the current month.
    #[must_use]
    pub const fn margin_microdollars(&self) -> i64 {
        self.revenue_microdollars - self.cost_microdollars_mtd
    }

    /// Month-to-date spend as a percentage of the plan's cap.
    ///
    /// `None` for an uncapped plan: an uncapped customer has no budget health
    /// to report, and rendering them at 0% would read as "plenty of headroom"
    /// rather than "not applicable".
    #[must_use]
    pub fn budget_used_pct(&self) -> Option<i64> {
        let cap = self.cap_microdollars.filter(|c| *c > 0)?;
        Some(self.cost_microdollars_mtd.saturating_mul(100) / cap)
    }
}

/// Every organization with its headline figures, most valuable first.
///
/// Ordering is by margin because that is the question the page exists to
/// answer. A customer burning more inference than their licence covers sorts
/// to the bottom, which is where an operator wants to find them.
pub async fn list_organization_metrics(
    pool: &PgPool,
) -> Result<Vec<OrganizationMetrics>, MarketplaceError> {
    let rule_type = organization_rule_type();
    let rows = sqlx::query!(
        r#"
        SELECT
            o.id AS "id!",
            o.slug AS "slug!",
            o.name AS "name!",
            o.plan_id,
            p.name AS "plan_name?",
            o.status AS "status!",
            o.is_platform AS "is_platform!",
            COALESCE(o.seat_limit_override, p.seat_limit) AS "seat_limit?",
            p.monthly_cost_cap_microdollars AS "cap?",
            p.monthly_cost_warn_microdollars AS "warn?",
            COALESCE(p.monthly_price_microdollars, 0) AS "revenue!",
            (SELECT COUNT(*) FROM organization_members m
               JOIN users u ON u.id = m.user_id
              WHERE m.org_id = o.id AND u.status = 'active') AS "seats_used!",
            (SELECT COUNT(*) FROM departments d WHERE d.org_id = o.id) AS "departments!",
            (SELECT COUNT(*) FROM access_control_rules r
              WHERE r.rule_type = $1 AND r.rule_value = o.slug AND r.access = 'allow')
                AS "entitlements!",
            usage.requests AS "requests!",
            usage.tokens AS "tokens!",
            usage.cost_30d AS "cost_30d!",
            usage.cost_mtd AS "cost_mtd!"
        FROM organizations o
        LEFT JOIN plans p ON p.id = o.plan_id
        LEFT JOIN LATERAL (
            SELECT
                COUNT(*)::BIGINT AS requests,
                COALESCE(SUM(r.input_tokens + r.output_tokens), 0)::BIGINT AS tokens,
                COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS cost_30d,
                COALESCE(SUM(r.cost_microdollars)
                    FILTER (WHERE r.created_at >= DATE_TRUNC('month', NOW())), 0)::BIGINT
                    AS cost_mtd
            FROM ai_requests r
            JOIN organization_members m ON m.user_id = r.user_id
            WHERE m.org_id = o.id
              AND r.created_at >= NOW() - INTERVAL '30 days'
        ) usage ON TRUE
        ORDER BY o.is_platform DESC, o.name
        "#,
        rule_type.as_str(),
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| OrganizationMetrics {
            id: r.id,
            slug: r.slug,
            name: r.name,
            plan_id: r.plan_id,
            plan_name: r.plan_name,
            status: r.status,
            is_platform: r.is_platform,
            seats_used: r.seats_used,
            seat_limit: r.seat_limit,
            departments: r.departments,
            entitlements: r.entitlements,
            requests_30d: r.requests,
            tokens_30d: r.tokens,
            cost_microdollars_30d: r.cost_30d,
            cost_microdollars_mtd: r.cost_mtd,
            cap_microdollars: r.cap,
            warn_microdollars: r.warn,
            revenue_microdollars: r.revenue,
        })
        .collect())
}

/// One organization's headline figures, by slug.
pub async fn find_organization_metrics(
    pool: &PgPool,
    slug: &str,
) -> Result<Option<OrganizationMetrics>, MarketplaceError> {
    Ok(list_organization_metrics(pool)
        .await?
        .into_iter()
        .find(|o| o.slug == slug))
}
