use std::sync::Arc;

use systemprompt::identifiers::UserId;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::{IntoResponse, Response},
};
use sqlx::PgPool;

use super::types::{EnrichedAchievementView, EnrichedUserView, UserDetailPageData, UsersPageData};
use crate::admin::repositories::UserRank;

fn enrich_users_with_ranks(
    users: &[crate::admin::types::UserSummary],
    rank_rows: &[UserRank],
) -> Vec<EnrichedUserView> {
    let rank_map: std::collections::HashMap<&str, &UserRank> =
        rank_rows.iter().map(|r| (r.user_id.as_str(), r)).collect();

    users
        .iter()
        .map(|u| {
            let (rank_name, xp) = rank_map
                .get(u.user_id.as_str())
                .map_or(("-".to_string(), 0), |rank| {
                    (rank.rank_name.clone(), rank.total_xp)
                });
            EnrichedUserView {
                user_id: u.user_id.to_string(),
                display_name: u.display_name.clone(),
                email: u.email.as_ref().map(std::string::ToString::to_string),
                roles: u.roles.clone(),
                is_active: u.is_active,
                last_active: u.last_active.to_rfc3339(),
                total_events: u.total_events,
                last_tool: u.last_tool.clone(),
                custom_skills_count: u.custom_skills_count,
                preferred_client: u.preferred_client.clone(),
                prompts: u.prompts,
                sessions: u.sessions,
                bytes: u.bytes,
                logins: u.logins,
                rank_name,
                xp,
            }
        })
        .collect()
}

pub(crate) async fn users_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            axum::http::StatusCode::FORBIDDEN,
            axum::response::Html(super::ACCESS_DENIED_HTML),
        )
            .into_response();
    }

    let users = repositories::list_users(&pool).await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list users");
        vec![]
    });

    let total_users = users.len();
    let active_users = users.iter().filter(|u| u.is_active).count();
    let total_events: i64 = users.iter().map(|u| u.total_events).sum();

    let rank_rows: Vec<UserRank> = repositories::fetch_user_ranks(&pool)
        .await
        .unwrap_or_else(|_| Vec::new());

    let enriched_users = enrich_users_with_ranks(&users, &rank_rows);

    let data = UsersPageData {
        page: "users",
        title: "Users",
        users: enriched_users,
        total_users,
        active_users,
        total_events,
    };

    let mut value = serde_json::to_value(&data).unwrap_or(serde_json::Value::Null);
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "page_stats".to_string(),
            serde_json::json!([
                {"value": total_users, "label": "Users"},
                {"value": active_users, "label": "Active"},
                {"value": total_events, "label": "Events"},
            ]),
        );
    }
    super::render_page(&engine, "users", &value, &user_ctx, &mkt_ctx)
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
        let data = UserDetailPageData {
            page: "user-detail",
            title: "User Detail",
            user: None,
            gamification: None,
            enriched_achievements: vec![],
            achievements_count: 0,
            not_found: true,
        };
        let value = serde_json::to_value(&data).unwrap_or(serde_json::Value::Null);
        return super::render_page(&engine, "user-detail", &value, &user_ctx, &mkt_ctx);
    };
    let user_id = UserId::new(id.as_str());

    let detail = repositories::find_user_detail(&pool, &user_id)
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, user_id = %user_id, "Failed to fetch user detail");
        })
        .ok()
        .flatten();
    let gamification =
        crate::admin::gamification::queries::find_user_gamification(&pool, user_id.as_str())
            .await
            .map_err(|e| {
                tracing::warn!(error = %e, user_id = %user_id, "Failed to fetch user gamification");
            })
            .ok()
            .flatten();

    let enriched_achievements = enrich_achievements(gamification.as_ref());
    let achievements_count = enriched_achievements.len();
    let not_found = detail.is_none();

    let data = UserDetailPageData {
        page: "user-detail",
        title: "User Detail",
        user: detail,
        gamification,
        enriched_achievements,
        achievements_count,
        not_found,
    };
    let value = serde_json::to_value(&data).unwrap_or(serde_json::Value::Null);
    super::render_page(&engine, "user-detail", &value, &user_ctx, &mkt_ctx)
}

fn enrich_achievements(
    gamification: Option<&crate::admin::types::UserGamificationProfile>,
) -> Vec<EnrichedAchievementView> {
    gamification
        .map(|g| {
            let defs = crate::admin::gamification::ACHIEVEMENTS;
            g.achievements
                .iter()
                .filter_map(|ua| {
                    defs.iter().find(|d| d.id == ua.achievement_id).map(|d| {
                        EnrichedAchievementView {
                            achievement_id: ua.achievement_id.clone(),
                            name: d.name,
                            description: d.description,
                            category: d.category,
                            unlocked_at: ua.unlocked_at,
                        }
                    })
                })
                .collect()
        })
        .unwrap_or_else(Vec::new)
}
