use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use crate::handlers::shared;
use crate::repositories;
use crate::types::{
    MarketplacePlugin, MarketplaceQuery, SubmitRatingRequest, UpdateVisibilityRequest, UserContext,
};

use super::responses::{MarketplaceListResponse, RulesResponse, UsersListResponse};

pub async fn list_marketplace_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<MarketplaceQuery>,
) -> Response {
    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let plugins = match repositories::list_plugins_for_roles(&services_path, &user_ctx.roles) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "Failed to list plugins");
            return shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
        }
    };

    let (usage_res, ratings_res, rules_res) = tokio::join!(
        repositories::get_all_plugin_usage(&pool),
        repositories::get_all_plugin_ratings(&pool),
        repositories::get_all_visibility_rules(&pool),
    );

    let usage_map: HashMap<String, _> = usage_res
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to get plugin usage");
            vec![]
        })
        .into_iter()
        .map(|u| (u.plugin_id.clone(), u))
        .collect();
    let ratings_map: HashMap<String, _> = ratings_res
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to get plugin ratings");
            vec![]
        })
        .into_iter()
        .map(|r| (r.plugin_id.clone(), r))
        .collect();
    let visibility_rules: HashMap<String, Vec<_>> = {
        let mut m: HashMap<String, Vec<_>> = HashMap::new();
        for rule in rules_res.unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to get visibility rules");
            vec![]
        }) {
            m.entry(rule.plugin_id.clone()).or_default().push(rule);
        }
        m
    };

    let mut marketplace_plugins: Vec<MarketplacePlugin> = plugins
        .iter()
        .map(|plugin| {
            build_marketplace_plugin(
                plugin,
                &services_path,
                &usage_map,
                &ratings_map,
                &visibility_rules,
            )
        })
        .collect();

    sort_and_filter(&mut marketplace_plugins, &query);

    Json(MarketplaceListResponse {
        plugins: marketplace_plugins,
    })
    .into_response()
}

fn build_marketplace_plugin(
    plugin: &crate::types::PluginOverview,
    services_path: &std::path::Path,
    usage_map: &HashMap<String, crate::types::PluginUsageAggregate>,
    ratings_map: &HashMap<String, crate::types::PluginRatingAggregate>,
    visibility_rules: &HashMap<String, Vec<crate::types::VisibilityRule>>,
) -> MarketplacePlugin {
    let detail = repositories::find_plugin_detail(services_path, &plugin.id);
    let (version, category, keywords, author, roles, enabled) = match detail {
        Ok(Some(d)) => (
            d.version,
            d.category,
            d.keywords,
            d.author_name,
            d.roles,
            d.enabled,
        ),
        _ => (
            "0.0.0".into(),
            String::new(),
            vec![],
            String::new(),
            vec![],
            true,
        ),
    };

    let usage = usage_map.get(&plugin.id);
    let rating = ratings_map.get(&plugin.id);
    let plugin_rules = visibility_rules
        .get(&plugin.id)
        .cloned()
        .unwrap_or_else(Vec::new);

    let total_events = usage.map_or(0, |u| u.total_events);
    let unique_users = usage.map_or(0, |u| u.unique_users);
    let active_7d = usage.map_or(0, |u| u.active_users_7d);
    let active_30d = usage.map_or(0, |u| u.active_users_30d);
    let avg_rating = rating.map_or(0.0, |r| r.avg_rating);
    let rating_count = rating.map_or(0, |r| r.rating_count);

    let rank_score = compute_rank_score(total_events, active_30d, avg_rating, rating_count);

    MarketplacePlugin {
        id: plugin.id.clone(),
        name: plugin.name.clone(),
        description: plugin.description.clone(),
        version,
        category,
        keywords,
        author_name: author,
        enabled,
        skill_count: plugin.skills.len(),
        agent_count: plugin.agents.len(),
        mcp_server_count: plugin.mcp_servers.len(),
        hook_count: 0,
        roles,
        visibility_rules: plugin_rules,
        total_events,
        unique_users,
        active_users_7d: active_7d,
        active_users_30d: active_30d,
        avg_rating,
        rating_count,
        rank_score,
    }
}

fn sort_and_filter(marketplace_plugins: &mut Vec<MarketplacePlugin>, query: &MarketplaceQuery) {
    let sort = query.sort.as_deref().unwrap_or("rank");
    match sort {
        "rating" => marketplace_plugins.sort_unstable_by(|a, b| {
            b.avg_rating
                .partial_cmp(&a.avg_rating)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
        "usage" => marketplace_plugins.sort_unstable_by(|a, b| b.total_events.cmp(&a.total_events)),
        "name" => marketplace_plugins.sort_unstable_by(|a, b| a.name.cmp(&b.name)),
        _ => marketplace_plugins.sort_unstable_by(|a, b| {
            b.rank_score
                .partial_cmp(&a.rank_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        }),
    }

    if let Some(ref search) = query.search {
        let q = search.to_lowercase();
        marketplace_plugins.retain(|p| {
            p.name.to_lowercase().contains(&q)
                || p.description.to_lowercase().contains(&q)
                || p.category.to_lowercase().contains(&q)
        });
    }

    if let Some(ref category) = query.category {
        marketplace_plugins.retain(|p| p.category == *category);
    }
}

pub async fn marketplace_plugin_users_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
) -> Response {
    match repositories::get_plugin_users(&pool, &plugin_id).await {
        Ok(users) => Json(UsersListResponse { users }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, plugin_id, "Failed to get plugin users");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub async fn submit_rating_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    Json(body): Json<SubmitRatingRequest>,
) -> Response {
    if body.rating < 1 || body.rating > 5 {
        return shared::error_response(StatusCode::BAD_REQUEST, "Rating must be between 1 and 5");
    }
    match repositories::upsert_rating(
        &pool,
        &plugin_id,
        &body.user_id,
        body.rating,
        body.review.as_deref(),
    )
    .await
    {
        Ok(rating) => (StatusCode::CREATED, Json(rating)).into_response(),
        Err(e) => {
            tracing::error!(error = %e, plugin_id, "Failed to submit rating");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub async fn update_visibility_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    Json(body): Json<UpdateVisibilityRequest>,
) -> Response {
    match repositories::set_visibility_rules(&pool, &plugin_id, &body.rules).await {
        Ok(rules) => Json(RulesResponse { rules }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, plugin_id, "Failed to update visibility rules");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

fn compute_rank_score(
    total_events: i64,
    active_30d: i64,
    avg_rating: f64,
    rating_count: i64,
) -> f64 {
    let usage_score = f64::from(i32::try_from(total_events).unwrap_or(0)).ln_1p();
    let active_score = f64::from(i32::try_from(active_30d).unwrap_or(0)).ln_1p();
    let rc_f = f64::from(i32::try_from(rating_count).unwrap_or(0));
    let bayesian_rating = avg_rating.mul_add(rc_f, 3.0 * 5.0) / (rc_f + 5.0);
    0.4f64.mul_add(usage_score, 0.3f64.mul_add(active_score, 0.3 * bayesian_rating))
}
