use sqlx::PgPool;

use super::calculations::{
    calculate_daily_apm_stats, calculate_daily_concurrency, calculate_multitasking_score,
    calculate_tool_diversity,
};
use super::throughput::calculate_daily_throughput;
use super::{format_bytes_rate, format_total_bytes, ApmCorrelation, TodayApmLive};
use crate::admin::numeric;

pub async fn fetch_today_apm_live(pool: &PgPool, user_id: &str) -> TodayApmLive {
    let today = chrono::Utc::now().date_naive();

    let (current_apm, current_concurrency, total_active_seconds) =
        compute_active_session_apm(pool, user_id).await;

    let (avg_apm, peak_apm, _avg_eapm) = calculate_daily_apm_stats(pool, user_id, today).await;
    let (peak_concurrency, avg_concurrency) =
        calculate_daily_concurrency(pool, user_id, today).await;

    let (total_input, total_output, _peak_bps) =
        calculate_daily_throughput(pool, user_id, today).await;
    let total_bytes = total_input + total_output;
    let total_throughput_display = format_total_bytes(total_bytes);
    let throughput_rate_display = format_bytes_rate(total_bytes, total_active_seconds);

    let tool_diversity = calculate_tool_diversity(pool, user_id, today).await;

    let session_count = fetch_today_session_count(pool, user_id).await;

    let multitasking_score = calculate_multitasking_score(
        pool,
        user_id,
        today,
        peak_concurrency,
        numeric::saturating_i32(session_count),
    )
    .await;

    TodayApmLive {
        current_apm,
        peak_apm: peak_apm.unwrap_or(current_apm),
        avg_apm: avg_apm.unwrap_or(current_apm),
        current_concurrency,
        peak_concurrency,
        avg_concurrency,
        total_throughput_display,
        throughput_rate_display,
        tool_diversity,
        multitasking_score,
    }
}

async fn compute_active_session_apm(pool: &PgPool, user_id: &str) -> (f32, i32, f64) {
    struct ActiveRow {
        tool_uses: Option<i64>,
        prompts: Option<i64>,
        started_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    let active_sessions = sqlx::query_as!(
        ActiveRow,
        r"SELECT tool_uses, prompts, started_at
          FROM plugin_session_summaries
          WHERE user_id = $1
            AND started_at::date = CURRENT_DATE
            AND ended_at IS NULL
            AND COALESCE(status, 'active') != 'deleted'",
        user_id,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_else(|_| Vec::new());

    let now = chrono::Utc::now();
    let mut current_apm = 0.0f32;
    for s in &active_sessions {
        let actions = s.tool_uses.unwrap_or(0) + s.prompts.unwrap_or(0);
        let mins = s
            .started_at
            .map_or(1.0, |st| {
                numeric::seconds_to_f64((now - st).num_seconds()) / 60.0
            })
            .max(1.0);
        current_apm += numeric::to_f32_from_i64(actions) / numeric::to_f32(mins);
    }

    #[allow(clippy::cast_possible_wrap)]
    let current_concurrency = numeric::saturating_i32(active_sessions.len() as i64);

    let total_active_seconds: f64 = active_sessions
        .iter()
        .filter_map(|s| {
            s.started_at
                .map(|st| numeric::seconds_to_f64((now - st).num_seconds()))
        })
        .sum::<f64>()
        .max(1.0);

    (current_apm, current_concurrency, total_active_seconds)
}

async fn fetch_today_session_count(pool: &PgPool, user_id: &str) -> i64 {
    sqlx::query_scalar!(
        r"SELECT COUNT(*)
          FROM plugin_session_summaries
          WHERE user_id = $1
            AND started_at::date = CURRENT_DATE
            AND COALESCE(status, 'active') != 'deleted'",
        user_id,
    )
    .fetch_one(pool)
    .await
    .unwrap_or(Some(0))
    .unwrap_or(0)
}

pub async fn fetch_apm_success_correlation(pool: &PgPool, user_id: &str) -> ApmCorrelation {
    struct Row {
        apm_tier: Option<String>,
        success_rate: Option<f64>,
        avg_quality: Option<f64>,
    }

    let rows = sqlx::query_as!(
        Row,
        r"SELECT
            CASE
                WHEN s.apm >= 30 THEN 'high'
                WHEN s.apm >= 10 THEN 'medium'
                ELSE 'low'
            END AS apm_tier,
            AVG(CASE WHEN a.goal_achieved = 'yes' THEN 1.0 ELSE 0.0 END)::DOUBLE PRECISION AS success_rate,
            AVG(a.quality_score)::DOUBLE PRECISION AS avg_quality
          FROM plugin_session_summaries s
          JOIN session_analyses a ON a.session_id = s.session_id
          WHERE s.user_id = $1 AND s.apm IS NOT NULL
          GROUP BY 1",
        user_id,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_else(|_| Vec::new());

    let mut result = ApmCorrelation::default();
    for r in &rows {
        let tier = r.apm_tier.as_deref().unwrap_or("");
        let sr = numeric::to_f32(r.success_rate.unwrap_or(0.0));
        let aq = numeric::to_f32(r.avg_quality.unwrap_or(0.0));
        match tier {
            "high" => {
                result.high_apm_success_rate = sr;
                result.high_apm_avg_quality = aq;
            }
            "medium" => {
                result.medium_apm_success_rate = sr;
            }
            "low" => {
                result.low_apm_success_rate = sr;
                result.low_apm_avg_quality = aq;
            }
            _ => {}
        }
    }

    result
}

pub async fn update_session_apm(
    pool: &PgPool,
    session_id: &str,
    apm: f32,
    eapm: f32,
    peak_concurrent: i32,
) {
    let result = sqlx::query!(
        r"UPDATE plugin_session_summaries
          SET apm = $1, eapm = $2, peak_concurrent = $3, updated_at = NOW()
          WHERE session_id = $4",
        apm,
        eapm,
        peak_concurrent,
        session_id,
    )
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, session_id, "Failed to update session APM");
    }
}
