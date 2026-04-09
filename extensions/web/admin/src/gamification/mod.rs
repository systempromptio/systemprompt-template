mod achievements;
pub mod constants;
pub mod queries;
mod queries_leaderboard;
mod recalculate;
mod recalculate_helpers;

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use super::types::{DepartmentQuery, PaginationQuery};

pub use constants::{
    rank_for_xp, xp_to_next_rank, AchievementDef, ACHIEVEMENTS, CUSTOM_SKILL_XP, ERROR_XP,
    FIRST_UNIQUE_SKILL_XP, PROMPT_XP, RANKS, SESSION_XP, STREAK_BONUS_XP, SUBAGENT_XP,
    TOKEN_XP_PER_1K, TOOL_USE_XP,
};
pub use recalculate::recalculate_all;

pub async fn leaderboard_handler(
    State(pool): State<Arc<PgPool>>,
    Query(pagination): Query<PaginationQuery>,
) -> Response {
    match queries::get_leaderboard(&pool, pagination.limit, pagination.offset, None).await {
        Ok(entries) => Json(entries).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get leaderboard");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub async fn department_leaderboard_handler(
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<DepartmentQuery>,
) -> Response {
    let dept = match &query.dept {
        Some(d) => d.clone(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "dept query parameter required"})),
            )
                .into_response()
        }
    };
    match queries::get_department_leaderboard(&pool, &dept).await {
        Ok(entries) => Json(entries).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get department leaderboard");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub async fn user_gamification_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<String>,
) -> Response {
    match queries::get_user_gamification(&pool, &user_id).await {
        Ok(Some(profile)) => Json(profile).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "User gamification profile not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "Failed to get user gamification");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub async fn achievements_handler(State(pool): State<Arc<PgPool>>) -> Response {
    match queries::get_achievement_stats(&pool).await {
        Ok(stats) => Json(stats).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get achievement stats");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub async fn departments_handler(State(pool): State<Arc<PgPool>>) -> Response {
    match queries::get_department_scores(&pool, None).await {
        Ok(scores) => Json(scores).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get department scores");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub async fn recalculate_handler(State(pool): State<Arc<PgPool>>) -> Response {
    match recalculate_all(&pool).await {
        Ok(count) => Json(serde_json::json!({
            "ok": true,
            "users_updated": count
        }))
        .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to recalculate gamification");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}
