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

pub(crate) async fn browse_plugins_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let (marketplace_groups, user_plugins_result) = tokio::join!(
        repositories::org_marketplaces::resolve_authorized_marketplace_groups(
            &pool,
            &user_ctx.roles,
            &user_ctx.department,
            user_ctx.is_admin,
        ),
        repositories::list_user_plugins_enriched(&pool, &user_ctx.user_id),
    );

    let marketplace_groups = marketplace_groups.unwrap_or_default();
    let user_plugins = user_plugins_result.unwrap_or_default();

    let added_base_ids: HashSet<String> = user_plugins
        .iter()
        .filter_map(|ep| ep.plugin.base_plugin_id.clone())
        .collect();

    let mut categories_set: HashSet<String> = HashSet::new();
    let mut plugins_json: Vec<serde_json::Value> = Vec::new();
    let mut already_added_count = 0usize;

    for (_mkt, plugin_ids) in &marketplace_groups {
        for pid in plugin_ids {
            if pid == "systemprompt" {
                continue;
            }

            let Ok(Some(detail)) = repositories::get_plugin_detail(&services_path, pid) else {
                continue;
            };

            let category = &detail.category;
            if !category.is_empty() {
                categories_set.insert(category.clone());
            }

            let already_added = added_base_ids.contains(pid);
            if already_added {
                already_added_count += 1;
            }

            plugins_json.push(json!({
                "plugin_id": pid,
                "name": detail.name,
                "description": detail.description,
                "category": detail.category,
                "version": detail.version,
                "skill_count": detail.skills.len(),
                "agent_count": detail.agents.len(),
                "mcp_count": detail.mcp_servers.len(),
                "already_added": already_added,
            }));
        }
    }

    plugins_json.sort_by(|a, b| {
        let a_name = a["name"].as_str().unwrap_or("");
        let b_name = b["name"].as_str().unwrap_or("");
        a_name.to_lowercase().cmp(&b_name.to_lowercase())
    });

    let total_available = plugins_json.len();
    let mut categories: Vec<String> = categories_set.into_iter().collect();
    categories.sort();

    let data = json!({
        "page": "browse-plugins",
        "title": "Browse Plugins",
        "plugins": plugins_json,
        "categories": categories,
        "stats": {
            "total_available": total_available,
            "already_added": already_added_count,
        },
    });

    super::render_page(&engine, "browse-plugins", &data, &user_ctx, &mkt_ctx)
}
