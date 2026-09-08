//! Recent usage anomalies for the spend dashboard, written by the
//! `usage_anomaly` job.
//!
//! Instance-wide across *users* by design: the detector compares whole-gateway
//! traffic against its own baseline, so narrowing these rows to a project would
//! claim a precision the data does not have.
//!
//! The time window is a different matter and is honoured. Every other figure on
//! the page is bounded by the window the reader chose, and a table of anomalies
//! from outside it, sitting under that window's heading, reads as though they
//! happened inside it.

use sqlx::PgPool;

use crate::util::time_range::TimeRange;

#[derive(Debug, Clone)]
pub struct UsageAnomalyRow {
    pub metric: String,
    pub window_start: chrono::DateTime<chrono::Utc>,
    pub observed: i64,
    pub baseline: i64,
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

pub async fn list_recent_anomalies(
    pool: &PgPool,
    range: TimeRange,
    limit: i64,
) -> Result<Vec<UsageAnomalyRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT metric AS "metric!", window_start AS "window_start!",
               observed AS "observed!", baseline AS "baseline!",
               detected_at AS "detected_at!"
        FROM usage_anomalies
        WHERE window_start >= $1 AND window_start < $2
        ORDER BY detected_at DESC
        LIMIT $3
        "#,
        range.from,
        range.to,
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
