use chrono::NaiveDate;
use sqlx::PgPool;

use crate::admin::numeric;

pub async fn calculate_session_apm(pool: &PgPool, session_id: &str) -> (f32, f32) {
    struct Row {
        tool_uses: Option<i64>,
        prompts: Option<i64>,
        errors: Option<i64>,
        started_at: Option<chrono::DateTime<chrono::Utc>>,
        ended_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    let row = sqlx::query_as!(
        Row,
        r"SELECT tool_uses, prompts, errors, started_at, ended_at
          FROM plugin_session_summaries
          WHERE session_id = $1",
        session_id,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, session_id = %session_id, "Failed to fetch session APM data");
    })
    .ok()
    .flatten();

    let Some(r) = row else {
        return (0.0, 0.0);
    };

    let tool_uses = r.tool_uses.unwrap_or(0);
    let prompts = r.prompts.unwrap_or(0);
    let errors = r.errors.unwrap_or(0);

    let duration_minutes = match (r.started_at, r.ended_at) {
        (Some(s), Some(e)) => {
            let mins = numeric::seconds_to_f64((e - s).num_seconds()) / 60.0;
            mins.max(1.0)
        }
        _ => 1.0,
    };

    let apm = numeric::to_f32_from_i64(tool_uses + prompts) / numeric::to_f32(duration_minutes);
    let eapm = numeric::to_f32_from_i64((tool_uses + prompts - errors).max(0))
        / numeric::to_f32(duration_minutes);

    (apm, eapm)
}

pub async fn calculate_daily_concurrency(
    pool: &PgPool,
    user_id: &str,
    date: NaiveDate,
) -> (i32, f32) {
    struct Row {
        started_at: Option<chrono::DateTime<chrono::Utc>>,
        ended_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    let rows = sqlx::query_as!(
        Row,
        r"SELECT started_at, ended_at
          FROM plugin_session_summaries
          WHERE user_id = $1
            AND started_at::date = $2
            AND COALESCE(status, 'active') != 'deleted'",
        user_id,
        date,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_else(|_| Vec::new());

    if rows.is_empty() {
        return (0, 0.0);
    }

    let now = chrono::Utc::now();
    let intervals: Vec<_> = rows
        .iter()
        .filter_map(|r| r.started_at.map(|s| (s, r.ended_at.unwrap_or(now))))
        .collect();

    sweep_line_concurrency(&intervals)
}

fn sweep_line_concurrency(
    intervals: &[(chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)],
) -> (i32, f32) {
    let mut events: Vec<(chrono::DateTime<chrono::Utc>, i32)> = Vec::new();
    for (start, end) in intervals {
        events.push((*start, 1));
        events.push((*end, -1));
    }

    events.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    let mut current = 0i32;
    let mut peak = 0i32;
    let mut weighted_sum = 0.0f64;
    let mut prev_time: Option<chrono::DateTime<chrono::Utc>> = None;

    for (time, delta) in &events {
        if let Some(prev) = prev_time {
            let dt = numeric::seconds_to_f64((*time - prev).num_seconds());
            weighted_sum += f64::from(current) * dt;
        }
        current += delta;
        peak = peak.max(current);
        prev_time = Some(*time);
    }

    let total_seconds = if let (Some(first), Some(last)) = (events.first(), events.last()) {
        numeric::seconds_to_f64((last.0 - first.0).num_seconds())
    } else {
        1.0
    };

    let avg = if total_seconds > 0.0 {
        weighted_sum / total_seconds
    } else {
        0.0
    };

    (peak, numeric::to_f32(avg))
}

pub async fn calculate_daily_apm_stats(
    pool: &PgPool,
    user_id: &str,
    date: NaiveDate,
) -> (Option<f32>, Option<f32>, Option<f32>) {
    struct Row {
        avg_apm: Option<f64>,
        peak_apm: Option<f32>,
        avg_eapm: Option<f64>,
    }

    let row = sqlx::query_as!(
        Row,
        r"SELECT
            AVG(apm)::DOUBLE PRECISION AS avg_apm,
            MAX(apm) AS peak_apm,
            AVG(eapm)::DOUBLE PRECISION AS avg_eapm
          FROM plugin_session_summaries
          WHERE user_id = $1
            AND started_at::date = $2
            AND ended_at IS NOT NULL
            AND apm IS NOT NULL",
        user_id,
        date,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, user_id = %user_id, "Failed to fetch daily APM stats");
    })
    .ok()
    .flatten();

    match row {
        Some(r) => (
            r.avg_apm.map(numeric::to_f32),
            r.peak_apm,
            r.avg_eapm.map(numeric::to_f32),
        ),
        None => (None, None, None),
    }
}

pub async fn calculate_tool_diversity(pool: &PgPool, user_id: &str, date: NaiveDate) -> i32 {
    let count = sqlx::query_scalar!(
        r"SELECT COUNT(DISTINCT tool_name)
          FROM plugin_usage_events
          WHERE user_id = $1
            AND event_type = 'PostToolUse'
            AND created_at::date = $2",
        user_id,
        date,
    )
    .fetch_one(pool)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0);

    numeric::saturating_i32(count)
}

pub async fn calculate_multitasking_score(
    pool: &PgPool,
    user_id: &str,
    date: NaiveDate,
    peak_concurrency: i32,
    session_count: i32,
) -> f32 {
    let subagent_spawns: i64 = sqlx::query_scalar::<_, i64>(
        r"SELECT COALESCE(SUM(subagent_spawns), 0)::bigint
          FROM plugin_session_summaries
          WHERE user_id = $1
            AND started_at::date = $2
            AND COALESCE(status, 'active') != 'deleted'",
    )
    .bind(user_id)
    .bind(date)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let denom = numeric::to_f32_from_i64(i64::from(session_count.max(1)));
    let score = (numeric::to_f32_from_i64(subagent_spawns) * 2.0
        + numeric::to_f32(f64::from(peak_concurrency)) * 3.0)
        / denom
        * 10.0;
    score.min(100.0)
}
