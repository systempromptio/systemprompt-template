use std::collections::HashSet;
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

#[allow(clippy::too_many_lines)]
pub(crate) async fn my_marketplace_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let org_plugin_ids = repositories::org_marketplaces::resolve_authorized_org_plugin_ids(
        &pool,
        &user_ctx.roles,
        &user_ctx.department,
        user_ctx.is_admin,
    )
    .await
    .unwrap_or_default();

    let user_plugins = repositories::list_user_plugins_enriched(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_default();

    let forked_base_ids: HashSet<String> = user_plugins
        .iter()
        .filter_map(|ep| ep.plugin.base_plugin_id.clone())
        .collect();

    let mut plugins_json: Vec<serde_json::Value> = Vec::new();
    let mut categories_set: HashSet<String> = HashSet::new();
    let mut inherited_count = 0usize;
    let mut customized_count = 0usize;
    let mut custom_count = 0usize;

    for pid in &org_plugin_ids {
        if forked_base_ids.contains(pid) {
            continue; // Will be shown as "customized" from user plugins
        }

        let detail = repositories::get_plugin_detail(&services_path, pid)
            .ok()
            .flatten();

        let (name, description, category, version, skill_count, agent_count, mcp_count, hook_count) =
            if let Some(ref d) = detail {
                (
                    d.name.as_str(),
                    d.description.as_str(),
                    d.category.as_str(),
                    d.version.as_str(),
                    d.skills.len(),
                    d.agents.len(),
                    d.mcp_servers.len(),
                    0usize,
                )
            } else {
                (pid.as_str(), "", "", "", 0, 0, 0, 0)
            };

        if !category.is_empty() {
            categories_set.insert(category.to_string());
        }

        inherited_count += 1;

        let skills_arr: Vec<serde_json::Value> = if let Some(ref d) = detail {
            d.skills
                .iter()
                .map(|s| json!({"id": s, "name": s}))
                .collect()
        } else {
            vec![]
        };
        let agents_arr: Vec<serde_json::Value> = if let Some(ref d) = detail {
            d.agents
                .iter()
                .map(|a| json!({"id": a, "name": a}))
                .collect()
        } else {
            vec![]
        };
        let mcp_arr: Vec<serde_json::Value> = if let Some(ref d) = detail {
            d.mcp_servers
                .iter()
                .map(|m| json!({"id": m, "name": m}))
                .collect()
        } else {
            vec![]
        };

        plugins_json.push(json!({
            "plugin_id": pid,
            "name": name,
            "description": description,
            "category": category,
            "version": version,
            "source": "inherited",
            "skill_count": skill_count,
            "agent_count": agent_count,
            "mcp_count": mcp_count,
            "hook_count": hook_count,
            "skills": skills_arr,
            "agents": agents_arr,
            "mcp_servers": mcp_arr,
            "hooks": [],
        }));
    }

    for ep in &user_plugins {
        let p = &ep.plugin;
        let source = if p.base_plugin_id.is_some() {
            customized_count += 1;
            "customized"
        } else {
            custom_count += 1;
            "custom"
        };

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

        plugins_json.push(json!({
            "plugin_id": p.plugin_id,
            "name": p.name,
            "description": p.description,
            "category": p.category,
            "version": p.version,
            "source": source,
            "base_plugin_id": p.base_plugin_id,
            "skill_count": ep.skill_count,
            "agent_count": ep.agent_count,
            "mcp_count": ep.mcp_count,
            "hook_count": ep.hook_count,
            "skills": skills_json,
            "agents": agents_json,
            "mcp_servers": mcp_json,
            "hooks": hooks_json,
        }));
    }

    plugins_json.sort_by(|a, b| {
        let a_name = a["name"].as_str().unwrap_or("");
        let b_name = b["name"].as_str().unwrap_or("");
        a_name.to_lowercase().cmp(&b_name.to_lowercase())
    });

    let total_count = plugins_json.len();
    let mut categories: Vec<String> = categories_set.into_iter().collect();
    categories.sort();

    let data = json!({
        "page": "my-marketplace",
        "title": "My Marketplace",
        "plugins": plugins_json,
        "categories": categories,
        "stats": {
            "total_count": total_count,
            "inherited_count": inherited_count,
            "customized_count": customized_count,
            "custom_count": custom_count,
        },
    });

    super::render_page(&engine, "my-marketplace", &data, &user_ctx, &mkt_ctx)
}
