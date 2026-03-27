use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::{IntoResponse, Response},
};
use sqlx::PgPool;

use super::types::{MyPluginViewPageData, NamedEntity, PluginDetailView};

const DEFAULT_HOOK_COUNT: usize = 14;

pub(crate) async fn my_plugin_view_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let Some(plugin_id) = params.get("id") else {
        return axum::response::Redirect::to("/admin/my/marketplace").into_response();
    };

    if plugin_id == "systemprompt" {
        return render_platform_plugin_view(&engine, &user_ctx, &mkt_ctx);
    }

    let enriched = repositories::list_user_plugins_enriched(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_else(|_| vec![]);

    let plugin_data = enriched.iter().find(|ep| ep.plugin.plugin_id == *plugin_id);

    let Some(plugin_data) = plugin_data else {
        return axum::response::Redirect::to("/admin/my/marketplace").into_response();
    };

    let p = &plugin_data.plugin;

    let plugin_view = PluginDetailView {
        plugin_id: p.plugin_id.clone(),
        name: p.name.clone(),
        description: p.description.clone(),
        category: p.category.clone(),
        version: p.version.clone(),
        base_plugin_id: p.base_plugin_id.clone(),
        author_name: p.author_name.clone(),
        skill_count: plugin_data.skill_count,
        agent_count: plugin_data.agent_count,
        mcp_count: plugin_data.mcp_count,
        hook_count: DEFAULT_HOOK_COUNT,
        skills: plugin_data.skills.iter().map(NamedEntity::from).collect(),
        agents: plugin_data.agents.iter().map(NamedEntity::from).collect(),
        mcp_servers: plugin_data
            .mcp_servers
            .iter()
            .map(NamedEntity::from)
            .collect(),
    };

    let data = MyPluginViewPageData {
        page: "my-plugin-view",
        title: p.name.clone(),
        plugin: plugin_view,
    };

    let value = serde_json::to_value(&data).unwrap_or_else(|_| serde_json::Value::Null);
    super::render_page(&engine, "my-plugin-view", &value, &user_ctx, &mkt_ctx)
}

fn render_platform_plugin_view(
    engine: &AdminTemplateEngine,
    user_ctx: &UserContext,
    mkt_ctx: &MarketplaceContext,
) -> Response {
    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let detail = repositories::find_plugin_detail(&services_path, "systemprompt")
        .map_err(|e| {
            tracing::warn!(error = ?e, "Failed to load systemprompt plugin detail");
        })
        .ok()
        .flatten();

    let Some(d) = detail else {
        return axum::response::Redirect::to("/admin/my/marketplace").into_response();
    };

    let plugin_view = PluginDetailView {
        plugin_id: "systemprompt".to_string(),
        name: d.name.clone(),
        description: d.description.clone(),
        category: d.category.clone(),
        version: d.version.clone(),
        base_plugin_id: Some("systemprompt".to_string()),
        author_name: d.author_name.clone(),
        skill_count: d.skills.len(),
        agent_count: d.agents.len(),
        mcp_count: d.mcp_servers.len(),
        hook_count: DEFAULT_HOOK_COUNT,
        skills: d
            .skills
            .iter()
            .map(|s| NamedEntity::from_str_pair(s))
            .collect(),
        agents: d
            .agents
            .iter()
            .map(|a| NamedEntity::from_str_pair(a))
            .collect(),
        mcp_servers: d
            .mcp_servers
            .iter()
            .map(|m| NamedEntity::from_str_pair(m))
            .collect(),
    };

    let data = MyPluginViewPageData {
        page: "my-plugin-view",
        title: d.name.clone(),
        plugin: plugin_view,
    };

    let value = serde_json::to_value(&data).unwrap_or_else(|_| serde_json::Value::Null);
    super::render_page(engine, "my-plugin-view", &value, user_ctx, mkt_ctx)
}
