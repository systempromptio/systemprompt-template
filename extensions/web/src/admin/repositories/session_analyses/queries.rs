use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::SessionAnalysisRow;

pub async fn fetch_session_analysis(pool: &PgPool, session_id: &str) -> Option<SessionAnalysisRow> {
    sqlx::query_as!(
        SessionAnalysisRow,
        r"SELECT session_id, title, description, summary, tags,
                 goal_achieved, quality_score, outcome, error_analysis,
                 skill_assessment, recommendations, skill_scores,
                 category, goal_outcome_map, efficiency_metrics,
                 best_practices_checklist, improvement_hints,
                 corrections_count, session_duration_minutes, total_turns
          FROM session_analyses
          WHERE session_id = $1",
        session_id,
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

pub async fn fetch_analysed_session_ids(
    pool: &PgPool,
    session_ids: &[String],
) -> std::collections::HashSet<String> {
    if session_ids.is_empty() {
        return std::collections::HashSet::new();
    }
    let rows = sqlx::query_scalar!(
        r"SELECT session_id FROM session_analyses WHERE session_id = ANY($1)",
        session_ids,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();
    rows.into_iter().collect()
}

pub async fn fetch_session_analyses_batch(
    pool: &PgPool,
    session_ids: &[String],
) -> Vec<SessionAnalysisRow> {
    if session_ids.is_empty() {
        return Vec::new();
    }
    sqlx::query_as!(
        SessionAnalysisRow,
        r"SELECT session_id, title, description, summary, tags,
                 goal_achieved, quality_score, outcome, error_analysis,
                 skill_assessment, recommendations, skill_scores,
                 category, goal_outcome_map, efficiency_metrics,
                 best_practices_checklist, improvement_hints,
                 corrections_count, session_duration_minutes, total_turns
          FROM session_analyses
          WHERE session_id = ANY($1)",
        session_ids,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

pub async fn fetch_recent_analyses(
    pool: &PgPool,
    user_id: &UserId,
    limit: i64,
) -> Vec<SessionAnalysisRow> {
    sqlx::query_as!(
        SessionAnalysisRow,
        r"SELECT session_id, title, description, summary, tags,
                 goal_achieved, quality_score, outcome, error_analysis,
                 skill_assessment, recommendations, skill_scores,
                 category, goal_outcome_map, efficiency_metrics,
                 best_practices_checklist, improvement_hints,
                 corrections_count, session_duration_minutes, total_turns
          FROM session_analyses
          WHERE user_id = $1
          ORDER BY created_at DESC
          LIMIT $2",
        user_id.as_str(),
        limit,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}
