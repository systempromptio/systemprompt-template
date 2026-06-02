//! Sliding-window aggregates feeding the anomaly-detection baseline.

use sqlx::PgPool;

use crate::types::WindowedCounts;

pub async fn fetch_windowed_counts(
    pool: &PgPool,
    window_seconds: i64,
) -> Result<WindowedCounts, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT
            COUNT(*)::bigint AS "decisions!",
            COUNT(*) FILTER (WHERE decision = 'deny')::bigint AS "denied!",
            COUNT(*) FILTER (
                WHERE decision = 'deny'
                  AND (policy = 'secret_scan' OR reason ILIKE '%secret%')
            )::bigint AS "secret_blocks!",
            COUNT(DISTINCT user_id)::bigint AS "distinct_actors!"
        FROM governance_decisions
        WHERE created_at > now() - make_interval(secs => $1::double precision)"#,
        window_seconds as f64,
    )
    .fetch_one(pool)
    .await?;
    Ok(WindowedCounts {
        decisions: row.decisions,
        denied: row.denied,
        secret_blocks: row.secret_blocks,
        distinct_actors: row.distinct_actors,
    })
}

/// Returns one row per matching window across the last `lookback_days`,
/// excluding the live window. Caller computes mean/stddev for σ-deviation.
pub async fn fetch_baseline_window_samples(
    pool: &PgPool,
    window_seconds: i64,
    lookback_days: i64,
) -> Result<Vec<WindowedCounts>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"WITH live_start AS (
            SELECT now() - make_interval(secs => $1::double precision) AS ts
        ),
        buckets AS (
            SELECT generate_series(
                date_trunc('minute', now() - make_interval(days => $2::int)),
                (SELECT ts FROM live_start) - make_interval(secs => $1::double precision),
                make_interval(secs => $1::double precision)
            ) AS bucket_start
        )
        SELECT
            COALESCE(COUNT(g.id), 0)::bigint AS "decisions!",
            COALESCE(COUNT(g.id) FILTER (WHERE g.decision = 'deny'), 0)::bigint AS "denied!",
            COALESCE(COUNT(g.id) FILTER (
                WHERE g.decision = 'deny'
                  AND (g.policy = 'secret_scan' OR g.reason ILIKE '%secret%')
            ), 0)::bigint AS "secret_blocks!",
            COALESCE(COUNT(DISTINCT g.user_id), 0)::bigint AS "distinct_actors!"
        FROM buckets b
        LEFT JOIN governance_decisions g
            ON g.created_at >= b.bucket_start
           AND g.created_at <  b.bucket_start + make_interval(secs => $1::double precision)
        GROUP BY b.bucket_start"#,
        window_seconds as f64,
        lookback_days as i32,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| WindowedCounts {
            decisions: r.decisions,
            denied: r.denied,
            secret_blocks: r.secret_blocks,
            distinct_actors: r.distinct_actors,
        })
        .collect())
}
