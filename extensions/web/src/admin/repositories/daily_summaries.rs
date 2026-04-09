use sqlx::PgPool;

#[derive(Debug, Default, sqlx::FromRow)]
pub struct DailySummaryRow {
    pub summary_date: chrono::NaiveDate,
    pub session_count: i32,
    pub avg_quality_score: Option<f32>,
    pub goals_achieved: i32,
    pub goals_partial: i32,
    pub goals_failed: i32,
    pub total_prompts: i64,
    pub total_tool_uses: i64,
    pub total_errors: i64,
    pub summary: String,
    pub patterns: Option<String>,
    pub skill_gaps: Option<String>,
    pub top_recommendation: Option<String>,
    pub daily_xp: i32,
    pub tags: String,
    pub avg_apm: Option<f32>,
    pub peak_apm: Option<f32>,
    pub avg_eapm: Option<f32>,
    pub peak_concurrency: i32,
    pub avg_concurrency: Option<f32>,
    pub total_input_bytes: i64,
    pub total_output_bytes: i64,
    pub peak_throughput_bps: i64,
    pub tool_diversity: i32,
    pub multitasking_score: Option<f32>,
    pub session_velocity: Option<f32>,
    pub achievements_unlocked: String,
    pub highlights: Option<String>,
    pub trends: Option<String>,
    pub category_distribution: Option<serde_json::Value>,
    pub plugins_count: i32,
    pub skills_count: i32,
    pub agents_count: i32,
    pub mcp_servers_count: i32,
    pub hooks_count: i32,
    pub health_score: Option<f32>,
    pub skill_effectiveness: Option<serde_json::Value>,
    pub avg_session_duration_minutes: Option<f32>,
    pub avg_turns_per_session: Option<f32>,
    pub total_corrections: i32,
    pub avg_automation_ratio: Option<f32>,
    pub plan_mode_sessions: i32,
}

#[derive(Debug)]
pub struct DailySummaryInput {
    pub session_count: i32,
    pub avg_quality_score: Option<f32>,
    pub goals_achieved: i32,
    pub goals_partial: i32,
    pub goals_failed: i32,
    pub total_prompts: i64,
    pub total_tool_uses: i64,
    pub total_errors: i64,
    pub summary: String,
    pub patterns: Option<String>,
    pub skill_gaps: Option<String>,
    pub top_recommendation: Option<String>,
    pub daily_xp: i32,
    pub tags: String,
    pub avg_apm: Option<f32>,
    pub peak_apm: Option<f32>,
    pub avg_eapm: Option<f32>,
    pub peak_concurrency: i32,
    pub avg_concurrency: Option<f32>,
    pub total_input_bytes: i64,
    pub total_output_bytes: i64,
    pub peak_throughput_bps: i64,
    pub tool_diversity: i32,
    pub multitasking_score: Option<f32>,
    pub session_velocity: Option<f32>,
    pub achievements_unlocked: String,
    pub highlights: Option<String>,
    pub trends: Option<String>,
    pub category_distribution: Option<serde_json::Value>,
    pub plugins_count: i32,
    pub skills_count: i32,
    pub agents_count: i32,
    pub mcp_servers_count: i32,
    pub hooks_count: i32,
    pub health_score: Option<f32>,
    pub skill_effectiveness: Option<serde_json::Value>,
    pub avg_session_duration_minutes: Option<f32>,
    pub avg_turns_per_session: Option<f32>,
    pub total_corrections: i32,
    pub avg_automation_ratio: Option<f32>,
    pub plan_mode_sessions: i32,
}

#[derive(Debug, Default, sqlx::FromRow, Clone, Copy)]
pub struct GlobalAverages {
    pub avg_sessions: Option<f32>,
    pub avg_quality: Option<f32>,
    pub avg_apm: Option<f32>,
    pub avg_peak_apm: Option<f32>,
    pub avg_error_rate: Option<f64>,
    pub avg_tool_diversity: Option<f32>,
    pub avg_multitasking: Option<f32>,
    pub avg_goal_rate: Option<f64>,
    pub avg_throughput: Option<i64>,
    pub total_users: Option<i64>,
}

