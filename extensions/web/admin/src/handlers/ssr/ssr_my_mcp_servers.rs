use std::collections::HashMap;
use std::sync::Arc;

use crate::repositories;
use crate::repositories::conversation_analytics;
use crate::templates::AdminTemplateEngine;
use crate::types::conversation_analytics::EntityEffectiveness;
use crate::types::{MarketplaceContext, UserContext, ENTITY_MCP_TOOL};
use axum::{
    extract::{Extension, State},
    response::Response,
};
use sqlx::PgPool;

use super::types::{McpServerStats, McpServerView, MyMcpServersPageData, NamedEntity};

pub async fn my_mcp_servers_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let (mcp_servers, user_plugins, usage_data, eff_data) =
        fetch_mcp_data(&pool, &user_ctx.user_id).await;

    let usage_map: HashMap<&str, _> = usage_data
        .iter()
        .filter(|u| u.entity_type == ENTITY_MCP_TOOL)
        .map(|u| (u.entity_name.as_str(), u))
        .collect();

    let eff_map: HashMap<&str, &EntityEffectiveness> = eff_data
        .iter()
        .map(|e| (e.entity_name.as_str(), e))
        .collect();

    let mcp_plugin_map = build_mcp_plugin_map(&user_plugins);
    let servers = build_server_views(&mcp_servers, &usage_map, &eff_map, &mcp_plugin_map);

    let data = MyMcpServersPageData {
        page: "my-mcp-servers",
        title: "My MCP Servers",
        stats: McpServerStats {
            total_count: mcp_servers.len(),
            enabled_count: mcp_servers.iter().filter(|s| s.enabled).count(),
        },
        servers,
    };

    let value =
        serde_json::to_value(&data).unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    super::render_page(&engine, "my-mcp-servers", &value, &user_ctx, &mkt_ctx)
}

async fn fetch_mcp_data(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
) -> (
    Vec<crate::types::UserMcpServer>,
    Vec<repositories::user_plugins::UserPluginEnriched>,
    Vec<crate::types::conversation_analytics::EntityUsageSummary>,
    Vec<EntityEffectiveness>,
) {
    let (mcp_servers_res, user_plugins_res, usage_res, eff_res, last_used_res) = tokio::join!(
        repositories::user_mcp_servers::list_user_mcp_servers(pool, user_id),
        repositories::list_user_plugins_enriched(pool, user_id),
        conversation_analytics::fetch_entity_usage_summary(pool, user_id),
        conversation_analytics::fetch_entity_effectiveness(pool, user_id, "mcp_tool"),
        conversation_analytics::fetch_entity_last_used(pool, user_id),
    );

    let mcp_servers = mcp_servers_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list user MCP servers");
        vec![]
    });
    let user_plugins = user_plugins_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list enriched user plugins");
        vec![]
    });
    let usage_data = usage_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch entity usage summary");
        vec![]
    });
    let eff_data = eff_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch MCP tool effectiveness");
        vec![]
    });
    let _last_used_data = last_used_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch entity last used");
        vec![]
    });

    (mcp_servers, user_plugins, usage_data, eff_data)
}

fn build_mcp_plugin_map(
    user_plugins: &[repositories::user_plugins::UserPluginEnriched],
) -> HashMap<String, Vec<NamedEntity>> {
    let mut map: HashMap<String, Vec<NamedEntity>> = HashMap::new();
    for ep in user_plugins {
        for m in &ep.mcp_servers {
            map.entry(m.id.clone()).or_default().push(NamedEntity {
                id: ep.plugin.plugin_id.clone(),
                name: ep.plugin.name.clone(),
            });
        }
    }
    map
}

fn build_server_views(
    mcp_servers: &[crate::types::UserMcpServer],
    usage_map: &HashMap<&str, &crate::types::conversation_analytics::EntityUsageSummary>,
    eff_map: &HashMap<&str, &EntityEffectiveness>,
    mcp_plugin_map: &HashMap<String, Vec<NamedEntity>>,
) -> Vec<McpServerView> {
    mcp_servers
        .iter()
        .map(|s| {
            let key = s.mcp_server_id.as_str();
            let usage = usage_map.get(key);
            let eff = eff_map.get(key);
            McpServerView {
                id: s.id.clone(),
                mcp_server_id: s.mcp_server_id.to_string(),
                name: s.name.clone(),
                description: s.description.clone(),
                endpoint: s.endpoint.clone(),
                enabled: s.enabled,
                plugin_names: mcp_plugin_map.get(&s.id).cloned().unwrap_or_else(Vec::new),
                base_mcp_server_id: s.base_mcp_server_id.as_ref().map(ToString::to_string),
                is_system: key == "systemprompt",
                total_uses: usage.map_or(0, |u| u.total_uses),
                session_count: usage.map_or(0, |u| u.session_count),
                avg_effectiveness: eff.map_or_else(
                    || "0.0".to_string(),
                    |e| format!("{:.1}", e.avg_effectiveness),
                ),
                scored_sessions: eff.map_or(0, |e| e.scored_sessions),
                goal_achievement_pct: eff.map_or_else(
                    || "0.0".to_string(),
                    |e| format!("{:.0}", e.goal_achievement_pct),
                ),
            }
        })
        .collect()
}
