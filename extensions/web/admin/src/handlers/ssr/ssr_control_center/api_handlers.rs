use std::sync::Arc;

use crate::handlers::hooks_track::ai_summary;
use crate::repositories::control_center;
use crate::tier_enforcement::{check_limit, TierEnforcementCache};
use crate::tier_limits::{Feature, LimitCheck};
use crate::types::UserContext;
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use sqlx::PgPool;
use systemprompt::ai::AiService;
use systemprompt::identifiers::SessionId;

pub async fn handle_rate_session(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<crate::types::conversation_analytics::RateSessionRequest>,
) -> Response {
    if req.rating < 1 || req.rating > 5 {
        return StatusCode::BAD_REQUEST.into_response();
    }
    match crate::repositories::conversation_analytics::upsert_session_rating(
        &pool,
        &user_ctx.user_id,
        &req.session_id,
        req.rating,
        &req.outcome,
        &req.notes,
    )
    .await
    {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => {
            tracing::warn!(error = %e, "Failed to rate session");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn handle_rate_skill(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<crate::types::conversation_analytics::RateSkillRequest>,
) -> Response {
    if req.rating < 1 || req.rating > 5 {
        return StatusCode::BAD_REQUEST.into_response();
    }
    match crate::repositories::conversation_analytics::upsert_skill_rating(
        &pool,
        &user_ctx.user_id,
        &req.skill_name,
        req.rating,
        &req.notes,
    )
    .await
    {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => {
            tracing::warn!(error = %e, "Failed to rate skill");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn handle_update_session_status(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<crate::types::control_center::UpdateSessionStatusRequest>,
) -> Response {
    let valid = ["active", "completed", "deleted"];
    if !valid.contains(&req.status.as_str()) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    match control_center::update_session_status(
        &pool,
        &user_ctx.user_id,
        &req.session_id,
        &req.status,
    )
    .await
    {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => {
            tracing::warn!(error = %e, "Failed to update session status");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn handle_batch_update_session_status(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<crate::types::control_center::BatchUpdateSessionStatusRequest>,
) -> Response {
    let valid = ["active", "completed", "deleted"];
    if !valid.contains(&req.status.as_str()) || req.session_ids.is_empty() {
        return StatusCode::BAD_REQUEST.into_response();
    }
    for session_id in &req.session_ids {
        if let Err(e) =
            control_center::update_session_status(&pool, &user_ctx.user_id, session_id, &req.status)
                .await
        {
            tracing::warn!(error = %e, session_id = %session_id, "Failed to batch update session status");
        }
    }
    StatusCode::OK.into_response()
}

pub async fn handle_analyse_session(
    Extension(user_ctx): Extension<UserContext>,
    Extension(ai_service): Extension<Option<Arc<AiService>>>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<crate::types::control_center::AnalyseSessionRequest>,
) -> Response {
    let Some(ref ai) = ai_service else {
        return (StatusCode::SERVICE_UNAVAILABLE, "AI service not available").into_response();
    };

    let user_id = &user_ctx.user_id;
    let session_id = SessionId::new(&req.session_id);

    match ai_summary::run_analysis_for_session(&pool, ai, user_id, &session_id, "", None).await {
        Some(analysis) => {

            let goal_outcome_map_json = analysis
                .goal_outcome_map
                .as_ref()
                .and_then(|v| serde_json::to_value(v).ok());
            let efficiency_metrics_json = analysis
                .efficiency_metrics
                .as_ref()
                .and_then(|v| serde_json::to_value(v).ok());
            let best_practices_json = analysis
                .best_practices_checklist
                .as_ref()
                .and_then(|v| serde_json::to_value(v).ok());
            Json(json!({
                "status": "ok",
                "session_id": req.session_id,
                "title": analysis.title,
                "description": analysis.description,
                "goal_summary": analysis.goal_summary,
                "outcomes": analysis.outcomes,
                "tags": analysis.tags,
                "goal_achieved": analysis.goal_achieved,
                "quality_score": analysis.quality_score,
                "outcome": analysis.outcome,
                "category": analysis.category,
                "error_analysis": analysis.error_analysis,
                "skill_assessment": analysis.skill_assessment,
                "recommendations": analysis.recommendations,
                "goal_outcome_map": goal_outcome_map_json,
                "efficiency_metrics": efficiency_metrics_json,
                "best_practices_checklist": best_practices_json,
                "improvement_hints": analysis.improvement_hints,
            }))
            .into_response()
        }
        None => {

            (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({
                    "error": "analysis_failed",
                    "message": "AI could not generate analysis. The session may lack sufficient context."
                })),
            )
                .into_response()
        }
    }
}

pub async fn handle_generate_report(
    Extension(user_ctx): Extension<UserContext>,
    Extension(ai_service): Extension<Option<Arc<AiService>>>,
    Extension(tier_cache): Extension<TierEnforcementCache>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let check = check_limit(
        &tier_cache,
        &pool,
        &user_ctx.user_id,
        LimitCheck::FeatureAccess(Feature::AiDailySummaries),
    )
    .await;
    if !check.allowed {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "feature_denied",
                "message": check.reason
            })),
        )
            .into_response();
    }

    let Some(ref ai) = ai_service else {
        return (StatusCode::SERVICE_UNAVAILABLE, "AI service not available").into_response();
    };

    let today = chrono::Utc::now().date_naive();
    match crate::repositories::daily_summaries::generate_user_daily_summary(
        &pool,
        user_ctx.user_id.as_str(),
        today,
        Some(ai),
    )
    .await
    {
        Ok(()) => Json(json!({"status": "ok"})).into_response(),
        Err(e) => {
            tracing::warn!(error = %e, "Failed to generate daily report");

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "generation_failed", "message": e.to_string()})),
            )
                .into_response()
        }
    }
}
