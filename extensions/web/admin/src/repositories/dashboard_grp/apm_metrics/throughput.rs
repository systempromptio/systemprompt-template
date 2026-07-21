use chrono::NaiveDate;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::numeric;

pub async fn calculate_daily_throughput(
    pool: &PgPool,
    user_id: &UserId,
    date: NaiveDate,
) -> (i64, i64, i64) {
    struct Row {
        total_input: Option<i64>,
        total_output: Option<i64>,
    }

    let totals = sqlx::query_as!(
        Row,
        r"SELECT
            SUM(content_input_bytes)::BIGINT AS total_input,
            SUM(content_output_bytes)::BIGINT AS total_output
          FROM plugin_session_summaries
          WHERE user_id = $1
            AND started_at::date = $2
            AND COALESCE(status, 'active') != 'deleted'",
        user_id.as_str(),
        date,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, user_id = %user_id, "Failed to fetch throughput totals");
    })
    .ok()
    .flatten();

    let total_input = totals.as_ref().and_then(|r| r.total_input).unwrap_or(0);
    let total_output = totals.as_ref().and_then(|r| r.total_output).unwrap_or(0);

    let peak_bps = calculate_peak_throughput(pool, user_id, date).await;

    (total_input, total_output, peak_bps)
}

async fn calculate_peak_throughput(pool: &PgPool, user_id: &UserId, date: NaiveDate) -> i64 {
    struct SessionBytesRow {
        total_bytes: Option<i64>,
        duration_secs: Option<f64>,
    }

    let session_rows = sqlx::query_as!(
        SessionBytesRow,
        r"SELECT
            COALESCE(content_input_bytes, 0) + COALESCE(content_output_bytes, 0) AS total_bytes,
            EXTRACT(EPOCH FROM (ended_at - started_at))::DOUBLE PRECISION AS duration_secs
          FROM plugin_session_summaries
          WHERE user_id = $1
            AND started_at::date = $2
            AND ended_at IS NOT NULL
            AND COALESCE(status, 'active') != 'deleted'",
        user_id.as_str(),
        date,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch session throughput rows");
        Vec::new()
    });

    session_rows
        .iter()
        .filter_map(|r| {
            let bytes = r.total_bytes.unwrap_or(0);
            let secs = r.duration_secs.unwrap_or(0.0);
            if secs > 0.0 {
                Some(numeric::to_i64(numeric::to_f64(bytes) / secs))
            } else {
                None
            }
        })
        .max()
        .unwrap_or(0)
}
