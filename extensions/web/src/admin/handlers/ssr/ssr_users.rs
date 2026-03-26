use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext, UsersQuery};
use axum::{
    extract::{Extension, Query, State},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

#[allow(clippy::too_many_lines)]
pub(crate) async fn users_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<UsersQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            axum::http::StatusCode::FORBIDDEN,
            axum::response::Html(super::ACCESS_DENIED_HTML),
        )
            .into_response();
    }

    let dept_filter = query.department.as_deref().filter(|d| *d != "Unassigned");

    let users = repositories::list_users(&pool, dept_filter)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list users");
            vec![]
        });

    let users = if query.department.as_deref() == Some("Unassigned") {
        users
            .into_iter()
            .filter(|u| u.department.is_none())
            .collect()
    } else {
        users
    };

    let total_users = users.len();
    let active_users = users.iter().filter(|u| u.is_active).count();
    let total_events: i64 = users.iter().map(|u| u.total_events).sum();
    let departments_count = {
        let mut depts: Vec<&str> = users
            .iter()
            .filter_map(|u| u.department.as_deref())
            .collect();
        depts.sort_unstable();
        depts.dedup();
        depts.len()
    };

    let dept_stats = repositories::fetch_department_stats(&pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch department stats");
            vec![]
        });

    let department_options: Vec<&str> = dept_stats.iter().map(|d| d.department.as_str()).collect();

    let dept_chart = super::compute_bar_chart(
        &dept_stats,
        |d| d.user_count,
        |d, pct| {
            let session_trend = match d.sessions_this_week.cmp(&d.sessions_prev_week) {
                std::cmp::Ordering::Greater => "up",
                std::cmp::Ordering::Less => "down",
                std::cmp::Ordering::Equal => "flat",
            };
            json!({
                "department": d.department,
                "user_count": d.user_count,
                "active_count": d.active_count,
                "total_events": d.total_events,
                "active_24h": d.active_24h,
                "active_7d": d.active_7d,
                "total_tokens": d.total_tokens,
                "total_prompts": d.total_prompts,
                "total_sessions": d.total_sessions,
                "sessions_this_week": d.sessions_this_week,
                "sessions_prev_week": d.sessions_prev_week,
                "session_trend": session_trend,
                "session_trend_up": session_trend == "up",
                "session_trend_down": session_trend == "down",
                "pct": pct,
            })
        },
    );

    #[derive(sqlx::FromRow)]
    #[allow(clippy::items_after_statements)]
    struct UserRank {
        user_id: String,
        rank_name: String,
        total_xp: i64,
    }
    let rank_rows: Vec<UserRank> = sqlx::query_as::<_, UserRank>(
        "SELECT user_id, rank_name, total_xp::BIGINT AS total_xp FROM employee_ranks",
    )
    .fetch_all(pool.as_ref())
    .await
    .unwrap_or_default();
    let rank_map: std::collections::HashMap<&str, &UserRank> =
        rank_rows.iter().map(|r| (r.user_id.as_str(), r)).collect();

    let enriched_users: Vec<serde_json::Value> = users
        .iter()
        .map(|u| {
            let mut val = serde_json::to_value(u).unwrap_or_default();
            if let Some(rank) = rank_map.get(u.user_id.as_str()) {
                val["rank_name"] = json!(rank.rank_name);
                val["xp"] = json!(rank.total_xp);
            } else {
                val["rank_name"] = json!("-");
                val["xp"] = json!(0);
            }
            val
        })
        .collect();

    let data = json!({
        "page": "users",
        "title": "Users",
        "users": enriched_users,
        "total_users": total_users,
        "active_users": active_users,
        "departments_count": departments_count,
        "total_events": total_events,
        "department_options": department_options,
        "selected_department": query.department,
        "dept_chart": dept_chart,
    });
    super::render_page(&engine, "users", &data, &user_ctx, &mkt_ctx)
}

pub(crate) async fn user_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    if !user_ctx.is_admin && Some(user_ctx.user_id.as_str()) != params.get("id").map(String::as_str)
    {
        return (
            axum::http::StatusCode::FORBIDDEN,
            axum::response::Html(
                "<h1>Access Denied</h1><p>You can only view your own profile.</p>",
            ),
        )
            .into_response();
    }

    let Some(id) = params.get("id") else {
        let data = json!({
            "page": "user-detail",
            "title": "User Detail",
            "not_found": true,
        });
        return super::render_page(&engine, "user-detail", &data, &user_ctx, &mkt_ctx);
    };
    let user_id = id.clone();

    let detail = repositories::get_user_detail(&pool, &user_id)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user_id, "Failed to fetch user detail");
        })
        .ok()
        .flatten();
    let gamification = crate::admin::gamification::queries::get_user_gamification(&pool, &user_id)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user_id, "Failed to fetch user gamification");
        })
        .ok()
        .flatten();

    let enriched_achievements: Vec<serde_json::Value> = gamification
        .as_ref()
        .map(|g| {
            let defs = crate::admin::gamification::ACHIEVEMENTS;
            g.achievements
                .iter()
                .filter_map(|ua| {
                    defs.iter().find(|d| d.id == ua.achievement_id).map(|d| {
                        json!({
                            "achievement_id": ua.achievement_id,
                            "name": d.name,
                            "description": d.description,
                            "category": d.category,
                            "unlocked_at": ua.unlocked_at,
                        })
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let data = json!({
        "page": "user-detail",
        "title": "User Detail",
        "user": detail,
        "gamification": gamification,
        "enriched_achievements": enriched_achievements,
        "achievements_count": enriched_achievements.len(),
        "not_found": detail.is_none(),
    });
    super::render_page(&engine, "user-detail", &data, &user_ctx, &mkt_ctx)
}
