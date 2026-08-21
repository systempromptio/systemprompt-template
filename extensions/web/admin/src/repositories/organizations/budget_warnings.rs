//! Soft-cap crossings, one row per organization per calendar month.
//!
//! Written fire-and-forget by the gateway budget guard when month-to-date
//! spend crosses the plan's warning threshold; read by the admin dashboard's
//! spend view. The guard never denies on a warning, and a failed write here
//! never blocks the request.

use sqlx::PgPool;
use systemprompt_web_shared::error::MarketplaceError;

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

#[derive(Debug, Clone)]
pub struct BudgetWarningHistoryRow {
    pub org_id: String,
    pub org_name: String,
    pub org_slug: String,
    pub month: chrono::NaiveDate,
    pub threshold_microdollars: i64,
    pub spent_microdollars: i64,
    pub first_seen_at: chrono::DateTime<chrono::Utc>,
    pub last_seen_at: chrono::DateTime<chrono::Utc>,
}

/// Soft-cap crossings for the trailing `months` calendar months, newest
/// first — the spend tab's history table. `org_slug` of `None` (platform
/// admin) lists every organization's.
pub async fn list_budget_warning_history(
    pool: &PgPool,
    org_slug: Option<&str>,
    months: i32,
) -> Result<Vec<BudgetWarningHistoryRow>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT w.org_id AS "org_id!", o.name AS "org_name!", o.slug AS "org_slug!",
               w.month AS "month!",
               w.threshold_microdollars AS "threshold!",
               w.spent_microdollars AS "spent!",
               w.first_seen_at AS "first_seen_at!",
               w.last_seen_at AS "last_seen_at!"
        FROM org_budget_warnings w
        JOIN organizations o ON o.id = w.org_id
        WHERE w.month >= DATE_TRUNC('month', NOW())::DATE - make_interval(months => $1::INT)
          AND ($2::TEXT IS NULL OR o.slug = $2)
        ORDER BY w.month DESC, o.name
        "#,
        months,
        org_slug,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| BudgetWarningHistoryRow {
            org_id: r.org_id,
            org_name: r.org_name,
            org_slug: r.org_slug,
            month: r.month,
            threshold_microdollars: r.threshold,
            spent_microdollars: r.spent,
            first_seen_at: r.first_seen_at,
            last_seen_at: r.last_seen_at,
        })
        .collect())
}
