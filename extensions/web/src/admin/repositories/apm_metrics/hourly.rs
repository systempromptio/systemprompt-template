use std::collections::HashMap;

use sqlx::PgPool;

use super::{DailyApmBucket, HourlyApmBucket};
use crate::admin::numeric;

pub async fn fetch_hourly_breakdown(pool: &PgPool, user_id: &str) -> Vec<HourlyApmBucket> {
    let event_rows = fetch_hourly_events(pool, user_id).await;
    let session_rows = fetch_hourly_sessions(pool, user_id).await;
    build_hourly_buckets(&event_rows, &session_rows)
}

struct EventRow {
    hour: Option<f64>,
    actions: Option<i64>,
    errors: Option<i64>,
    unique_tools: Option<i64>,
}

async fn fetch_hourly_events(pool: &PgPool, user_id: &str) -> Vec<EventRow> {
    sqlx::query_as!(
        EventRow,
        r"SELECT
            EXTRACT(HOUR FROM created_at)::DOUBLE PRECISION AS hour,
            COUNT(*) FILTER (WHERE event_type IN ('PostToolUse', 'UserPromptSubmit')) AS actions,
            COUNT(*) FILTER (WHERE event_type = 'PostToolUseFailure') AS errors,
            COUNT(DISTINCT tool_name) FILTER (WHERE event_type = 'PostToolUse') AS unique_tools
          FROM plugin_usage_events
          WHERE user_id = $1
            AND created_at::date = CURRENT_DATE
          GROUP BY 1",
        user_id,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_else(|_| Vec::new())
}

struct SessionHourRow {
    hour: Option<f64>,
    session_count: Option<i64>,
    subagent_sum: Option<i64>,
    input_bytes: Option<i64>,
    output_bytes: Option<i64>,
}

async fn fetch_hourly_sessions(pool: &PgPool, user_id: &str) -> Vec<SessionHourRow> {
    sqlx::query_as!(
        SessionHourRow,
        r"WITH hours AS (
            SELECT generate_series(0, 23) AS h
          )
          SELECT
            h::DOUBLE PRECISION AS hour,
            COUNT(s.session_id) AS session_count,
            COALESCE(SUM(s.subagent_spawns), 0)::BIGINT AS subagent_sum,
            COALESCE(SUM(s.content_input_bytes), 0)::BIGINT AS input_bytes,
            COALESCE(SUM(s.content_output_bytes), 0)::BIGINT AS output_bytes
          FROM hours
          LEFT JOIN plugin_session_summaries s
            ON s.user_id = $1
            AND s.started_at::date = CURRENT_DATE
            AND COALESCE(s.status, 'active') != 'deleted'
            AND EXTRACT(HOUR FROM s.started_at) <= h
            AND (s.ended_at IS NULL OR EXTRACT(HOUR FROM s.ended_at) >= h)
          GROUP BY h",
        user_id,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_else(|_| Vec::new())
}

fn build_hourly_buckets(
    event_rows: &[EventRow],
    session_rows: &[SessionHourRow],
) -> Vec<HourlyApmBucket> {
    let mut buckets: Vec<HourlyApmBucket> = (0..24)
        .map(|h| HourlyApmBucket {
            hour: h,
            ..Default::default()
        })
        .collect();

    for r in event_rows {
        let h = numeric::to_usize(r.hour.unwrap_or(0.0));
        if h < 24 {
            buckets[h].actions = r.actions.unwrap_or(0);
            buckets[h].errors = r.errors.unwrap_or(0);
            buckets[h].unique_tools = r.unique_tools.unwrap_or(0);
        }
    }

    for r in session_rows {
        let h = numeric::to_usize(r.hour.unwrap_or(0.0));
        if h < 24 {
            buckets[h].sessions = r.session_count.unwrap_or(0);
            buckets[h].subagent_spawns = r.subagent_sum.unwrap_or(0);
            buckets[h].input_bytes = r.input_bytes.unwrap_or(0);
            buckets[h].output_bytes = r.output_bytes.unwrap_or(0);
        }
    }

    buckets
}

pub async fn fetch_daily_breakdown(pool: &PgPool, user_id: &str, days: i32) -> Vec<DailyApmBucket> {
    let (event_rows, session_rows) = tokio::join!(
        fetch_daily_events(pool, user_id, days),
        fetch_daily_sessions(pool, user_id, days),
    );
    build_daily_buckets(days, &event_rows, &session_rows)
}

struct DailyEventRow {
    bucket_date: Option<chrono::NaiveDate>,
    actions: Option<i64>,
    errors: Option<i64>,
    unique_tools: Option<i64>,
}

async fn fetch_daily_events(pool: &PgPool, user_id: &str, days: i32) -> Vec<DailyEventRow> {
    sqlx::query_as!(
        DailyEventRow,
        r"SELECT
            created_at::date AS bucket_date,
            COUNT(*) FILTER (WHERE event_type IN ('PostToolUse', 'UserPromptSubmit')) AS actions,
            COUNT(*) FILTER (WHERE event_type = 'PostToolUseFailure') AS errors,
            COUNT(DISTINCT tool_name) FILTER (WHERE event_type = 'PostToolUse') AS unique_tools
          FROM plugin_usage_events
          WHERE user_id = $1
            AND created_at >= CURRENT_DATE - make_interval(days => $2 - 1)
          GROUP BY 1
          ORDER BY 1",
        user_id,
        days,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_else(|_| Vec::new())
}

struct DailySessionRow {
    bucket_date: Option<chrono::NaiveDate>,
    session_count: Option<i64>,
    subagent_sum: Option<i64>,
    input_bytes: Option<i64>,
    output_bytes: Option<i64>,
}

async fn fetch_daily_sessions(pool: &PgPool, user_id: &str, days: i32) -> Vec<DailySessionRow> {
    sqlx::query_as!(
        DailySessionRow,
        r"SELECT
            d::date AS bucket_date,
            COUNT(s.session_id) AS session_count,
            COALESCE(SUM(s.subagent_spawns), 0)::BIGINT AS subagent_sum,
            COALESCE(SUM(s.content_input_bytes), 0)::BIGINT AS input_bytes,
            COALESCE(SUM(s.content_output_bytes), 0)::BIGINT AS output_bytes
          FROM generate_series(
            CURRENT_DATE - make_interval(days => $2 - 1),
            CURRENT_DATE,
            '1 day'::interval
          ) AS d
          LEFT JOIN plugin_session_summaries s
            ON s.user_id = $1
            AND s.started_at::date = d::date
            AND COALESCE(s.status, 'active') != 'deleted'
          GROUP BY d
          ORDER BY d",
        user_id,
        days,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_else(|_| Vec::new())
}

fn build_daily_buckets(
    days: i32,
    event_rows: &[DailyEventRow],
    session_rows: &[DailySessionRow],
) -> Vec<DailyApmBucket> {
    let today = chrono::Utc::now().date_naive();
    let mut buckets: Vec<DailyApmBucket> = (0..days)
        .map(|i| {
            let d = today - chrono::Duration::days(i64::from(days - 1 - i));
            DailyApmBucket {
                date: d.format("%Y-%m-%d").to_string(),
                ..Default::default()
            }
        })
        .collect();

    let date_index: HashMap<String, usize> = buckets
        .iter()
        .enumerate()
        .map(|(i, b)| (b.date.clone(), i))
        .collect();

    for r in event_rows {
        if let Some(d) = r.bucket_date {
            let key = d.format("%Y-%m-%d").to_string();
            if let Some(&idx) = date_index.get(&key) {
                buckets[idx].actions = r.actions.unwrap_or(0);
                buckets[idx].errors = r.errors.unwrap_or(0);
                buckets[idx].unique_tools = r.unique_tools.unwrap_or(0);
            }
        }
    }

    for r in session_rows {
        if let Some(d) = r.bucket_date {
            let key = d.format("%Y-%m-%d").to_string();
            if let Some(&idx) = date_index.get(&key) {
                buckets[idx].sessions = r.session_count.unwrap_or(0);
                buckets[idx].subagent_spawns = r.subagent_sum.unwrap_or(0);
                buckets[idx].input_bytes = r.input_bytes.unwrap_or(0);
                buckets[idx].output_bytes = r.output_bytes.unwrap_or(0);
            }
        }
    }

    buckets
}
