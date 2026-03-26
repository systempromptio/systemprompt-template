use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

pub(crate) async fn marketplace_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let services_path = super::get_services_path()
        .map_err(|_| {
            tracing::warn!("Failed to get services path for marketplace");
        })
        .ok();
    let all_roles = vec!["admin".to_string()];
    let plugins_list = services_path
        .as_ref()
        .and_then(|p| {
            repositories::list_plugins_for_roles(p, &all_roles)
                .map_err(|e| {
                    tracing::warn!(error = %e, "Failed to list plugins for roles");
                })
                .ok()
        })
        .unwrap_or_default();

    let marketplace_plugins =
        build_marketplace_plugins(services_path.as_ref(), &plugins_list, &pool).await;

    let data = json!({
        "page": "marketplace",
        "title": "All Plugins",
        "marketplace_plugins": marketplace_plugins,
    });
    super::render_page(&engine, "marketplace", &data, &user_ctx, &mkt_ctx)
}

async fn build_marketplace_plugins(
    services_path: Option<&std::path::PathBuf>,
    plugins_list: &[crate::admin::types::PluginOverview],
    pool: &Arc<PgPool>,
) -> Vec<serde_json::Value> {
    use crate::admin::types::VisibilityRule;

    let (usage_res, ratings_res, rules_res) = tokio::join!(
        repositories::get_all_plugin_usage(pool),
        repositories::get_all_plugin_ratings(pool),
        repositories::get_all_visibility_rules(pool),
    );

    let usage_map: std::collections::HashMap<String, _> = usage_res
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to get plugin usage");
            vec![]
        })
        .into_iter()
        .map(|u| (u.plugin_id.clone(), u))
        .collect();
    let ratings_map: std::collections::HashMap<String, _> = ratings_res
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to get plugin ratings");
            vec![]
        })
        .into_iter()
        .map(|r| (r.plugin_id.clone(), r))
        .collect();
    let vis_rules: std::collections::HashMap<String, Vec<VisibilityRule>> = {
        let mut m: std::collections::HashMap<String, Vec<VisibilityRule>> =
            std::collections::HashMap::new();
        for rule in rules_res.unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to get visibility rules");
            vec![]
        }) {
            m.entry(rule.plugin_id.clone()).or_default().push(rule);
        }
        m
    };

    let mut results: Vec<serde_json::Value> = plugins_list
        .iter()
        .map(|plugin| {
            build_plugin_json(plugin, services_path, &usage_map, &ratings_map, &vis_rules)
        })
        .collect();

    results.sort_by(|a, b| {
        let ra = a["rank_score"].as_f64().unwrap_or(0.0);
        let rb = b["rank_score"].as_f64().unwrap_or(0.0);
        rb.partial_cmp(&ra).unwrap_or(std::cmp::Ordering::Equal)
    });

    results
}

