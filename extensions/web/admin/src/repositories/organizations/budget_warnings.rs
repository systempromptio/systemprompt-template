//! Soft-cap crossings, one row per organization per calendar month.
//!
//! Written fire-and-forget by the gateway budget guard when month-to-date
//! spend crosses the plan's warning threshold; read by the admin dashboard's
//! spend view. The guard never denies on a warning, and a failed write here
//! never blocks the request.

use sqlx::PgPool;
use systemprompt_web_shared::error::MarketplaceError;

#[derive(Debug, Clone)]
pub struct OrgBudgetWarning {
    pub org_id: String,
    pub month: chrono::NaiveDate,
    pub threshold_microdollars: i64,
    pub spent_microdollars: i64,
    pub first_seen_at: chrono::DateTime<chrono::Utc>,
    pub last_seen_at: chrono::DateTime<chrono::Utc>,
}

pub async fn upsert_org_budget_warning(
    pool: &PgPool,
    org_id: &str,
    threshold_microdollars: i64,
    spent_microdollars: i64,
) -> Result<(), MarketplaceError> {
    sqlx::query!(
        "INSERT INTO org_budget_warnings
            (org_id, month, threshold_microdollars, spent_microdollars)
         VALUES ($1, DATE_TRUNC('month', NOW())::DATE, $2, $3)
         ON CONFLICT (org_id, month) DO UPDATE SET
            threshold_microdollars = EXCLUDED.threshold_microdollars,
            spent_microdollars = EXCLUDED.spent_microdollars,
            last_seen_at = NOW()",
        org_id,
        threshold_microdollars,
        spent_microdollars,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Warnings for the current calendar month, keyed by organization id.
pub async fn list_current_month_warnings(
    pool: &PgPool,
) -> Result<Vec<OrgBudgetWarning>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT org_id AS "org_id!", month AS "month!",
               threshold_microdollars AS "threshold!",
               spent_microdollars AS "spent!",
               first_seen_at AS "first_seen_at!",
               last_seen_at AS "last_seen_at!"
        FROM org_budget_warnings
        WHERE month = DATE_TRUNC('month', NOW())::DATE
        "#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| OrgBudgetWarning {
            org_id: r.org_id,
            month: r.month,
            threshold_microdollars: r.threshold,
            spent_microdollars: r.spent,
            first_seen_at: r.first_seen_at,
            last_seen_at: r.last_seen_at,
        })
        .collect())
}
