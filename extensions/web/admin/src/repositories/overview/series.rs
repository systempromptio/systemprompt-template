//! The request spine behind the overview's sparkline.
//!
//! A fixed bucket count rather than a fixed bucket width: the sparkline is the
//! same 24-point shape whichever window is picked, so the tile's height never
//! depends on how many hours the reader asked for. At the 24-hour window a
//! bucket is exactly one hour, which is what the tile's caption says.

use sqlx::PgPool;

use crate::util::time_range::TimeRange;

// Why: how many points the sparkline draws. Fixed, so the tile's shape does
// not change height with the window the reader picked.
pub const BUCKETS: i64 = 24;

pub async fn list_request_buckets(
    pool: &PgPool,
    range: TimeRange,
) -> Result<Vec<i64>, sqlx::Error> {
    let span_secs = (range.to - range.from).num_seconds().max(BUCKETS);
    // Why: bucket width in seconds, floored at one so a degenerate range
    // cannot divide by zero in the statement below.
    let width = ((span_secs as f64) / (BUCKETS as f64)).max(1.0);
    let rows = sqlx::query!(
        r#"
        WITH spine AS (SELECT generate_series(0, $3::INT - 1) AS bucket),
             counts AS (
                 SELECT LEAST(
                            $3::INT - 1,
                            FLOOR(EXTRACT(EPOCH FROM (r.created_at - $1)) / $4::FLOAT8)::INT
                        ) AS bucket,
                        COUNT(*)::BIGINT AS requests
                 FROM ai_requests r
                 WHERE NOT r.synthetic
                   AND r.created_at >= $1
                   AND r.created_at < $2
                 GROUP BY 1
             )
        SELECT COALESCE(counts.requests, 0)::BIGINT AS "requests!"
        FROM spine LEFT JOIN counts ON counts.bucket = spine.bucket
        ORDER BY spine.bucket
        "#,
        range.from,
        range.to,
        i32::try_from(BUCKETS).unwrap_or(24),
        width,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|row| row.requests).collect())
}
