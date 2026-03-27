use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug, Default)]
pub struct TodaySummary {
    pub sessions_count: i64,
    pub analysed_count: i64,
    pub avg_quality: f64,
    pub goals_achieved: i64,
    pub goals_partial: i64,
    pub goals_failed: i64,
    pub new_achievements: Vec<String>,
    pub top_recommendation: String,
}

#[derive(Debug, sqlx::FromRow)]
struct SessionCountRow {
    pub count: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct AnalysisMetricsRow {
    pub total: i64,
    pub avg_quality: Option<f64>,
    pub achieved: i64,
    pub partial: i64,
    pub failed: i64,
}

pub async fn fetch_today_summary(pool: &PgPool, user_id: &UserId) -> TodaySummary {
    let session_row = sqlx::query_as!(
        SessionCountRow,
        r#"SELECT COUNT(*)::BIGINT AS "count!"
          FROM plugin_session_summaries
          WHERE user_id = $1 AND created_at::date = CURRENT_DATE"#,
        user_id.as_str(),
    )
    .fetch_one(pool)
    .await
    .unwrap_or(SessionCountRow { count: 0 });

    let analysis_row = sqlx::query_as!(
        AnalysisMetricsRow,
        r#"SELECT
            COUNT(*)::BIGINT AS "total!",
            AVG(quality_score::DOUBLE PRECISION) AS avg_quality,
            COUNT(*) FILTER (WHERE goal_achieved = 'yes')::BIGINT AS "achieved!",
            COUNT(*) FILTER (WHERE goal_achieved = 'partial')::BIGINT AS "partial!",
            COUNT(*) FILTER (WHERE goal_achieved = 'no')::BIGINT AS "failed!"
          FROM session_analyses
          WHERE user_id = $1 AND created_at::date = CURRENT_DATE"#,
        user_id.as_str(),
    )
    .fetch_one(pool)
    .await
    .unwrap_or(AnalysisMetricsRow {
        total: 0,
        avg_quality: None,
        achieved: 0,
        partial: 0,
        failed: 0,
    });

    let new_achievements = sqlx::query_scalar!(
        r"SELECT achievement_id FROM user_achievements
          WHERE user_id = $1 AND unlocked_at::date = CURRENT_DATE
          ORDER BY unlocked_at DESC",
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
    .unwrap_or_else(|_| Vec::new());

    let top_rec = sqlx::query_scalar!(
        r"SELECT recommendations FROM session_analyses
          WHERE user_id = $1 AND created_at::date = CURRENT_DATE
            AND recommendations IS NOT NULL AND recommendations != ''
          ORDER BY created_at DESC LIMIT 1",
        user_id.as_str(),
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| {
        tracing::warn!(error = %e, user_id = %user_id.as_str(), "Failed to fetch today's top recommendation");
    })
    .ok()
    .flatten()
    .flatten()
    .unwrap_or_else(String::new);

    TodaySummary {
        sessions_count: session_row.count,
        analysed_count: analysis_row.total,
        avg_quality: analysis_row.avg_quality.unwrap_or(0.0),
        goals_achieved: analysis_row.achieved,
        goals_partial: analysis_row.partial,
        goals_failed: analysis_row.failed,
        new_achievements,
        top_recommendation: top_rec,
    }
}
