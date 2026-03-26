use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

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

    let enriched = repositories::list_user_plugins_enriched(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_default();

    let plugin_data = enriched.iter().find(|ep| ep.plugin.plugin_id == *plugin_id);

    let Some(plugin_data) = plugin_data else {
        return axum::response::Redirect::to("/admin/my/marketplace").into_response();
    };

    let p = &plugin_data.plugin;

    let skills_json: Vec<serde_json::Value> = plugin_data
        .skills
        .iter()
        .map(|s| json!({ "id": s.id, "name": s.name }))
        .collect();
    let agents_json: Vec<serde_json::Value> = plugin_data
        .agents
        .iter()
        .map(|a| json!({ "id": a.id, "name": a.name }))
        .collect();
    let mcp_json: Vec<serde_json::Value> = plugin_data
        .mcp_servers
        .iter()
        .map(|m| json!({ "id": m.id, "name": m.name }))
        .collect();

    let plugin_json = json!({
        "plugin_id": p.plugin_id,
        "name": p.name,
        "description": p.description,
        "category": p.category,
        "version": p.version,
        "base_plugin_id": p.base_plugin_id,
        "author_name": p.author_name,
        "skill_count": plugin_data.skill_count,
        "agent_count": plugin_data.agent_count,
        "mcp_count": plugin_data.mcp_count,
        "skills": skills_json,
        "agents": agents_json,
        "mcp_servers": mcp_json,
    });

    let data = json!({
        "page": "my-plugin-view",
        "title": p.name,
        "plugin": plugin_json,
    });

    super::render_page(&engine, "my-plugin-view", &data, &user_ctx, &mkt_ctx)
}
