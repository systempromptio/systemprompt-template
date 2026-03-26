use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

pub(crate) async fn my_plugins_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let enriched = repositories::list_user_plugins_enriched(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list enriched user plugins");
            vec![]
        });

    let mut categories_set = std::collections::HashSet::new();
    let mut enabled_count = 0usize;

    let plugins_json: Vec<serde_json::Value> = enriched
        .iter()
        .map(|ep| {
            let p = &ep.plugin;
            if p.enabled {
                enabled_count += 1;
            }
            if !p.category.is_empty() {
                categories_set.insert(p.category.clone());
            }

            let skills_json: Vec<serde_json::Value> = ep
                .skills
                .iter()
                .map(|s| json!({ "id": s.id, "name": s.name }))
                .collect();
            let agents_json: Vec<serde_json::Value> = ep
                .agents
                .iter()
                .map(|a| json!({ "id": a.id, "name": a.name }))
                .collect();
            let mcp_json: Vec<serde_json::Value> = ep
                .mcp_servers
                .iter()
                .map(|m| json!({ "id": m.id, "name": m.name }))
                .collect();
            let hooks_json: Vec<serde_json::Value> = ep
                .hooks
                .iter()
                .map(|h| {
                    json!({
                        "id": h.id,
                        "name": h.name,
                        "event": h.event,
                        "matcher": h.matcher,
                        "is_async": h.is_async,
                    })
                })
                .collect();

            json!({
                "id": p.id,
                "plugin_id": p.plugin_id,
                "name": p.name,
                "description": p.description,
                "version": p.version,
                "enabled": p.enabled,
                "category": p.category,
                "keywords": p.keywords,
                "author_name": p.author_name,
                "base_plugin_id": p.base_plugin_id,
                "created_at": p.created_at,
                "updated_at": p.updated_at,
                "skill_count": ep.skill_count,
                "agent_count": ep.agent_count,
                "mcp_count": ep.mcp_count,
                "hook_count": ep.hook_count,
                "skills": skills_json,
                "agents": agents_json,
                "mcp_servers": mcp_json,
                "hooks": hooks_json,
            })
        })
        .collect();

    let plugin_count = enriched.len();
    let mut categories: Vec<String> = categories_set.into_iter().collect();
    categories.sort();

    let data = json!({
        "page": "my-plugins",
        "title": "My Plugins",
        "plugins": plugins_json,
        "categories": categories,
        "stats": {
            "plugin_count": plugin_count,
            "enabled_count": enabled_count,
        },
    });
    super::render_page(&engine, "my-plugins", &data, &user_ctx, &mkt_ctx)
}

pub(crate) async fn my_plugin_edit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let plugin_id = params.get("id");
    let is_edit = plugin_id.is_some();

    let plugin_with_assoc = if let Some(id) = plugin_id {
        repositories::get_plugin_with_associations(&pool, &user_ctx.user_id, id)
            .await
            .map_err(|e| {
                tracing::warn!(error = %e, plugin_id = %id, "Failed to fetch user plugin");
            })
            .ok()
            .flatten()
    } else {
        None
    };

    let user_skills = repositories::list_user_skills(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_default();
    let user_agents = repositories::list_user_agents(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_default();
    let user_mcp_servers = repositories::list_user_mcp_servers(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_default();

    let selected_skills: Vec<String> = plugin_with_assoc
        .as_ref()
        .map_or(vec![], |p| p.skill_ids.clone());
    let selected_agents: Vec<String> = plugin_with_assoc
        .as_ref()
        .map_or(vec![], |p| p.agent_ids.clone());
    let selected_mcp: Vec<String> = plugin_with_assoc
        .as_ref()
        .map_or(vec![], |p| p.mcp_server_ids.clone());

    let skills_list: Vec<serde_json::Value> = user_skills
        .iter()
        .map(|s| {
            json!({
                "value": s.id,
                "name": s.name,
                "checked": selected_skills.contains(&s.id),
            })
        })
        .collect();
    let agents_list: Vec<serde_json::Value> = user_agents
        .iter()
        .map(|a| {
            json!({
                "value": a.id,
                "name": a.name,
                "checked": selected_agents.contains(&a.id),
            })
        })
        .collect();
    let mcp_list: Vec<serde_json::Value> = user_mcp_servers
        .iter()
        .map(|m| {
            json!({
                "value": m.id,
                "name": m.name,
                "checked": selected_mcp.contains(&m.id),
            })
        })
        .collect();

    let plugin_json = plugin_with_assoc.as_ref().map(|p| {
        json!({
            "id": p.plugin.id,
            "plugin_id": p.plugin.plugin_id,
            "name": p.plugin.name,
            "description": p.plugin.description,
            "version": p.plugin.version,
            "enabled": p.plugin.enabled,
            "category": p.plugin.category,
            "keywords": p.plugin.keywords,
            "author_name": p.plugin.author_name,
            "base_plugin_id": p.plugin.base_plugin_id,
        })
    });

    let keywords_csv = plugin_with_assoc
        .as_ref()
        .map_or(String::new(), |p| p.plugin.keywords.join(", "));

    let data = json!({
        "page": "my-plugin-edit",
        "title": if is_edit { "Edit My Plugin" } else { "Create My Plugin" },
        "is_edit": is_edit,
        "plugin": plugin_json,
        "keywords_csv": keywords_csv,
        "skills_list": skills_list,
        "agents_list": agents_list,
        "mcp_list": mcp_list,
    });
    super::render_page(&engine, "my-plugin-edit", &data, &user_ctx, &mkt_ctx)
}
