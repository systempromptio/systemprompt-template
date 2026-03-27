use sqlx::PgPool;

use super::calculations::{
    calculate_daily_apm_stats, calculate_daily_concurrency, calculate_multitasking_score,
    calculate_tool_diversity,
};
use super::TodayPerformanceSummary;
use crate::admin::numeric;

pub async fn fetch_today_performance_summary(
    pool: &PgPool,
    user_id: &str,
) -> TodayPerformanceSummary {
    let today = chrono::Utc::now().date_naive();

    let agg = fetch_today_aggregates(pool, user_id, today).await;

    let total_sessions = agg.sessions;
    let total_prompts = agg.prompts;
    let total_tool_uses = agg.tool_uses;
    let total_errors = agg.errors;
    let total_actions = total_prompts + total_tool_uses;
    let active_minutes = numeric::to_f32(agg.duration_secs / 60.0);

    let error_rate_pct = if total_actions > 0 {
        (numeric::to_f32_from_i64(total_errors) / numeric::to_f32_from_i64(total_actions)) * 100.0
    } else {
        0.0
    };

    let (avg_apm_opt, peak_apm_opt, _) = calculate_daily_apm_stats(pool, user_id, today).await;
    let (peak_concurrency, _) = calculate_daily_concurrency(pool, user_id, today).await;
    let tool_diversity = calculate_tool_diversity(pool, user_id, today).await;
    let multitasking_score = calculate_multitasking_score(
        pool,
        user_id,
        today,
        peak_concurrency,
        numeric::saturating_i32(total_sessions),
    )
    .await;

    TodayPerformanceSummary {
        total_sessions,
        total_actions,
        total_prompts,
        total_tool_uses,
        total_errors,
        error_rate_pct,
        total_input_bytes: agg.input,
        total_output_bytes: agg.output,
        avg_apm: avg_apm_opt.unwrap_or(0.0),
        peak_apm: peak_apm_opt.unwrap_or(0.0),
        peak_concurrency,
        tool_diversity,
        multitasking_score,
        active_minutes,
    }
}

struct TodayAggregates {
    sessions: i64,
    prompts: i64,
    tool_uses: i64,
    errors: i64,
    input: i64,
    output: i64,
    duration_secs: f64,
}

async fn fetch_today_aggregates(
    pool: &PgPool,
    user_id: &str,
    today: chrono::NaiveDate,
) -> TodayAggregates {
    struct AggRow {
        sessions: Option<i64>,
        prompts: Option<i64>,
        tool_uses: Option<i64>,
        errors: Option<i64>,
        input: Option<i64>,
        output: Option<i64>,
        duration_secs: Option<f64>,
    }

    let agg = sqlx::query_as!(
        AggRow,
        r"SELECT
            COUNT(*)::BIGINT AS sessions,
            COALESCE(SUM(prompts), 0)::BIGINT AS prompts,
            COALESCE(SUM(tool_uses), 0)::BIGINT AS tool_uses,
            COALESCE(SUM(errors), 0)::BIGINT AS errors,
            COALESCE(SUM(content_input_bytes), 0)::BIGINT AS input,
            COALESCE(SUM(content_output_bytes), 0)::BIGINT AS output,
            COALESCE(SUM(EXTRACT(EPOCH FROM (COALESCE(ended_at, NOW()) - started_at))), 0)::DOUBLE PRECISION AS duration_secs
          FROM plugin_session_summaries
          WHERE user_id = $1
            AND started_at::date = $2
            AND COALESCE(status, 'active') != 'deleted'",
        user_id,
        today,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, user_id = %user_id, "Failed to fetch today's performance aggregates");
    })
    .ok()
    .flatten();

    TodayAggregates {
        sessions: agg.as_ref().and_then(|r| r.sessions).unwrap_or(0),
        prompts: agg.as_ref().and_then(|r| r.prompts).unwrap_or(0),
        tool_uses: agg.as_ref().and_then(|r| r.tool_uses).unwrap_or(0),
        errors: agg.as_ref().and_then(|r| r.errors).unwrap_or(0),
        input: agg.as_ref().and_then(|r| r.input).unwrap_or(0),
        output: agg.as_ref().and_then(|r| r.output).unwrap_or(0),
        duration_secs: agg.as_ref().and_then(|r| r.duration_secs).unwrap_or(0.0),
    }
}