pub async fn upsert_daily_summary(
    pool: &PgPool,
    user_id: &str,
    date: chrono::NaiveDate,
    input: &DailySummaryInput,
) -> Result<(), sqlx::Error> {
    execute_upsert(pool, user_id, date, input).await
}

async fn execute_upsert(
    pool: &PgPool,
    user_id: &str,
    date: chrono::NaiveDate,
    input: &DailySummaryInput,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"INSERT INTO daily_summaries
            (user_id, summary_date, session_count, avg_quality_score,
             goals_achieved, goals_partial, goals_failed,
             total_prompts, total_tool_uses, total_errors,
             summary, patterns, skill_gaps, top_recommendation,
             daily_xp, tags,
             avg_apm, peak_apm, avg_eapm, peak_concurrency, avg_concurrency,
             total_input_bytes, total_output_bytes, peak_throughput_bps,
             tool_diversity, multitasking_score, session_velocity, achievements_unlocked,
             highlights, trends, category_distribution, plugins_count, skills_count, agents_count, mcp_servers_count, hooks_count,
             health_score, skill_effectiveness,
             avg_session_duration_minutes, avg_turns_per_session, total_corrections, avg_automation_ratio, plan_mode_sessions)
          VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
                  $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28,
                  $29, $30, $31, $32, $33, $34, $35, $36, $37, $38,
                  $39, $40, $41, $42, $43)
          ON CONFLICT (user_id, summary_date) DO UPDATE SET
            session_count = EXCLUDED.session_count, avg_quality_score = EXCLUDED.avg_quality_score,
            goals_achieved = EXCLUDED.goals_achieved, goals_partial = EXCLUDED.goals_partial,
            goals_failed = EXCLUDED.goals_failed, total_prompts = EXCLUDED.total_prompts,
            total_tool_uses = EXCLUDED.total_tool_uses, total_errors = EXCLUDED.total_errors,
            summary = EXCLUDED.summary, patterns = EXCLUDED.patterns,
            skill_gaps = EXCLUDED.skill_gaps, top_recommendation = EXCLUDED.top_recommendation,
            daily_xp = EXCLUDED.daily_xp, tags = EXCLUDED.tags,
            avg_apm = EXCLUDED.avg_apm, peak_apm = EXCLUDED.peak_apm,
            avg_eapm = EXCLUDED.avg_eapm, peak_concurrency = EXCLUDED.peak_concurrency,
            avg_concurrency = EXCLUDED.avg_concurrency, total_input_bytes = EXCLUDED.total_input_bytes,
            total_output_bytes = EXCLUDED.total_output_bytes, peak_throughput_bps = EXCLUDED.peak_throughput_bps,
            tool_diversity = EXCLUDED.tool_diversity, multitasking_score = EXCLUDED.multitasking_score,
            session_velocity = EXCLUDED.session_velocity, achievements_unlocked = EXCLUDED.achievements_unlocked,
            highlights = EXCLUDED.highlights, trends = EXCLUDED.trends,
            category_distribution = EXCLUDED.category_distribution,
            plugins_count = EXCLUDED.plugins_count, skills_count = EXCLUDED.skills_count,
            agents_count = EXCLUDED.agents_count, mcp_servers_count = EXCLUDED.mcp_servers_count,
            hooks_count = EXCLUDED.hooks_count, health_score = EXCLUDED.health_score,
            skill_effectiveness = EXCLUDED.skill_effectiveness,
            avg_session_duration_minutes = EXCLUDED.avg_session_duration_minutes,
            avg_turns_per_session = EXCLUDED.avg_turns_per_session,
            total_corrections = EXCLUDED.total_corrections,
            avg_automation_ratio = EXCLUDED.avg_automation_ratio,
            plan_mode_sessions = EXCLUDED.plan_mode_sessions, updated_at = NOW()",
        user_id, date, input.session_count, input.avg_quality_score,
        input.goals_achieved, input.goals_partial, input.goals_failed,
        input.total_prompts, input.total_tool_uses, input.total_errors,
        input.summary, input.patterns.clone(),
        input.skill_gaps.clone(),
        input.top_recommendation.clone(),
        input.daily_xp, input.tags, input.avg_apm, input.peak_apm,
        input.avg_eapm, input.peak_concurrency, input.avg_concurrency,
        input.total_input_bytes, input.total_output_bytes, input.peak_throughput_bps,
        input.tool_diversity, input.multitasking_score, input.session_velocity,
        input.achievements_unlocked, input.highlights.clone(),
        input.trends.clone(),
        input.category_distribution.clone(),
        input.plugins_count, input.skills_count,
        input.agents_count, input.mcp_servers_count, input.hooks_count,
        input.health_score, input.skill_effectiveness.clone(),
        input.avg_session_duration_minutes, input.avg_turns_per_session,
        input.total_corrections, input.avg_automation_ratio, input.plan_mode_sessions,
    )
    .execute(pool)
    .await?;
    Ok(())
}

