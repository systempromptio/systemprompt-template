use std::sync::Arc;

use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

pub(crate) async fn leaderboard_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let tab = params.get("tab").map_or("individual", String::as_str);

    if let Err(e) = crate::admin::gamification::recalculate_all(&pool).await {
        tracing::warn!(error = %e, "Failed to recalculate gamification on page load");
    }

    let leaderboard = crate::admin::gamification::queries::get_leaderboard(&pool, 100, 0, None)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to get leaderboard");
            vec![]
        });
    let departments = crate::admin::gamification::queries::get_department_scores(&pool, None)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to get department scores");
            vec![]
        });

    let data = json!({
        "page": "leaderboard",
        "title": "Leaderboard",
        "tab": tab,
        "tab_individual": tab == "individual",
        "tab_department": tab == "department",
        "leaderboard": leaderboard,
        "departments": departments,
    });
    super::render_page(&engine, "leaderboard", &data, &user_ctx, &mkt_ctx)
}

pub(crate) async fn achievements_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let achievements = crate::admin::gamification::queries::get_achievement_stats(&pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to get achievement stats");
            vec![]
        });
    let data = json!({
        "page": "achievements",
        "title": "Achievements",
        "achievements": achievements,
    });
    super::render_page(&engine, "achievements", &data, &user_ctx, &mkt_ctx)
}
