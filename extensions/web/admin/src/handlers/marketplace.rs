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
use crate::types::{MarketplacePlugin, MarketplaceQuery, UpdateVisibilityRequest, UserContext};

use super::responses::{MarketplaceListResponse, RulesResponse};

pub async fn list_marketplace_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<MarketplaceQuery>,
) -> Response {
    match build_marketplace_list(&pool, &user_ctx, &query).await {
        Ok(response) => Json(response).into_response(),
        Err(r) => r,
    }
}

async fn build_marketplace_list(
    pool: &PgPool,
    user_ctx: &UserContext,
    query: &MarketplaceQuery,
) -> Result<MarketplaceListResponse<Vec<MarketplacePlugin>>, Response> {
    let services_path = shared::get_services_path().map_err(|r| *r)?;

    let plugins =
        repositories::list_plugins_for_roles(&services_path, &user_ctx.roles).map_err(|e| {
            tracing::error!(error = %e, "Failed to list plugins");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        })?;

    let (ratings_map, visibility_rules) = fetch_marketplace_data(pool).await?;

    let mut marketplace_plugins: Vec<MarketplacePlugin> = plugins
        .iter()
        .map(|plugin| {
            build_marketplace_plugin(plugin, &services_path, &ratings_map, &visibility_rules)
        })
        .collect();

    sort_and_filter(&mut marketplace_plugins, query);

    Ok(MarketplaceListResponse {
        plugins: marketplace_plugins,
    })
}

type MarketplaceData = (
    HashMap<String, crate::types::PluginRatingAggregate>,
    HashMap<String, Vec<crate::types::VisibilityRule>>,
);

async fn fetch_marketplace_data(pool: &PgPool) -> Result<MarketplaceData, Response> {
    let (ratings_res, rules_res) = tokio::join!(
        repositories::list_plugin_ratings(pool),
        repositories::list_visibility_rules(pool),
    );

    let ratings_map: HashMap<String, _> = ratings_res
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to get plugin ratings");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        })?
        .into_iter()
        .map(|r| (r.plugin_id.clone(), r))
        .collect();

    let visibility_rules: HashMap<String, Vec<_>> = {
        let mut m: HashMap<String, Vec<_>> = HashMap::new();
        for rule in rules_res.map_err(|e| {
            tracing::error!(error = %e, "Failed to get visibility rules");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        })? {
            m.entry(rule.plugin_id.clone()).or_default().push(rule);
        }
        m
    };

    Ok((ratings_map, visibility_rules))
}

fn build_marketplace_plugin(
    plugin: &crate::types::PluginOverview,
    services_path: &std::path::Path,
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

    let rating = ratings_map.get(&plugin.id);
    let plugin_rules = visibility_rules
        .get(&plugin.id)
        .cloned()
        .unwrap_or_else(Vec::new);

    let avg_rating = rating.map_or(0.0, |r| r.avg_rating);
    let rating_count = rating.map_or(0, |r| r.rating_count);

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
        avg_rating,
        rating_count,
    }
}

fn sort_and_filter(marketplace_plugins: &mut Vec<MarketplacePlugin>, query: &MarketplaceQuery) {
    let sort = query.sort.as_deref().unwrap_or("rating");
    match sort {
        "name" => marketplace_plugins.sort_unstable_by(|a, b| a.name.cmp(&b.name)),
        _ => marketplace_plugins.sort_unstable_by(|a, b| {
            b.avg_rating
                .partial_cmp(&a.avg_rating)
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

pub async fn update_visibility_handler(
    State(pool): State<Arc<PgPool>>,
    Path(plugin_id): Path<String>,
    Json(body): Json<UpdateVisibilityRequest>,
) -> Response {
    match repositories::set_visibility_rules(&pool, &plugin_id, &body.rules).await {
        Ok(rules) => Json(RulesResponse { rules }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, plugin_id, "Failed to update visibility rules");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        }
    }
}