const SELECT_COLUMNS: &str = r"summary_date, session_count, avg_quality_score,
    goals_achieved, goals_partial, goals_failed,
    total_prompts, total_tool_uses, total_errors,
    summary, patterns, skill_gaps, top_recommendation,
    daily_xp, tags,
    avg_apm, peak_apm, avg_eapm, peak_concurrency, avg_concurrency,
    total_input_bytes, total_output_bytes, peak_throughput_bps,
    tool_diversity, multitasking_score, session_velocity, achievements_unlocked,
    highlights, trends, category_distribution, plugins_count, skills_count, agents_count, mcp_servers_count, hooks_count,
    health_score, skill_effectiveness,
    avg_session_duration_minutes, avg_turns_per_session, total_corrections, avg_automation_ratio, plan_mode_sessions";

pub async fn fetch_daily_summary(
    pool: &PgPool,
    user_id: &str,
    date: chrono::NaiveDate,
) -> Option<DailySummaryRow> {
    sqlx::query_as::<_, DailySummaryRow>(&format!(
        "SELECT {SELECT_COLUMNS} FROM daily_summaries WHERE user_id = $1 AND summary_date = $2"
    ))
    .bind(user_id)
    .bind(date)
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, user_id = %user_id, "Failed to fetch daily summary");
    })
    .ok()
    .flatten()
}

pub async fn fetch_recent_daily_summaries(
    pool: &PgPool,
    user_id: &str,
    limit: i64,
) -> Vec<DailySummaryRow> {
    sqlx::query_as::<_, DailySummaryRow>(&format!(
        "SELECT {SELECT_COLUMNS} FROM daily_summaries WHERE user_id = $1 ORDER BY summary_date DESC LIMIT $2"
    ))
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .unwrap_or_else(|_| Vec::new())
}

pub async fn fetch_global_averages(pool: &PgPool) -> GlobalAverages {
    sqlx::query_as!(
        GlobalAverages,
        r#"SELECT
            AVG(session_count)::REAL AS avg_sessions,
            AVG(avg_quality_score)::REAL AS avg_quality,
            AVG(avg_apm)::REAL AS avg_apm,
            AVG(peak_apm)::REAL AS avg_peak_apm,
            AVG(total_errors::REAL / NULLIF(total_prompts + total_tool_uses, 0) * 100) AS avg_error_rate,
            AVG(tool_diversity)::REAL AS avg_tool_diversity,
            AVG(multitasking_score)::REAL AS avg_multitasking,
            AVG(goals_achieved::REAL / NULLIF(goals_achieved + goals_failed, 0) * 100) AS avg_goal_rate,
            AVG(total_input_bytes + total_output_bytes)::BIGINT AS avg_throughput,
            COUNT(DISTINCT user_id)::BIGINT AS total_users
          FROM daily_summaries
          WHERE summary_date >= CURRENT_DATE - INTERVAL '30 days'"#,
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, "Failed to fetch global averages");
    })
    .ok()
    .flatten()
    .unwrap_or_else(GlobalAverages::default)
}