fn build_plugin_json(
    plugin: &crate::admin::types::PluginOverview,
    services_path: Option<&std::path::PathBuf>,
    usage_map: &std::collections::HashMap<String, crate::admin::types::PluginUsageAggregate>,
    ratings_map: &std::collections::HashMap<String, crate::admin::types::PluginRatingAggregate>,
    vis_rules: &std::collections::HashMap<String, Vec<crate::admin::types::VisibilityRule>>,
) -> serde_json::Value {
    let detail = services_path.and_then(|p| {
        repositories::get_plugin_detail(p, &plugin.id)
            .map_err(|e| {
                tracing::warn!(error = %e, plugin_id = %plugin.id, "Failed to fetch plugin detail");
            })
            .ok()
            .flatten()
    });
    let (version, category, roles, enabled) = match &detail {
        Some(d) => (
            d.version.clone(),
            d.category.clone(),
            d.roles.clone(),
            d.enabled,
        ),
        None => ("0.0.0".into(), String::new(), vec![], true),
    };
    let usage = usage_map.get(&plugin.id);
    let rating = ratings_map.get(&plugin.id);
    let plugin_rules = vis_rules.get(&plugin.id).cloned().unwrap_or_default();
    let total_events = usage.map_or(0, |u| u.total_events);
    let unique_users = usage.map_or(0, |u| u.unique_users);
    let active_7d = usage.map_or(0, |u| u.active_users_7d);
    let active_30d = usage.map_or(0, |u| u.active_users_30d);
    let avg_rating = rating.map_or(0.0, |r| r.avg_rating);
    let rating_count = rating.map_or(0, |r| r.rating_count);
    let rank_score = {
        let usage_score = (f64::from(i32::try_from(total_events).unwrap_or(0)) + 1.0).ln();
        let active_score = (f64::from(i32::try_from(active_30d).unwrap_or(0)) + 1.0).ln();
        let rc_f = f64::from(i32::try_from(rating_count).unwrap_or(0));
        let bayesian = (avg_rating * rc_f + 3.0 * 5.0) / (rc_f + 5.0);
        0.4 * usage_score + 0.3 * active_score + 0.3 * bayesian
    };
    json!({
        "id": plugin.id,
        "name": plugin.name,
        "description": plugin.description,
        "version": version,
        "category": category,
        "enabled": enabled,
        "roles": roles,
        "visibility_rules": plugin_rules,
        "skill_count": plugin.skills.len(),
        "agent_count": plugin.agents.len(),
        "mcp_server_count": plugin.mcp_servers.len(),
        "hook_count": plugin.hooks.len(),
        "total_events": total_events,
        "unique_users": unique_users,
        "active_users_7d": active_7d,
        "active_users_30d": active_30d,
        "avg_rating": avg_rating,
        "avg_rating_display": format!("{avg_rating:.1}"),
        "rating_count": rating_count,
        "rank_score": rank_score,
        "rank_score_display": format!("{rank_score:.2}"),
    })
}

pub(crate) async fn marketplace_versions_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let all_versions = repositories::marketplace_versions::list_all_versions_summary(&pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list all versions summary");
            vec![]
        });

    let mut seen_users = std::collections::HashSet::new();
    let mut users: Vec<serde_json::Value> = Vec::new();
    for v in &all_versions {
        if seen_users.insert(v.user_id.clone()) {
            users.push(json!({
                "user_id": v.user_id,
                "display_name": v.display_name,
                "email": v.email,
            }));
        }
    }

    let mut group_order: Vec<String> = Vec::new();
    let mut groups: std::collections::HashMap<String, Vec<serde_json::Value>> =
        std::collections::HashMap::new();
    let mut group_meta: std::collections::HashMap<String, (&str, Option<&str>, Option<&str>)> =
        std::collections::HashMap::new();

    for v in &all_versions {
        if !groups.contains_key(&v.user_id) {
            group_order.push(v.user_id.clone());
            group_meta.insert(
                v.user_id.clone(),
                (&v.user_id, v.display_name.as_deref(), v.email.as_deref()),
            );
        }
        let skill_names: Vec<serde_json::Value> = match &v.skill_names {
            serde_json::Value::Array(arr) => arr.clone(),
            _ => vec![],
        };
        groups.entry(v.user_id.clone()).or_default().push(json!({
            "id": v.id,
            "user_id": v.user_id,
            "version_number": v.version_number,
            "version_type": v.version_type,
            "skills_count": v.skills_count,
            "skill_names": skill_names,
            "created_at": v.created_at,
        }));
    }

    let user_groups: Vec<serde_json::Value> = group_order
        .iter()
        .filter_map(|uid| {
            let versions = groups.get(uid)?;
            let &(user_id, display_name, email) = group_meta.get(uid)?;
            let label = display_name.or(email).unwrap_or(user_id);
            let latest = versions.first()?;
            Some(json!({
                "label": label,
                "user_id": user_id,
                "version_count": versions.len(),
                "latest_version_number": latest["version_number"],
                "latest_created_at": latest["created_at"],
                "versions": versions,
            }))
        })
        .collect();

    let data = json!({
        "page": "marketplace-versions",
        "title": "Marketplace Versions",
        "users": users,
        "user_groups": user_groups,
    });
    super::render_page(&engine, "marketplace-versions", &data, &user_ctx, &mkt_ctx)
}
