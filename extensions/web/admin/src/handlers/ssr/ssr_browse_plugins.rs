use std::collections::HashSet;
use std::sync::Arc;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    response::Response,
};
use sqlx::PgPool;

use super::types::{BrowsePluginStats, BrowsePluginView, BrowsePluginsPageData};

pub async fn browse_plugins_page(
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
        repositories::org_marketplaces::resolve_authorized_marketplace_groups(&pool,),
        repositories::list_user_plugins_enriched(&pool, &user_ctx.user_id),
    );

    let marketplace_groups = marketplace_groups.unwrap_or_else(|_| vec![]);
    let user_plugins = user_plugins_result.unwrap_or_else(|_| vec![]);

    let added_base_ids: HashSet<String> = user_plugins
        .iter()
        .filter_map(|ep| ep.plugin.base_plugin_id.clone())
        .collect();

    let plugin_ids = if marketplace_groups.is_empty() {
        discover_included_plugins(&services_path)
    } else {
        marketplace_groups
            .iter()
            .flat_map(|(_, ids)| ids.clone())
            .collect()
    };

    let (mut plugins, categories_set, already_added_count) =
        collect_browse_plugins(&plugin_ids, &services_path, &added_base_ids);

    plugins.sort_unstable_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    let total_available = plugins.len();
    let mut categories: Vec<String> = categories_set.into_iter().collect();
    categories.sort();

    let data = BrowsePluginsPageData {
        page: "browse-plugins",
        title: "Browse Plugins",
        plugins,
        categories,
        stats: BrowsePluginStats {
            total_available,
            already_added: already_added_count,
        },
    };

    let value = serde_json::to_value(&data).unwrap_or_else(|_| serde_json::Value::Null);
    super::render_page(&engine, "browse-plugins", &value, &user_ctx, &mkt_ctx)
}

#[derive(serde::Deserialize)]
struct PluginsConfig {
    #[serde(default)]
    includes: Vec<String>,
}

fn discover_included_plugins(services_path: &std::path::Path) -> Vec<String> {
    let config_path = services_path.join("plugins").join("config.yaml");
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(path = %config_path.display(), error = %e, "Plugin config not found");
            return vec![];
        }
    };
    let cfg: PluginsConfig = match serde_yaml::from_str(&content) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(path = %config_path.display(), error = %e, "Invalid plugin config YAML");
            return vec![];
        }
    };
    cfg.includes
        .iter()
        .filter_map(|s| s.split('/').next())
        .map(String::from)
        .collect()
}

fn collect_browse_plugins(
    plugin_ids: &[String],
    services_path: &std::path::Path,
    added_base_ids: &HashSet<String>,
) -> (Vec<BrowsePluginView>, HashSet<String>, usize) {
    let mut categories_set: HashSet<String> = HashSet::new();
    let mut plugins: Vec<BrowsePluginView> = Vec::new();
    let mut already_added_count = 0usize;

    for pid in plugin_ids {
        let Ok(Some(detail)) = repositories::find_plugin_detail(services_path, pid) else {
            continue;
        };
        if !detail.category.is_empty() {
            categories_set.insert(detail.category.clone());
        }
        let already_added = added_base_ids.contains(pid);
        if already_added {
            already_added_count += 1;
        }
        plugins.push(BrowsePluginView {
            plugin_id: pid.clone(),
            name: detail.name,
            description: detail.description,
            category: detail.category,
            version: detail.version,
            skill_count: detail.skills.len(),
            agent_count: detail.agents.len(),
            mcp_count: detail.mcp_servers.len(),
            already_added,
        });
    }

    (plugins, categories_set, already_added_count)
}
