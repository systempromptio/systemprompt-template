//! Recent usage anomalies for the spend dashboard, written by the
//! `usage_anomaly` job.
//!
//! Instance-wide by design: the detector compares whole-gateway traffic
//! against its own baseline, so scoping these rows to an organization would
//! claim a precision the data does not have.

use sqlx::PgPool;

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
    limit: i64,
) -> Result<Vec<UsageAnomalyRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT metric AS "metric!", window_start AS "window_start!",
               observed AS "observed!", baseline AS "baseline!",
               detected_at AS "detected_at!"
        FROM usage_anomalies
        ORDER BY detected_at DESC
        LIMIT $1
        "#,
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
