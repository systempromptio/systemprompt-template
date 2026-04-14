use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Default, sqlx::FromRow)]
pub struct ProfileReportRow {
    pub user_id: String,
    pub archetype: String,
    pub archetype_description: String,
    pub archetype_confidence: i16,
    // JSON: AI-generated profile analysis from jsonb column
    pub strengths: Option<serde_json::Value>,
    // JSON: AI-generated profile analysis from jsonb column
    pub weaknesses: Option<serde_json::Value>,
    pub ai_narrative: Option<String>,
    pub ai_style_analysis: Option<String>,
    pub ai_comparison: Option<String>,
    pub ai_patterns: Option<String>,
    pub ai_improvements: Option<String>,
    pub ai_tips: Option<String>,
    // JSON: AI-generated profile analysis from jsonb column
    pub metrics_snapshot: Option<serde_json::Value>,
    pub period_days: i32,
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug)]
pub struct ProfileReportInput {
    pub archetype: String,
    pub archetype_description: String,
    pub archetype_confidence: i16,
    // JSON: AI-generated profile analysis from jsonb column
    pub strengths: serde_json::Value,
    // JSON: AI-generated profile analysis from jsonb column
    pub weaknesses: serde_json::Value,
    pub ai_narrative: Option<String>,
    pub ai_style_analysis: Option<String>,
    pub ai_comparison: Option<String>,
    pub ai_patterns: Option<String>,
    pub ai_improvements: Option<String>,
    pub ai_tips: Option<String>,
    // JSON: AI-generated profile analysis from jsonb column
    pub metrics_snapshot: serde_json::Value,
    pub period_days: i32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UserAggregateMetrics {
    pub total_days: i64,
    pub total_sessions: i64,
    pub avg_sessions_per_day: f64,
    pub avg_quality: f64,
    pub avg_apm: f64,
    pub avg_peak_apm: f64,
    pub avg_error_rate: f64,
    pub avg_tool_diversity: f64,
    pub avg_multitasking: f64,
    pub avg_goal_rate: f64,
    pub avg_throughput: f64,
    pub total_goals_achieved: i64,
    pub total_goals_partial: i64,
    pub total_goals_failed: i64,
    pub total_prompts: i64,
    pub total_tool_uses: i64,
    pub total_errors: i64,
    pub avg_session_velocity: f64,
    pub avg_concurrency: f64,
    pub category_distribution: std::collections::HashMap<String, i64>,
}

pub async fn fetch_profile_report(pool: &PgPool, user_id: &str) -> Option<ProfileReportRow> {
    sqlx::query_as::<_, ProfileReportRow>(
        r"SELECT
            user_id, archetype, archetype_description, archetype_confidence,
            strengths, weaknesses,
            ai_narrative, ai_style_analysis, ai_comparison,
            ai_patterns, ai_improvements, ai_tips,
            metrics_snapshot, period_days, generated_at
          FROM user_profile_reports
          WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, user_id = %user_id, "Failed to fetch profile report");
    })
    .ok()
    .flatten()
}

pub async fn upsert_profile_report(
    pool: &PgPool,
    user_id: &str,
    input: &ProfileReportInput,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"INSERT INTO user_profile_reports
            (user_id, archetype, archetype_description, archetype_confidence,
             strengths, weaknesses,
             ai_narrative, ai_style_analysis, ai_comparison,
             ai_patterns, ai_improvements, ai_tips,
             metrics_snapshot, period_days, generated_at, updated_at)
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, NOW(), NOW())
          ON CONFLICT (user_id) DO UPDATE SET
            archetype = EXCLUDED.archetype,
            archetype_description = EXCLUDED.archetype_description,
            archetype_confidence = EXCLUDED.archetype_confidence,
            strengths = EXCLUDED.strengths,
            weaknesses = EXCLUDED.weaknesses,
            ai_narrative = EXCLUDED.ai_narrative,
            ai_style_analysis = EXCLUDED.ai_style_analysis,
            ai_comparison = EXCLUDED.ai_comparison,
            ai_patterns = EXCLUDED.ai_patterns,
            ai_improvements = EXCLUDED.ai_improvements,
            ai_tips = EXCLUDED.ai_tips,
            metrics_snapshot = EXCLUDED.metrics_snapshot,
            period_days = EXCLUDED.period_days,
            generated_at = NOW(),
            updated_at = NOW()",
        user_id,
        input.archetype,
        input.archetype_description,
        input.archetype_confidence,
        input.strengths,
        input.weaknesses,
        input.ai_narrative,
        input.ai_style_analysis,
        input.ai_comparison,
        input.ai_patterns,
        input.ai_improvements,
        input.ai_tips,
        input.metrics_snapshot,
        input.period_days,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn fetch_user_aggregate_metrics(
    pool: &PgPool,
    user_id: &str,
    days: i32,
) -> UserAggregateMetrics {
    let row = fetch_aggregate_row(pool, user_id, days).await;
    let Some(row) = row else {
        return UserAggregateMetrics::default();
    };

    let total_days = row.total_days.unwrap_or(0);
    let total_sessions = row.total_sessions.unwrap_or(0);
    let category_distribution = fetch_category_distribution(pool, user_id, days).await;

    let avg_sessions_per_day = if total_days > 0 {
        let sessions_f = f64::from(u32::try_from(total_sessions).unwrap_or(0));
        let days_f = f64::from(u32::try_from(total_days).unwrap_or(1));
        sessions_f / days_f
    } else {
        0.0
    };

    UserAggregateMetrics {
        total_days,
        total_sessions,
        avg_sessions_per_day,
        avg_quality: row.avg_quality.unwrap_or(0.0),
        avg_apm: row.avg_apm.unwrap_or(0.0),
        avg_peak_apm: row.avg_peak_apm.unwrap_or(0.0),
        avg_error_rate: row.avg_error_rate.unwrap_or(0.0),
        avg_tool_diversity: row.avg_tool_diversity.unwrap_or(0.0),
        avg_multitasking: row.avg_multitasking.unwrap_or(0.0),
        avg_goal_rate: row.avg_goal_rate.unwrap_or(0.0),
        avg_throughput: row.avg_throughput.unwrap_or(0.0),
        total_goals_achieved: row.total_goals_achieved.unwrap_or(0),
        total_goals_partial: row.total_goals_partial.unwrap_or(0),
        total_goals_failed: row.total_goals_failed.unwrap_or(0),
        total_prompts: row.total_prompts.unwrap_or(0),
        total_tool_uses: row.total_tool_uses.unwrap_or(0),
        total_errors: row.total_errors.unwrap_or(0),
        avg_session_velocity: row.avg_session_velocity.unwrap_or(0.0),
        avg_concurrency: row.avg_concurrency.unwrap_or(0.0),
        category_distribution,
    }
}

