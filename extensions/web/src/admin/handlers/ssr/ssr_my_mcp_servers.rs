use std::collections::HashMap;
use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::repositories::conversation_analytics;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::conversation_analytics::EntityEffectiveness;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    response::Response,
};
use sqlx::PgPool;

use super::types::{McpServerStats, McpServerView, MyMcpServersPageData, NamedEntity};

pub(crate) async fn my_mcp_servers_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let (mcp_servers_res, user_plugins_res, usage_res, eff_res, last_used_res) = tokio::join!(
        repositories::user_mcp_servers::list_user_mcp_servers(&pool, &user_ctx.user_id),
        repositories::list_user_plugins_enriched(&pool, &user_ctx.user_id),
        conversation_analytics::fetch_entity_usage_summary(&pool, &user_ctx.user_id),
        conversation_analytics::fetch_entity_effectiveness(&pool, &user_ctx.user_id, "mcp_tool"),
        conversation_analytics::fetch_entity_last_used(&pool, &user_ctx.user_id),
    );

    let mcp_servers = mcp_servers_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list user MCP servers");
        vec![]
    });

    let user_plugins = user_plugins_res.unwrap_or_else(|_| vec![]);

    let usage_data = usage_res.unwrap_or_else(|_| vec![]);
    let eff_data = eff_res.unwrap_or_else(|_| vec![]);
    let _last_used_data = last_used_res.unwrap_or_else(|_| vec![]);

    let usage_map: HashMap<&str, _> = usage_data
        .iter()
        .filter(|u| u.entity_type == "mcp_tool")
        .map(|u| (u.entity_name.as_str(), u))
        .collect();

    let eff_map: HashMap<&str, &EntityEffectiveness> = eff_data
        .iter()
        .map(|e| (e.entity_name.as_str(), e))
        .collect();

    let mut mcp_plugin_map: HashMap<String, Vec<NamedEntity>> = HashMap::new();
    for ep in &user_plugins {
        for m in &ep.mcp_servers {
            mcp_plugin_map
                .entry(m.id.clone())
                .or_default()
                .push(NamedEntity {
                    id: ep.plugin.plugin_id.clone(),
                    name: ep.plugin.name.clone(),
                });
        }
    }

    let total_count = mcp_servers.len();
    let enabled_count = mcp_servers.iter().filter(|s| s.enabled).count();

    let servers: Vec<McpServerView> = mcp_servers
        .iter()
        .map(|s| {
            let is_system = s.mcp_server_id.as_str() == "systemprompt";
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
                base_mcp_server_id: s
                    .base_mcp_server_id
                    .as_ref()
                    .map(std::string::ToString::to_string),
                is_system,
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
        .collect();

    let data = MyMcpServersPageData {
        page: "my-mcp-servers",
        title: "My MCP Servers",
        servers,
        stats: McpServerStats {
            total_count,
            enabled_count,
        },
    };

    let value =
        serde_json::to_value(&data).unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    super::render_page(&engine, "my-mcp-servers", &value, &user_ctx, &mkt_ctx)
}
