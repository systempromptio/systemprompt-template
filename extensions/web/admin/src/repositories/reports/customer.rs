//! One organization's month, as the customer is allowed to see it.
//!
//! Nothing here selects `cost_microdollars`. What a customer pays is their
//! plan's licence fee, which is a contracted number; what we paid a provider to
//! serve them is our margin, and a report that carries both is one screenshot
//! away from being a pricing negotiation. Keeping the column out of the query —
//! rather than out of the template — means the guarantee survives someone
//! adding a field to the view-model later.
//!
//! Departments are resolved through `user_profile_ext.department`, which is
//! free text keyed by name rather than a foreign key, so an unset department
//! folds into `Default` instead of dropping the user's usage from the report.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt_web_shared::error::MarketplaceError;

/// The header figures: who they are, what they bought, what they used.
#[derive(Debug, Clone)]
pub struct CustomerMonthSummary {
    pub org_id: String,
    pub slug: String,
    pub name: String,
    pub plan_name: Option<String>,
    pub price_microdollars: i64,
    pub seat_limit: Option<i32>,
    pub seats_used: i64,
    // Why: Seats that actually made a request. The gap against `seats_used` is the
    // number a customer's own administrator is usually looking for.
    pub active_users: i64,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub error_count: i64,
}

impl CustomerMonthSummary {
    #[must_use]
    pub const fn total_tokens(&self) -> i64 {
        self.input_tokens + self.output_tokens
    }
}

pub async fn find_customer_month_summary(
    pool: &PgPool,
    org_id: &str,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Option<CustomerMonthSummary>, MarketplaceError> {
    let row = sqlx::query!(
        r#"
        SELECT
            o.id AS "org_id!",
            o.slug AS "slug!",
            o.name AS "name!",
            p.name AS "plan_name?",
            COALESCE(p.monthly_price_microdollars, 0) AS "price!",
            COALESCE(o.seat_limit_override, p.seat_limit) AS "seat_limit?",
            (SELECT COUNT(*) FROM organization_members m
               JOIN users u ON u.id = m.user_id
              WHERE m.org_id = o.id AND u.status = 'active') AS "seats_used!",
            usage.active_users AS "active_users!",
            usage.requests AS "requests!",
            usage.input_tokens AS "input_tokens!",
            usage.output_tokens AS "output_tokens!",
            usage.cache_read_tokens AS "cache_read_tokens!",
            usage.error_count AS "error_count!"
        FROM organizations o
        LEFT JOIN plans p ON p.id = o.plan_id
        LEFT JOIN LATERAL (
            SELECT
                COUNT(*)::BIGINT AS requests,
                COUNT(DISTINCT r.user_id)::BIGINT AS active_users,
                COALESCE(SUM(r.input_tokens), 0)::BIGINT AS input_tokens,
                COALESCE(SUM(r.output_tokens), 0)::BIGINT AS output_tokens,
                COALESCE(SUM(r.cache_read_tokens), 0)::BIGINT AS cache_read_tokens,
                COUNT(*) FILTER (WHERE r.status NOT IN ('success', 'completed'))::BIGINT
                    AS error_count
            FROM ai_requests r
            JOIN organization_members m ON m.user_id = r.user_id
            WHERE m.org_id = o.id
              AND r.created_at >= $2 AND r.created_at < $3
        ) usage ON TRUE
        WHERE o.id = $1
        "#,
        org_id,
        from,
        to,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| CustomerMonthSummary {
        org_id: r.org_id,
        slug: r.slug,
        name: r.name,
        plan_name: r.plan_name,
        price_microdollars: r.price,
        seat_limit: r.seat_limit,
        seats_used: r.seats_used,
        active_users: r.active_users,
        requests: r.requests,
        input_tokens: r.input_tokens,
        output_tokens: r.output_tokens,
        cache_read_tokens: r.cache_read_tokens,
        error_count: r.error_count,
    }))
}

/// One member's consumption for the month.
#[derive(Debug, Clone)]
pub struct CustomerUserUsage {
    pub email: String,
    pub display_name: String,
    pub department: String,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub distinct_models: i64,
}

