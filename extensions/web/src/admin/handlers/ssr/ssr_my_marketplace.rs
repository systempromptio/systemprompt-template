use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    response::Response,
};
use sqlx::PgPool;

use super::types::{
    MarketplacePluginView, MarketplaceStats, MyMarketplacePageData, NamedEntity,
    PlatformMarketplacePlugin,
};

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

    let user_plugins = repositories::list_user_plugins_enriched(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_default();

    let platform_plugin = build_platform_plugin(&services_path);
    let platform_hook_count = platform_plugin.as_ref().map_or(0, |p| p.hook_count);
    let (plugins, total_skills, total_agents, total_mcp) =
        collect_marketplace_plugins(&user_plugins);
    let total_hooks = platform_hook_count + plugins.len() * DEFAULT_HOOK_COUNT;

    let data = MyMarketplacePageData {
        page: "my-marketplace",
        title: "My Marketplace",
        platform_plugin: serde_json::to_value(&platform_plugin).unwrap_or_default(),
        has_plugins: !plugins.is_empty(),
        stats: MarketplaceStats {
            plugin_count: plugins.len(),
            total_skills,
            total_agents,
            total_mcp,
            total_hooks,
        },
        plugins,
    };
    let data_value = serde_json::to_value(&data).unwrap_or_default();
    super::render_page(&engine, "my-marketplace", &data_value, &user_ctx, &mkt_ctx)
}

fn collect_marketplace_plugins(
    user_plugins: &[crate::admin::repositories::user_plugins::UserPluginEnriched],
) -> (Vec<MarketplacePluginView>, usize, usize, usize) {
    let mut total_skills = 0usize;
    let mut total_agents = 0usize;
    let mut total_mcp = 0usize;
    let mut plugins: Vec<MarketplacePluginView> = Vec::new();

    for ep in user_plugins {
        let p = &ep.plugin;
        if p.enabled {
            total_skills += ep.skill_count;
            total_agents += ep.agent_count;
            total_mcp += ep.mcp_count;
        }
        let skills: Vec<NamedEntity> = ep.skills.iter().map(NamedEntity::from).collect();
        let agents: Vec<NamedEntity> = ep.agents.iter().map(NamedEntity::from).collect();
        let mcp_servers: Vec<NamedEntity> = ep.mcp_servers.iter().map(NamedEntity::from).collect();
        plugins.push(MarketplacePluginView {
            plugin_id: p.plugin_id.clone(),
            name: p.name.clone(),
            description: p.description.clone(),
            category: p.category.clone(),
            base_plugin_id: p.base_plugin_id.clone(),
            enabled: p.enabled,
            skill_count: ep.skill_count,
            agent_count: ep.agent_count,
            mcp_count: ep.mcp_count,
            hook_count: DEFAULT_HOOK_COUNT,
            skills,
            agents,
            mcp_servers,
        });
    }

    (plugins, total_skills, total_agents, total_mcp)
}

const DEFAULT_HOOK_COUNT: usize = 14;

fn build_platform_plugin(services_path: &std::path::Path) -> Option<PlatformMarketplacePlugin> {
    let detail = match repositories::find_plugin_detail(services_path, "systemprompt") {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!(error = ?e, "Failed to load systemprompt plugin detail");
            None
        }
    };

    let d = detail?;
    let skills: Vec<NamedEntity> = d
        .skills
        .iter()
        .map(|s| NamedEntity::from_str_pair(s))
        .collect();
    let agents: Vec<NamedEntity> = d
        .agents
        .iter()
        .map(|a| NamedEntity::from_str_pair(a))
        .collect();
    let mcp_servers: Vec<NamedEntity> = d
        .mcp_servers
        .iter()
        .map(|m| NamedEntity::from_str_pair(m))
        .collect();

    Some(PlatformMarketplacePlugin {
        plugin_id: "systemprompt".to_string(),
        name: d.name,
        description: d.description,
        category: d.category,
        version: d.version,
        skill_count: skills.len(),
        agent_count: agents.len(),
        mcp_count: mcp_servers.len(),
        hook_count: DEFAULT_HOOK_COUNT,
        skills,
        agents,
        mcp_servers,
    })
}
