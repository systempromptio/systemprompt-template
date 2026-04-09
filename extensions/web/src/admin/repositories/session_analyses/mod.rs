mod health;
mod queries;
mod today_summary;

pub use health::{fetch_health_metrics, HealthMetrics};
pub use queries::{
    fetch_analysed_session_ids, fetch_recent_analyses, fetch_session_analyses_batch,
    fetch_session_analysis,
};
pub use today_summary::{fetch_today_summary, TodaySummary};

use sqlx::PgPool;

use crate::admin::handlers::hooks_track::ai_summary::SessionAnalysis;

pub type SessionAnalysisDetail = SessionAnalysisRow;

#[derive(Debug, sqlx::FromRow)]
pub struct SessionAnalysisRow {
    pub session_id: String,
    pub title: String,
    pub description: String,
    pub summary: String,
    pub tags: String,
    pub goal_achieved: String,
    pub quality_score: i16,
    pub outcome: String,
    pub error_analysis: Option<String>,
    pub skill_assessment: Option<String>,
    pub recommendations: Option<String>,
    pub skill_scores: Option<serde_json::Value>,
    pub category: String,
    pub goal_outcome_map: Option<serde_json::Value>,
    pub efficiency_metrics: Option<serde_json::Value>,
    pub best_practices_checklist: Option<serde_json::Value>,
    pub improvement_hints: Option<String>,
    pub corrections_count: i32,
    pub session_duration_minutes: Option<i32>,
    pub total_turns: Option<i32>,
}

struct InsertParams {
    tags: String,
    composed_summary: String,
    skill_scores_json: Option<serde_json::Value>,
    category: String,
    goal_outcome_map_json: Option<serde_json::Value>,
    efficiency_metrics_json: Option<serde_json::Value>,
    best_practices_json: Option<serde_json::Value>,
    corrections_count: i32,
    duration_minutes: Option<i32>,
    total_turns: Option<i32>,
    automation_ratio: Option<f32>,
    plan_mode_used: bool,
    client_surface: String,
}

fn prepare_insert_params(analysis: &SessionAnalysis) -> InsertParams {
    InsertParams {
        tags: analysis.tags.join(","),
        composed_summary: analysis.composed_summary(),
        skill_scores_json: analysis
            .skill_scores
            .as_ref()
            .and_then(|s| serde_json::to_value(s).ok()),
        category: analysis.category.as_deref().unwrap_or("other").to_string(),
        goal_outcome_map_json: analysis
            .goal_outcome_map
            .as_ref()
            .and_then(|v| serde_json::to_value(v).ok()),
        efficiency_metrics_json: analysis
            .efficiency_metrics
            .as_ref()
            .and_then(|v| serde_json::to_value(v).ok()),
        best_practices_json: analysis
            .best_practices_checklist
            .as_ref()
            .and_then(|v| serde_json::to_value(v).ok()),
        corrections_count: analysis
            .efficiency_metrics
            .as_ref()
            .map_or(0, |e| e.corrections_count),
        duration_minutes: analysis
            .efficiency_metrics
            .as_ref()
            .map(|e| e.duration_minutes),
        total_turns: analysis.efficiency_metrics.as_ref().map(|e| e.total_turns),
        automation_ratio: analysis.automation_ratio,
        plan_mode_used: analysis.plan_mode_used.unwrap_or(false),
        client_surface: analysis.client_surface.as_deref().unwrap_or("").to_string(),
    }
}

pub async fn insert_session_analysis(
    pool: &PgPool,
    session_id: &str,
    user_id: &str,
    analysis: &SessionAnalysis,
) {
    let p = prepare_insert_params(analysis);

    tracing::debug!(
        session_id,
        quality_score = analysis.quality_score,
        goal_achieved = %analysis.goal_achieved,
        "Inserting session analysis"
    );

    if let Err(e) = execute_upsert_analysis(pool, session_id, user_id, analysis, &p).await {
        tracing::warn!(error = %e, "Failed to insert session analysis");
    }
}

struct UpsertAnalysisIds<'a> {
    session_id: &'a str,
    user_id: &'a str,
}

async fn execute_upsert_analysis(
    pool: &PgPool,
    session_id: &str,
    user_id: &str,
    analysis: &SessionAnalysis,
    p: &InsertParams,
) -> Result<(), sqlx::Error> {
    let ids = UpsertAnalysisIds { session_id, user_id };
    run_upsert_query(pool, &ids, analysis, p).await
}

async fn run_upsert_query(
    pool: &PgPool,
    ids: &UpsertAnalysisIds<'_>,
    analysis: &SessionAnalysis,
    p: &InsertParams,
) -> Result<(), sqlx::Error> {
    sqlx::query(SESSION_ANALYSIS_UPSERT_SQL)
        .bind(ids.session_id)
        .bind(ids.user_id)
        .bind(&analysis.title)
        .bind(&analysis.description)
        .bind(&p.composed_summary)
        .bind(&p.tags)
        .bind(&analysis.goal_achieved)
        .bind(analysis.quality_score)
        .bind(&analysis.outcome)
        .bind(&analysis.error_analysis)
        .bind(&analysis.skill_assessment)
        .bind(&analysis.recommendations)
        .bind(&p.skill_scores_json)
        .bind(&p.category)
        .bind(&p.goal_outcome_map_json)
        .bind(&p.efficiency_metrics_json)
        .bind(&p.best_practices_json)
        .bind(&analysis.improvement_hints)
        .bind(p.corrections_count)
        .bind(p.duration_minutes)
        .bind(p.total_turns)
        .bind(p.automation_ratio)
        .bind(p.plan_mode_used)
        .bind(&p.client_surface)
        .execute(pool)
        .await?;
    Ok(())
}

const SESSION_ANALYSIS_UPSERT_SQL: &str = r"INSERT INTO session_analyses
    (session_id, user_id, title, description, summary, tags,
     goal_achieved, quality_score, outcome, error_analysis,
     skill_assessment, recommendations, skill_scores,
     category, goal_outcome_map, efficiency_metrics,
     best_practices_checklist, improvement_hints,
     corrections_count, session_duration_minutes, total_turns,
     automation_ratio, plan_mode_used, client_surface)
  VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13,
          $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24)
  ON CONFLICT (session_id) DO UPDATE SET
    title = EXCLUDED.title,
    description = EXCLUDED.description,
    summary = EXCLUDED.summary,
    tags = EXCLUDED.tags,
    goal_achieved = EXCLUDED.goal_achieved,
    quality_score = EXCLUDED.quality_score,
    outcome = EXCLUDED.outcome,
    error_analysis = EXCLUDED.error_analysis,
    skill_assessment = EXCLUDED.skill_assessment,
    recommendations = EXCLUDED.recommendations,
    skill_scores = EXCLUDED.skill_scores,
    category = EXCLUDED.category,
    goal_outcome_map = EXCLUDED.goal_outcome_map,
    efficiency_metrics = EXCLUDED.efficiency_metrics,
    best_practices_checklist = EXCLUDED.best_practices_checklist,
    improvement_hints = EXCLUDED.improvement_hints,
    corrections_count = EXCLUDED.corrections_count,
    session_duration_minutes = EXCLUDED.session_duration_minutes,
    total_turns = EXCLUDED.total_turns,
    automation_ratio = EXCLUDED.automation_ratio,
    plan_mode_used = EXCLUDED.plan_mode_used,
    client_surface = EXCLUDED.client_surface,
    updated_at = NOW()";
