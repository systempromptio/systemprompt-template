//! Aggregated `ai_requests` statistics for the Inference Requests page.
//!
//! Three helpers:
//! - [`fetch_request_stats`] — overall KPI strip (rate / latency percentiles /
//!   cost / error rate / pre-flight deny rate over a [`TimeRange`]).
//! - [`fetch_latency_histogram`] — bucketed at fixed bin edges.
//! - [`fetch_cost_over_time`] — 24-bucket cost time series.

use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::governance_grp::time_range::TimeRange;

/// Fixed latency-histogram bin edges (ms). The final bin is open-ended.
pub const LATENCY_BIN_EDGES_MS: [f64; 8] =
    [50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0, 10_000.0];

/// KPI strip for the page header.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct RequestStats {
    pub total: i64,
    pub error_count: i64,
    pub requests_per_minute: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub total_cost_microdollars: i64,
    pub error_rate: f64,
    pub denied_session_count: i64,
    pub denied_session_rate: f64,
}

pub async fn fetch_request_stats(
    pool: &PgPool,
    range: TimeRange,
) -> Result<RequestStats, sqlx::Error> {
    let row = sqlx::query!(
        r#"WITH
            requests AS (
                SELECT id, status, latency_ms, cost_microdollars, session_id, created_at
                FROM ai_requests
                WHERE created_at >= $1 AND created_at < $2
            ),
            with_deny AS (
                SELECT DISTINCT r.session_id
                FROM requests r
                JOIN governance_decisions g
                  ON g.session_id = r.session_id
                 AND g.decision = 'deny'
                WHERE r.session_id IS NOT NULL
            )
        SELECT
            COUNT(*)::bigint AS "total!",
            COUNT(*) FILTER (WHERE status NOT IN ('completed', 'pending', 'streaming'))::bigint
                AS "error_count!",
            COALESCE(percentile_cont(0.50) WITHIN GROUP (ORDER BY latency_ms), 0)::float8
                AS "p50!",
            COALESCE(percentile_cont(0.95) WITHIN GROUP (ORDER BY latency_ms), 0)::float8
                AS "p95!",
            COALESCE(percentile_cont(0.99) WITHIN GROUP (ORDER BY latency_ms), 0)::float8
                AS "p99!",
            COALESCE(SUM(cost_microdollars), 0)::bigint AS "total_cost!",
            (SELECT COUNT(*) FROM with_deny)::bigint AS "denied_sessions!",
            COUNT(DISTINCT session_id) FILTER (WHERE session_id IS NOT NULL)::bigint
                AS "distinct_sessions!"
        FROM requests"#,
        range.from,
        range.to,
    )
    .fetch_one(pool)
    .await?;

    let elapsed_minutes = (range.to - range.from).num_seconds() as f64 / 60.0;
    let requests_per_minute = if elapsed_minutes > 0.0 {
        row.total as f64 / elapsed_minutes
    } else {
        0.0
    };
    let error_rate = if row.total > 0 {
        row.error_count as f64 / row.total as f64
    } else {
        0.0
    };
    let denied_session_rate = if row.distinct_sessions > 0 {
        row.denied_sessions as f64 / row.distinct_sessions as f64
    } else {
        0.0
    };

    Ok(RequestStats {
        total: row.total,
        error_count: row.error_count,
        requests_per_minute,
        p50_latency_ms: row.p50,
        p95_latency_ms: row.p95,
        p99_latency_ms: row.p99,
        total_cost_microdollars: row.total_cost,
        error_rate,
        denied_session_count: row.denied_sessions,
        denied_session_rate,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct LatencyBucket {
    pub label: String,
    pub upper_bound_ms: Option<f64>,
    pub count: i64,
}

pub async fn fetch_latency_histogram(
    pool: &PgPool,
    range: TimeRange,
) -> Result<Vec<LatencyBucket>, sqlx::Error> {
    let edges = &LATENCY_BIN_EDGES_MS;
    let edges_pg: Vec<f64> = edges.to_vec();

    let rows = sqlx::query!(
        r#"SELECT
            width_bucket(latency_ms::float8, $1::float8[]) AS "bucket!",
            COUNT(*)::bigint AS "count!"
          FROM ai_requests
          WHERE created_at >= $2 AND created_at < $3
            AND latency_ms IS NOT NULL
          GROUP BY 1
          ORDER BY 1"#,
        &edges_pg,
        range.from,
        range.to,
    )
    .fetch_all(pool)
    .await?;

    let mut by_bucket: std::collections::HashMap<i32, i64> = std::collections::HashMap::new();
    for r in rows {
        by_bucket.insert(r.bucket, r.count);
    }

    let mut out = Vec::with_capacity(edges.len() + 1);
    for (i, &edge) in edges.iter().enumerate() {
        let bucket = (i + 1) as i32;
        let prev = if i == 0 { 0.0 } else { edges[i - 1] };
        let label = format!("{}–{} ms", format_ms(prev), format_ms(edge));
        out.push(LatencyBucket {
            label,
            upper_bound_ms: Some(edge),
            count: by_bucket.get(&bucket).copied().unwrap_or(0),
        });
    }
    let last_bucket = (edges.len() + 1) as i32;
    out.push(LatencyBucket {
        label: format!("{}+ ms", format_ms(edges[edges.len() - 1])),
        upper_bound_ms: None,
        count: by_bucket.get(&last_bucket).copied().unwrap_or(0),
    });
    Ok(out)
}

fn format_ms(v: f64) -> String {
    if v >= 1000.0 {
        format!("{}s", v as i64 / 1000)
    } else {
        format!("{}", v as i64)
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct CostBucket {
    pub bucket_index: i32,
    pub bucket_start: chrono::DateTime<chrono::Utc>,
    pub cost_microdollars: i64,
}

const COST_BUCKETS: i32 = 24;

pub async fn fetch_cost_over_time(
    pool: &PgPool,
    range: TimeRange,
) -> Result<Vec<CostBucket>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"WITH params AS (
            SELECT $1::timestamptz AS lo,
                   $2::timestamptz AS hi,
                   $3::int          AS n
          ),
          edges AS (
            SELECT generate_series(0, (SELECT n FROM params))::int AS i,
                   (SELECT lo FROM params)
                   + (((SELECT hi FROM params) - (SELECT lo FROM params))
                      * generate_series(0, (SELECT n FROM params))::float8
                      / (SELECT n FROM params)::float8) AS edge_ts
          ),
          bucketed AS (
            SELECT
              GREATEST(LEAST(
                width_bucket(
                  EXTRACT(EPOCH FROM created_at)::float8,
                  EXTRACT(EPOCH FROM (SELECT lo FROM params))::float8,
                  EXTRACT(EPOCH FROM (SELECT hi FROM params))::float8,
                  (SELECT n FROM params)
                ),
                (SELECT n FROM params)), 1)::int AS bucket_index,
              cost_microdollars
            FROM ai_requests
            WHERE created_at >= (SELECT lo FROM params)
              AND created_at <  (SELECT hi FROM params)
          ),
          summed AS (
            SELECT bucket_index, SUM(cost_microdollars)::bigint AS cost
            FROM bucketed
            GROUP BY bucket_index
          )
        SELECT
          e.i AS "bucket_index!",
          e.edge_ts AS "bucket_start!",
          COALESCE(s.cost, 0)::bigint AS "cost!"
        FROM edges e
        LEFT JOIN summed s ON s.bucket_index = e.i + 1
        WHERE e.i < (SELECT n FROM params)
        ORDER BY e.i"#,
        range.from,
        range.to,
        COST_BUCKETS,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| CostBucket {
            bucket_index: r.bucket_index,
            bucket_start: r.bucket_start,
            cost_microdollars: r.cost,
        })
        .collect())
}
