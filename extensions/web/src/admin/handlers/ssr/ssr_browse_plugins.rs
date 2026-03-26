use std::collections::HashSet;
use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    response::Response,
};
use sqlx::PgPool;

use super::types::{BrowsePluginStats, BrowsePluginView, BrowsePluginsPageData};

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
        repositories::org_marketplaces::resolve_authorized_marketplace_groups(&pool,),
        repositories::list_user_plugins_enriched(&pool, &user_ctx.user_id),
    );

    let marketplace_groups = marketplace_groups.unwrap_or_default();
    let user_plugins = user_plugins_result.unwrap_or_default();

    let added_base_ids: HashSet<String> = user_plugins
        .iter()
        .filter_map(|ep| ep.plugin.base_plugin_id.clone())
        .collect();

    let (mut plugins, categories_set, already_added_count) =
        collect_browse_plugins(&marketplace_groups, &services_path, &added_base_ids);

    plugins.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

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

    let value = serde_json::to_value(&data).unwrap_or_default();
    super::render_page(&engine, "browse-plugins", &value, &user_ctx, &mkt_ctx)
}

fn collect_browse_plugins(
    marketplace_groups: &[(
        crate::admin::types::marketplaces::OrgMarketplace,
        Vec<String>,
    )],
    services_path: &std::path::Path,
    added_base_ids: &HashSet<String>,
) -> (Vec<BrowsePluginView>, HashSet<String>, usize) {
    let mut categories_set: HashSet<String> = HashSet::new();
    let mut plugins: Vec<BrowsePluginView> = Vec::new();
    let mut already_added_count = 0usize;

    for (_mkt, plugin_ids) in marketplace_groups {
        for pid in plugin_ids {
            if pid == "systemprompt" {
                continue;
            }
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
    }

    (plugins, categories_set, already_added_count)
}
