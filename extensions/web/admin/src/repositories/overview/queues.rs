//! The two queues the overview surfaces: calls held for a human decision, and
//! usage the anomaly detector flagged.
//!
//! Both are counted rather than listed in full. The overview's job is to say
//! that something is waiting and where to go; the pages behind them own the
//! detail.

use sqlx::PgPool;

pub use crate::repositories::analytics::site::anomalies::UsageAnomalyRow;

// Why: `usage_anomalies` has no resolved flag — the detector writes a row per
// metric per hourly window and nothing closes it — so "open" is a recency rule
// stated here rather than a column read from the table.
pub const ANOMALY_OPEN_HOURS: i64 = 24;

// Why: expired holds are excluded. The call one was holding has already
// failed, so counting it would send an admin to approve something that can no
// longer run.
pub async fn count_pending_approvals(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!"
           FROM approval_requests
           WHERE status = 'pending' AND expires_at > NOW()"#
    )
    .fetch_one(pool)
    .await?;
    Ok(count)
}

pub async fn list_open_anomalies(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<UsageAnomalyRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT metric AS "metric!", window_start AS "window_start!",
               observed AS "observed!", baseline AS "baseline!",
               detected_at AS "detected_at!"
        FROM usage_anomalies
        WHERE detected_at >= NOW() - make_interval(hours => $1)
        ORDER BY detected_at DESC
        LIMIT $2
        "#,
        i32::try_from(ANOMALY_OPEN_HOURS).unwrap_or(24),
        limit,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| UsageAnomalyRow {
            metric: r.metric,
            window_start: r.window_start,
            observed: r.observed,
            baseline: r.baseline,
            detected_at: r.detected_at,
        })
        .collect())
}

// Why: the bands the overview's severity badge paints. Twice the baseline is
// the detector's own idea of an anomaly worth a page, so it is red; a metric
// with no baseline at all is new traffic rather than a spike, and reads as a
// caution rather than a failure.
#[must_use]
pub fn anomaly_tone(observed: i64, baseline: i64) -> &'static str {
    if baseline <= 0 {
        return "warn";
    }
    let ratio = observed as f64 / baseline as f64;
    if ratio >= 2.0 {
        "err"
    } else if ratio >= 1.5 {
        "warn"
    } else {
        "muted"
    }
}
