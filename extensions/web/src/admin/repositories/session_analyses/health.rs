use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug, Default)]
pub struct HealthMetrics {
    pub total_sessions_30d: i64,
    pub avg_quality: f64,
    pub goals_achieved: i64,
    pub goals_failed: i64,
    pub sessions_with_errors: i64,
    pub total_prompts: i64,
    pub total_tool_uses: i64,
    pub total_errors: i64,
    pub health_score: i64,
    pub top_recommendation: String,
}

pub async fn fetch_health_metrics(pool: &PgPool, user_id: &UserId) -> HealthMetrics {
    let (analysis_row, summary_row, top_rec) = tokio::join!(
        fetch_health_analysis_metrics(pool, user_id),
        fetch_health_summary_metrics(pool, user_id),
        fetch_latest_recommendation(pool, user_id),
    );

    let avg_quality = analysis_row.avg_quality.unwrap_or(0.0);
    let total_sessions = analysis_row.total.max(summary_row.sessions);

    let health_score = compute_health_score(avg_quality, analysis_row.achieved, total_sessions);

    HealthMetrics {
        total_sessions_30d: total_sessions,
        avg_quality,
        goals_achieved: analysis_row.achieved,
        goals_failed: analysis_row.failed,
        sessions_with_errors: analysis_row.sessions_with_errors,
        total_prompts: summary_row.prompts.unwrap_or(0),
        total_tool_uses: summary_row.tool_uses.unwrap_or(0),
        total_errors: summary_row.errors.unwrap_or(0),
        health_score,
        top_recommendation: top_rec,
    }
}

#[derive(Debug, sqlx::FromRow)]
struct HealthAnalysisRow {
    pub total: i64,
    pub avg_quality: Option<f64>,
    pub achieved: i64,
    pub failed: i64,
    pub sessions_with_errors: i64,
}

async fn fetch_health_analysis_metrics(pool: &PgPool, user_id: &UserId) -> HealthAnalysisRow {
    sqlx::query_as!(
        HealthAnalysisRow,
        r#"SELECT
            COUNT(*)::BIGINT AS "total!",
            AVG(quality_score::DOUBLE PRECISION) AS avg_quality,
            COUNT(*) FILTER (WHERE goal_achieved = 'yes')::BIGINT AS "achieved!",
            COUNT(*) FILTER (WHERE goal_achieved = 'no')::BIGINT AS "failed!",
            COUNT(*) FILTER (WHERE error_analysis IS NOT NULL AND error_analysis != '')::BIGINT AS "sessions_with_errors!"
          FROM session_analyses
          WHERE user_id = $1
            AND created_at > NOW() - INTERVAL '30 days'"#,
        user_id.as_str(),
    )
    .fetch_one(pool)
    .await
    .unwrap_or(HealthAnalysisRow {
        total: 0,
        avg_quality: None,
        achieved: 0,
        failed: 0,
        sessions_with_errors: 0,
    })
}

#[derive(Debug, sqlx::FromRow)]
struct SummaryMetricsRow {
    pub prompts: Option<i64>,
    pub tool_uses: Option<i64>,
    pub errors: Option<i64>,
    pub sessions: i64,
}

async fn fetch_health_summary_metrics(pool: &PgPool, user_id: &UserId) -> SummaryMetricsRow {
    sqlx::query_as!(
        SummaryMetricsRow,
        r#"SELECT
            SUM(prompts)::BIGINT AS prompts,
            SUM(tool_uses)::BIGINT AS tool_uses,
            SUM(errors)::BIGINT AS errors,
            COUNT(*)::BIGINT AS "sessions!"
          FROM plugin_session_summaries
          WHERE user_id = $1
            AND created_at > NOW() - INTERVAL '30 days'"#,
        user_id.as_str(),
    )
    .fetch_one(pool)
    .await
    .unwrap_or(SummaryMetricsRow {
        prompts: None,
        tool_uses: None,
        errors: None,
        sessions: 0,
    })
}

async fn fetch_latest_recommendation(pool: &PgPool, user_id: &UserId) -> String {
    sqlx::query_scalar!(
        r"SELECT recommendations
          FROM session_analyses
          WHERE user_id = $1
            AND recommendations IS NOT NULL AND recommendations != ''
          ORDER BY created_at DESC
          LIMIT 1",
        user_id.as_str(),
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .flatten()
    .unwrap_or_default()
}

fn compute_health_score(avg_quality: f64, goals_achieved: i64, total_sessions: i64) -> i64 {
    use crate::admin::numeric;

    let quality_component = if avg_quality > 0.0 {
        (avg_quality / 5.0) * 55.0
    } else {
        27.5
    };

    let goal_rate = if total_sessions > 0 {
        numeric::to_f64(goals_achieved) / numeric::to_f64(total_sessions)
    } else {
        0.5
    };
    let goal_component = goal_rate * 45.0;

    let raw = (quality_component + goal_component)
        .clamp(0.0, 100.0)
        .round();
    #[expect(
        clippy::cast_possible_truncation,
        reason = "clamped 0..=100, .round() guarantees integer value"
    )]
    let health_score = raw as i64;
    health_score
}