#[derive(sqlx::FromRow)]
struct AggRow {
    total_days: Option<i64>,
    total_sessions: Option<i64>,
    avg_quality: Option<f64>,
    avg_apm: Option<f64>,
    avg_peak_apm: Option<f64>,
    avg_error_rate: Option<f64>,
    avg_tool_diversity: Option<f64>,
    avg_multitasking: Option<f64>,
    avg_goal_rate: Option<f64>,
    avg_throughput: Option<f64>,
    total_goals_achieved: Option<i64>,
    total_goals_partial: Option<i64>,
    total_goals_failed: Option<i64>,
    total_prompts: Option<i64>,
    total_tool_uses: Option<i64>,
    total_errors: Option<i64>,
    avg_session_velocity: Option<f64>,
    avg_concurrency: Option<f64>,
}

async fn fetch_aggregate_row(pool: &PgPool, user_id: &str, days: i32) -> Option<AggRow> {
    sqlx::query_as::<_, AggRow>(
        r"SELECT
            COUNT(*)::BIGINT AS total_days,
            SUM(session_count)::BIGINT AS total_sessions,
            AVG(avg_quality_score)::FLOAT8 AS avg_quality,
            AVG(avg_apm)::FLOAT8 AS avg_apm,
            AVG(peak_apm)::FLOAT8 AS avg_peak_apm,
            AVG(total_errors::FLOAT8 / NULLIF(total_prompts + total_tool_uses, 0) * 100) AS avg_error_rate,
            AVG(tool_diversity)::FLOAT8 AS avg_tool_diversity,
            AVG(multitasking_score)::FLOAT8 AS avg_multitasking,
            AVG(goals_achieved::FLOAT8 / NULLIF(goals_achieved + goals_failed, 0) * 100) AS avg_goal_rate,
            AVG((total_input_bytes + total_output_bytes)::FLOAT8) AS avg_throughput,
            SUM(goals_achieved)::BIGINT AS total_goals_achieved,
            SUM(goals_partial)::BIGINT AS total_goals_partial,
            SUM(goals_failed)::BIGINT AS total_goals_failed,
            SUM(total_prompts)::BIGINT AS total_prompts,
            SUM(total_tool_uses)::BIGINT AS total_tool_uses,
            SUM(total_errors)::BIGINT AS total_errors,
            AVG(session_velocity)::FLOAT8 AS avg_session_velocity,
            AVG(avg_concurrency)::FLOAT8 AS avg_concurrency
          FROM daily_summaries
          WHERE user_id = $1
            AND summary_date >= CURRENT_DATE - $2",
    )
    .bind(user_id)
    .bind(days)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, user_id = %user_id, "Failed to fetch aggregate metrics for profile report");
    })
    .ok()
    .flatten()
}

async fn fetch_category_distribution(
    pool: &PgPool,
    user_id: &str,
    days: i32,
) -> std::collections::HashMap<String, i64> {
    #[derive(sqlx::FromRow)]
    struct CatRow {
        category: Option<String>,
        total: Option<i64>,
    }

    let rows = sqlx::query_as::<_, CatRow>(
        r"SELECT category, SUM(count)::BIGINT AS total
          FROM (
            SELECT jsonb_object_keys(category_distribution) AS category,
                   (category_distribution ->> jsonb_object_keys(category_distribution))::BIGINT AS count
            FROM daily_summaries
            WHERE user_id = $1
              AND summary_date >= CURRENT_DATE - $2
              AND category_distribution IS NOT NULL
          ) sub
          GROUP BY category
          ORDER BY total DESC",
    )
    .bind(user_id)
    .bind(days)
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch category distribution for profile report");
        Vec::new()
    });

    rows.into_iter()
        .filter_map(|r| Some((r.category?, r.total.unwrap_or(0))))
        .collect()
}
