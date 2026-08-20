//! Month-to-date spend against the plan's cap, for one user's organization.
//!
//! One query with two readers: the gateway budget guard, which refuses the
//! request that would cross the cap, and the enterprise console, which shows
//! how close a customer is to it. Kept in one place so the percentage on
//! screen and the ceiling in force cannot describe different months.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_shared::error::MarketplaceError;

/// Month-to-date spend against cap for the organization a user belongs to.
///
/// Shared with [`crate::gateway_org_budget`] so the cap the dashboard reports
/// and the cap the gateway enforces are one query. Only active organizations
/// with a capped plan produce a row; everything else is `None`, which every
/// caller reads as "no ceiling applies".
#[derive(Debug, Clone)]
pub struct OrganizationSpend {
    pub org_id: String,
    pub name: String,
    pub spent_microdollars: i64,
    pub cap_microdollars: i64,
    /// The plan's soft threshold; `None` means no warning is configured.
    /// The loader guarantees a set value is below the cap.
    pub warn_microdollars: Option<i64>,
}

pub async fn find_spend_for_user(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<OrganizationSpend>, MarketplaceError> {
    let row = sqlx::query!(
        r#"
        SELECT
            o.id AS "org_id!",
            o.name AS "name!",
            p.monthly_cost_cap_microdollars AS "cap!",
            p.monthly_cost_warn_microdollars AS "warn?",
            COALESCE((
                SELECT SUM(r.cost_microdollars)::BIGINT
                FROM ai_requests r
                JOIN organization_members peer ON peer.user_id = r.user_id
                WHERE peer.org_id = o.id
                  AND r.created_at >= DATE_TRUNC('month', NOW())
            ), 0) AS "spent!"
        FROM organization_members m
        JOIN organizations o ON o.id = m.org_id
        JOIN plans p ON p.id = o.plan_id
        WHERE m.user_id = $1
          AND o.status = 'active'
          AND p.monthly_cost_cap_microdollars IS NOT NULL
        "#,
        user_id.as_str()
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| OrganizationSpend {
        org_id: r.org_id,
        name: r.name,
        spent_microdollars: r.spent,
        cap_microdollars: r.cap,
        warn_microdollars: r.warn,
    }))
}
