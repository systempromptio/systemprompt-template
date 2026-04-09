use std::collections::HashSet;
use std::sync::Arc;

use systemprompt::identifiers::AgentId;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{AgentDetail, MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

fn build_agent_updated_at(
    agents_dir: &std::path::Path,
) -> std::collections::HashMap<String, String> {
    let mut agent_updated_at: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    if !agents_dir.exists() {
        return agent_updated_at;
    }
    let Ok(entries) = std::fs::read_dir(agents_dir) else {
        return agent_updated_at;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(metadata) = path.metadata() else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        let datetime: chrono::DateTime<chrono::Utc> = modified.into();
        let iso = datetime.to_rfc3339();
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(config) = serde_yaml::from_str::<serde_yaml::Value>(&content) else {
            continue;
        };
        if let Some(agents_map) = config.get("agents").and_then(|a| a.as_mapping()) {
            for key in agents_map.keys() {
                if let Some(aid) = key.as_str() {
                    agent_updated_at.insert(aid.to_string(), iso.clone());
                }
            }
        }
    }
    agent_updated_at
}

fn count_entities_via_plugins(
    entity_id: &str,
    entity_plugin_map: &std::collections::HashMap<String, Vec<(String, String)>>,
    target_plugin_map: &std::collections::HashMap<String, Vec<(String, String)>>,
) -> usize {
    if let Some(plugins) = entity_plugin_map.get(entity_id) {
        let mut ids: HashSet<String> = HashSet::new();
        for (plugin_id, _) in plugins {
            for (target_id, target_plugins) in target_plugin_map {
                if target_plugins.iter().any(|(pid, _)| pid == plugin_id) {
                    ids.insert(target_id.clone());
                }
            }
        }
        ids.len()
    } else {
        0
    }
}

fn build_agent_json(
    agent: &AgentDetail,
    agent_plugin_map: &std::collections::HashMap<String, Vec<(String, String)>>,
    skill_plugin_map: &std::collections::HashMap<String, Vec<(String, String)>>,
    mcp_plugin_map: &std::collections::HashMap<String, Vec<(String, String)>>,
    usage_counts: &std::collections::HashMap<String, i64>,
    agent_updated_at: &std::collections::HashMap<String, String>,
    filter_plugins: &mut HashSet<String>,
) -> serde_json::Value {
    let assigned_plugins: Vec<serde_json::Value> = agent_plugin_map
        .get(&agent.id)
        .map(|plugins| {
            plugins
                .iter()
                .map(|(pid, pname)| json!({"id": pid, "name": pname}))
                .collect()
        })
        .unwrap_or_default();

    let skill_count = count_entities_via_plugins(&agent.id, agent_plugin_map, skill_plugin_map);
    let mcp_count = count_entities_via_plugins(&agent.id, agent_plugin_map, mcp_plugin_map);

    let prompt_preview = if agent.system_prompt.len() > 300 {
        format!("{}...", &agent.system_prompt[..300])
    } else {
        agent.system_prompt.clone()
    };

    for p in &assigned_plugins {
        if let Some(name) = p.get("name").and_then(|v| v.as_str()) {
            filter_plugins.insert(name.to_string());
        }
    }

    let usage_count = usage_counts.get(&agent.id).copied().unwrap_or(0);
    let updated_at = agent_updated_at.get(&agent.id).cloned().unwrap_or_default();

    json!({
        "id": agent.id,
        "name": agent.name,
        "description": agent.description,
        "is_primary": agent.is_primary,
        "system_prompt": agent.system_prompt,
        "system_prompt_preview": prompt_preview,
        "port": agent.port,
        "endpoint": agent.endpoint,
        "assigned_plugins": assigned_plugins,
        "assigned_plugin_ids": assigned_plugins.iter().filter_map(|p| p.get("id").and_then(|v| v.as_str())).collect::<Vec<_>>(),
        "plugin_count": assigned_plugins.len(),
        "skill_count": skill_count,
        "mcp_count": mcp_count,
        "updated_at": updated_at,
        "usage_count": usage_count,
    })
}

pub(crate) async fn agents_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let all_agents = repositories::list_agents(&services_path).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list agents");
        vec![]
    });

    let agents = if user_ctx.is_admin {
        all_agents
    } else {
        let plugins = repositories::list_plugins_for_roles(&services_path, &user_ctx.roles)
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Failed to list plugins for roles");
                vec![]
            });
        let visible_agent_ids: HashSet<String> = plugins
            .iter()
            .flat_map(|p| p.agents.iter().map(|a| a.id.clone()))
            .collect();
        all_agents
            .into_iter()
            .filter(|a| visible_agent_ids.contains(&a.id))
            .collect()
    };

    let (skill_plugin_map, agent_plugin_map, mcp_plugin_map) =
        repositories::build_entity_plugin_maps(&services_path);

    let all_plugins = repositories::list_plugins_for_roles(&services_path, &["admin".to_string()])
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list all plugins");
            vec![]
        });

    let plugin_list: Vec<serde_json::Value> = all_plugins
        .iter()
        .map(|p| json!({"id": p.id, "name": p.name}))
        .collect();

    let agent_ids: Vec<AgentId> = agents.iter().map(|a| AgentId::new(&a.id)).collect();
    let usage_counts = repositories::fetch_agent_usage_counts(&pool, &agent_ids).await;

    let agent_updated_at = build_agent_updated_at(&services_path.join("agents"));

    let mut filter_plugins: HashSet<String> = HashSet::new();

    let agents_data: Vec<serde_json::Value> = agents
        .iter()
        .map(|agent| {
            build_agent_json(
                agent,
                &agent_plugin_map,
                &skill_plugin_map,
                &mcp_plugin_map,
                &usage_counts,
                &agent_updated_at,
                &mut filter_plugins,
            )
        })
        .collect();

    let mut sorted_plugins: Vec<String> = filter_plugins.into_iter().collect();
    sorted_plugins.sort();

    let data = json!({
        "page": "agents",
        "title": "Org Agents",
        "agents": agents_data,
        "all_plugins": plugin_list,
        "filter_plugins": sorted_plugins,
    });
    super::render_page(&engine, "agents", &data, &user_ctx, &mkt_ctx)
}

pub(crate) async fn agent_edit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let agent_id = params.get("id");
    let is_edit = agent_id.is_some();
    let agent = if let Some(id) = agent_id {
        let services_path = match super::get_services_path() {
            Ok(p) => p,
            Err(r) => return *r,
        };
        repositories::find_agent(&services_path, id)
            .map_err(|e| {
                tracing::warn!(error = %e, agent_id = %id, "Failed to fetch agent");
            })
            .ok()
            .flatten()
    } else {
        None
    };

    let data = json!({
        "page": "agent-edit",
        "title": if is_edit { "Edit Agent" } else { "Create Agent" },
        "is_edit": is_edit,
        "agent": agent,
    });
    super::render_page(&engine, "agent-edit", &data, &user_ctx, &mkt_ctx)
}