// Why: Members with no activity are omitted: their row would be a line of
// zeroes, and on a large customer that is most of the table.
pub async fn list_customer_month_users(
    pool: &PgPool,
    org_id: &str,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<CustomerUserUsage>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            u.email AS "email!",
            COALESCE(NULLIF(u.display_name, ''), u.name) AS "display_name!",
            COALESCE(NULLIF(e.department, ''), 'Default') AS "department!",
            COUNT(*)::BIGINT AS "requests!",
            COALESCE(SUM(r.input_tokens), 0)::BIGINT AS "input_tokens!",
            COALESCE(SUM(r.output_tokens), 0)::BIGINT AS "output_tokens!",
            COUNT(DISTINCT r.model)::BIGINT AS "distinct_models!"
        FROM ai_requests r
        JOIN organization_members m ON m.user_id = r.user_id
        JOIN users u ON u.id = r.user_id
        LEFT JOIN user_profile_ext e ON e.user_id = r.user_id
        WHERE m.org_id = $1
          AND r.created_at >= $2 AND r.created_at < $3
        GROUP BY u.id, u.email, u.display_name, u.name, e.department
        ORDER BY COALESCE(SUM(r.input_tokens + r.output_tokens), 0) DESC, COUNT(*) DESC
        "#,
        org_id,
        from,
        to,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| CustomerUserUsage {
            email: r.email,
            display_name: r.display_name,
            department: r.department,
            requests: r.requests,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            distinct_models: r.distinct_models,
        })
        .collect())
}

/// One department's consumption for the month.
#[derive(Debug, Clone)]
pub struct CustomerDepartmentUsage {
    pub department: String,
    pub members: i64,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
}

pub async fn list_customer_month_departments(
    pool: &PgPool,
    org_id: &str,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<CustomerDepartmentUsage>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            COALESCE(NULLIF(e.department, ''), 'Default') AS "department!",
            COUNT(DISTINCT r.user_id)::BIGINT AS "members!",
            COUNT(*)::BIGINT AS "requests!",
            COALESCE(SUM(r.input_tokens), 0)::BIGINT AS "input_tokens!",
            COALESCE(SUM(r.output_tokens), 0)::BIGINT AS "output_tokens!"
        FROM ai_requests r
        JOIN organization_members m ON m.user_id = r.user_id
        LEFT JOIN user_profile_ext e ON e.user_id = r.user_id
        WHERE m.org_id = $1
          AND r.created_at >= $2 AND r.created_at < $3
        GROUP BY 1
        ORDER BY COALESCE(SUM(r.input_tokens + r.output_tokens), 0) DESC
        "#,
        org_id,
        from,
        to,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| CustomerDepartmentUsage {
            department: r.department,
            members: r.members,
            requests: r.requests,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
        })
        .collect())
}

/// One model's consumption for the month.
#[derive(Debug, Clone)]
pub struct CustomerModelUsage {
    pub provider: String,
    pub model: String,
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
}

pub async fn list_customer_month_models(
    pool: &PgPool,
    org_id: &str,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
) -> Result<Vec<CustomerModelUsage>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT
            COALESCE(r.provider, 'unrouted') AS "provider!",
            COALESCE(r.model, 'unrouted') AS "model!",
            COUNT(*)::BIGINT AS "requests!",
            COALESCE(SUM(r.input_tokens), 0)::BIGINT AS "input_tokens!",
            COALESCE(SUM(r.output_tokens), 0)::BIGINT AS "output_tokens!",
            COALESCE(SUM(r.cache_read_tokens), 0)::BIGINT AS "cache_read_tokens!"
        FROM ai_requests r
        JOIN organization_members m ON m.user_id = r.user_id
        WHERE m.org_id = $1
          AND r.created_at >= $2 AND r.created_at < $3
        GROUP BY 1, 2
        ORDER BY COALESCE(SUM(r.input_tokens + r.output_tokens), 0) DESC
        "#,
        org_id,
        from,
        to,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| CustomerModelUsage {
            provider: r.provider,
            model: r.model,
            requests: r.requests,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            cache_read_tokens: r.cache_read_tokens,
        })
        .collect())
}
