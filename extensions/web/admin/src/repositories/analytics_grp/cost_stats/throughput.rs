//! 24-bucket input/output token throughput series over a time range.

use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::governance_grp::time_range::TimeRange;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ThroughputBucket {
    pub bucket_index: i32,
    pub bucket_start: chrono::DateTime<chrono::Utc>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub total_tokens: i64,
}

const THROUGHPUT_BUCKETS: i32 = 24;

pub async fn fetch_token_throughput_over_time(
    pool: &PgPool,
    range: TimeRange,
) -> Result<Vec<ThroughputBucket>, sqlx::Error> {
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
              input_tokens,
              output_tokens
            FROM ai_requests
            WHERE created_at >= (SELECT lo FROM params)
              AND created_at <  (SELECT hi FROM params)
          ),
          summed AS (
            SELECT bucket_index,
                   COALESCE(SUM(input_tokens), 0)::bigint AS input_tokens,
                   COALESCE(SUM(output_tokens), 0)::bigint AS output_tokens
            FROM bucketed
            GROUP BY bucket_index
          )
        SELECT
          e.i AS "bucket_index!",
          e.edge_ts AS "bucket_start!",
          COALESCE(s.input_tokens, 0)::bigint AS "input_tokens!",
          COALESCE(s.output_tokens, 0)::bigint AS "output_tokens!"
        FROM edges e
        LEFT JOIN summed s ON s.bucket_index = e.i + 1
        WHERE e.i < (SELECT n FROM params)
        ORDER BY e.i"#,
        range.from,
        range.to,
        THROUGHPUT_BUCKETS,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ThroughputBucket {
            bucket_index: r.bucket_index,
            bucket_start: r.bucket_start,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            total_tokens: r.input_tokens + r.output_tokens,
        })
        .collect())
}
