//! Anomaly detection: per-(policy, decision) z-score against a 7-day baseline.

use serde::Serialize;
use sqlx::PgPool;

use super::time_range::TimeRange;

const Z_THRESHOLD: f64 = 2.0;
const BASELINE_DAYS: i64 = 7;

#[derive(Debug, Clone, Serialize)]
pub struct Anomaly {
    pub policy: String,
    pub decision: String,
    pub window_count: i64,
    pub baseline_mean: f64,
    pub baseline_stddev: f64,
    pub z_score: f64,
    pub affected_user_count: i64,
    pub affected_agent_count: i64,
}

/// Find (policy, decision) buckets in `range` whose count is at least
/// `Z_THRESHOLD` standard deviations above the daily baseline of the prior
/// `BASELINE_DAYS` days.
pub async fn fetch_decision_anomalies(
    pool: &PgPool,
    range: TimeRange,
) -> Result<Vec<Anomaly>, sqlx::Error> {
    let baseline_start = range.from - chrono::Duration::days(BASELINE_DAYS);
    run_anomaly_query(pool, range, baseline_start).await
}

#[expect(
    clippy::too_many_lines,
    reason = "body is one irreducible compile-time-checked query_as! SQL literal"
)]
async fn run_anomaly_query(
    pool: &PgPool,
    range: TimeRange,
    baseline_start: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<Anomaly>, sqlx::Error> {
    sqlx::query_as!(
        Anomaly,
        r#"WITH window_counts AS (
            SELECT policy, decision, COUNT(*)::bigint AS window_count
            FROM governance_decisions
            WHERE created_at >= $1 AND created_at < $2
            GROUP BY policy, decision
        ),
        daily_buckets AS (
            SELECT
                policy,
                decision,
                date_trunc('day', created_at) AS day,
                COUNT(*)::float8 AS day_count
            FROM governance_decisions
            WHERE created_at >= $3 AND created_at < $1
            GROUP BY policy, decision, date_trunc('day', created_at)
        ),
        baseline AS (
            SELECT
                policy,
                decision,
                AVG(day_count)::float8 AS mean,
                COALESCE(STDDEV_SAMP(day_count), 0)::float8 AS stddev
            FROM daily_buckets
            GROUP BY policy, decision
        ),
        affected AS (
            SELECT
                policy,
                decision,
                COUNT(DISTINCT user_id)::bigint AS user_count,
                COUNT(DISTINCT COALESCE(agent_id, ''))::bigint AS agent_count
            FROM governance_decisions
            WHERE created_at >= $1 AND created_at < $2
            GROUP BY policy, decision
        ),
        scored AS (
            SELECT
                w.policy,
                w.decision,
                w.window_count,
                COALESCE(b.mean, 0)::float8 AS baseline_mean,
                COALESCE(b.stddev, 0)::float8 AS baseline_stddev,
                CASE
                    WHEN COALESCE(b.stddev, 0) > 0
                        THEN ((w.window_count::float8 - b.mean) / b.stddev)
                    WHEN w.window_count::float8 > COALESCE(b.mean, 0)
                        THEN 999.0::float8
                    ELSE 0.0::float8
                END AS z_score,
                COALESCE(a.user_count, 0) AS user_count,
                COALESCE(a.agent_count, 0) AS agent_count
            FROM window_counts w
            LEFT JOIN baseline b ON b.policy = w.policy AND b.decision = w.decision
            LEFT JOIN affected a ON a.policy = w.policy AND a.decision = w.decision
        )
        SELECT
            policy AS "policy!",
            decision AS "decision!",
            window_count AS "window_count!",
            baseline_mean AS "baseline_mean!",
            baseline_stddev AS "baseline_stddev!",
            z_score AS "z_score!",
            user_count AS "affected_user_count!",
            agent_count AS "affected_agent_count!"
        FROM scored
        WHERE z_score >= $4
        ORDER BY z_score DESC, window_count DESC"#,
        range.from,
        range.to,
        baseline_start,
        Z_THRESHOLD,
    )
    .fetch_all(pool)
    .await
}
