use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use sqlx::PgPool;

use crate::repositories;
use crate::repositories::user_settings::UpsertUserSettings;
use crate::types::UserContext;

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub notify_daily_summary: Option<bool>,
    pub notify_achievements: Option<bool>,
    pub leaderboard_opt_in: Option<bool>,
    pub timezone: Option<String>,
}

pub async fn update_user_settings_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<UpdateSettingsRequest>,
) -> Response {
    let existing =
        repositories::user_settings::find_user_settings(&pool, &user_ctx.user_id)
            .await
            .map_err(|e| {
                tracing::warn!(error = %e, user_id = %user_ctx.user_id, "Failed to fetch existing user settings");
            })
            .ok()
            .flatten();

    let notify_daily_summary = body
        .notify_daily_summary
        .unwrap_or_else(|| existing.as_ref().is_none_or(|s| s.notify_daily_summary));
    let notify_achievements = body
        .notify_achievements
        .unwrap_or_else(|| existing.as_ref().is_none_or(|s| s.notify_achievements));
    let leaderboard_opt_in = body
        .leaderboard_opt_in
        .unwrap_or_else(|| existing.as_ref().is_none_or(|s| s.leaderboard_opt_in));
    let display_name = body
        .display_name
        .as_deref()
        .or_else(|| existing.as_ref().and_then(|s| s.display_name.as_deref()));
    let avatar_url = body
        .avatar_url
        .as_deref()
        .or_else(|| existing.as_ref().and_then(|s| s.avatar_url.as_deref()));
    let timezone = body
        .timezone
        .as_deref()
        .or_else(|| existing.as_ref().map(|s| s.timezone.as_str()))
        .unwrap_or("UTC");

    if timezone.parse::<chrono_tz::Tz>().is_err() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid timezone"})),
        )
            .into_response();
    }

    let input = UpsertUserSettings {
        user_id: &user_ctx.user_id,
        display_name,
        avatar_url,
        notify_daily_summary,
        notify_achievements,
        leaderboard_opt_in,
        timezone,
    };

    match repositories::user_settings::upsert_user_settings(&pool, &input).await {
        Ok(settings) => Json(serde_json::json!({
            "ok": true,
            "settings": settings,
        }))
        .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update user settings");
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to save settings"})),
            )
                .into_response()
        }
    }
}
