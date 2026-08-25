//! Budget threshold crossings, one row per organization per month per kind.
//!
//! Written fire-and-forget by the gateway budget guard — `soft_cap` when
//! month-to-date spend crosses the plan's warning threshold, and
//! `forecast_overrun` when the linear month-end projection first exceeds the
//! hard cap — and read by the admin dashboard's spend view. The guard never
//! denies on a warning, and a failed write here never blocks the request.

use sqlx::PgPool;
use systemprompt_web_shared::error::MarketplaceError;

use crate::util::org_scope::OrgScope;

/// Which threshold event a warning row records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetWarningKind {
    SoftCap,
    ForecastOverrun,
}

impl BudgetWarningKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SoftCap => "soft_cap",
            Self::ForecastOverrun => "forecast_overrun",
        }
    }
}

// Why: the guard calls this on every request once spend is past the
// threshold, so "crossed" is true thousands of times a month while the event
// worth telling a human about happens once. The bool reports whether this call
// was the *first* crossing for the organization this month: `xmax = 0` is true
// only on the INSERT arm of an upsert, which is exactly that — cheaper and
// racier-proof than a SELECT-then-INSERT.
pub async fn upsert_org_budget_warning(
    pool: &PgPool,
    org_id: &str,
    kind: BudgetWarningKind,
    threshold_microdollars: i64,
    spent_microdollars: i64,
) -> Result<bool, MarketplaceError> {
    let row = sqlx::query!(
        r#"INSERT INTO org_budget_warnings
            (org_id, month, kind, threshold_microdollars, spent_microdollars)
         VALUES ($1, DATE_TRUNC('month', NOW())::DATE, $2, $3, $4)
         ON CONFLICT (org_id, month, kind) DO UPDATE SET
            threshold_microdollars = EXCLUDED.threshold_microdollars,
            spent_microdollars = EXCLUDED.spent_microdollars,
            last_seen_at = NOW()
         RETURNING (xmax = 0) AS "first_crossing!""#,
        org_id,
        kind.as_str(),
        threshold_microdollars,
        spent_microdollars,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.first_crossing)
}

#[derive(Debug, Clone)]
pub struct BudgetWarningHistoryRow {
    pub org_id: String,
    pub org_name: String,
    pub org_slug: String,
    pub kind: String,
    pub month: chrono::NaiveDate,
    pub threshold_microdollars: i64,
    pub spent_microdollars: i64,
    pub first_seen_at: chrono::DateTime<chrono::Utc>,
    pub last_seen_at: chrono::DateTime<chrono::Utc>,
}

// Why: Soft-cap crossings for the trailing `months` calendar months, newest
// first — the spend tab's history table.
pub async fn list_budget_warning_history(
    pool: &PgPool,
    scope: &OrgScope,
    months: i32,
) -> Result<Vec<BudgetWarningHistoryRow>, MarketplaceError> {
    let rows = sqlx::query!(
        r#"
        SELECT w.org_id AS "org_id!", o.name AS "org_name!", o.slug AS "org_slug!",
               w.kind AS "kind!",
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
        scope.as_slug(),
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| BudgetWarningHistoryRow {
            org_id: r.org_id,
            org_name: r.org_name,
            org_slug: r.org_slug,
            kind: r.kind,
            month: r.month,
            threshold_microdollars: r.threshold,
            spent_microdollars: r.spent,
            first_seen_at: r.first_seen_at,
            last_seen_at: r.last_seen_at,
        })
        .collect())
}
