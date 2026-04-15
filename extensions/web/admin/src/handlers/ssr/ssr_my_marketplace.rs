use std::sync::Arc;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    response::Response,
};
use sqlx::PgPool;

use super::types::{MarketplacePluginView, MarketplaceStats, MyMarketplacePageData, NamedEntity};

const DEFAULT_HOOK_COUNT: usize = 14;

pub async fn my_marketplace_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let services_path = super::get_services_path()
        .unwrap_or_else(|_| std::path::PathBuf::from("services"));
    let user_plugins = repositories::list_effective_enriched_plugins(
        &pool,
        &user_ctx.user_id,
        &services_path,
    )
    .await;

    let (plugins, total_skills, total_agents, total_mcp) =
        collect_marketplace_plugins(&user_plugins);
    let total_hooks = plugins.len() * DEFAULT_HOOK_COUNT;

    let data = MyMarketplacePageData {
        page: "my-marketplace",
        title: "My Marketplace",
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
    let data_value = serde_json::to_value(&data).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to serialize marketplace page data");
        serde_json::Value::Null
    });
    super::render_page(&engine, "my-marketplace", &data_value, &user_ctx, &mkt_ctx)
}

fn collect_marketplace_plugins(
    user_plugins: &[repositories::user_plugins::UserPluginEnriched],
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
