use crate::admin::repositories;
use crate::admin::repositories::conversation_analytics;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::conversation_analytics::{
    EntityEffectiveness, EntityUsageSummary, SkillEffectiveness,
};
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::{IntoResponse, Redirect, Response},
};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;

use super::ssr_my_plugins_helpers::{
    build_association_lists, build_platform_plugin, build_plugin_edit_data, collect_my_plugins,
};
use super::types::{MyPluginEditPageData, MyPluginsPageData, PluginStats};

pub(crate) async fn my_plugins_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let (enriched, entity_usage, skill_eff, agent_eff) = tokio::join!(
        async {
            repositories::list_user_plugins_enriched(&pool, &user_ctx.user_id)
                .await
                .unwrap_or_default()
        },
        async {
            conversation_analytics::fetch_entity_usage_summary(&pool, &user_ctx.user_id)
                .await
                .unwrap_or_default()
        },
        async {
            conversation_analytics::fetch_skill_effectiveness(&pool, &user_ctx.user_id)
                .await
                .unwrap_or_default()
        },
        async {
            conversation_analytics::fetch_entity_effectiveness(&pool, &user_ctx.user_id, "agent")
                .await
                .unwrap_or_default()
        },
    );

    let skill_usage_map: HashMap<&str, &EntityUsageSummary> = entity_usage
        .iter()
        .filter(|e| e.entity_type == "skill")
        .map(|e| (e.entity_id.as_str(), e))
        .collect();
    let skill_eff_map: HashMap<&str, &SkillEffectiveness> =
        skill_eff.iter().map(|s| (s.skill_id.as_str(), s)).collect();
    let agent_eff_map: HashMap<&str, &EntityEffectiveness> = agent_eff
        .iter()
        .map(|a| (a.entity_name.as_str(), a))
        .collect();
    let platform_plugin = super::get_services_path()
        .ok()
        .and_then(|p| build_platform_plugin(&p));
    let (plugins_json, categories) = collect_my_plugins(
        &enriched,
        platform_plugin,
        &skill_usage_map,
        &skill_eff_map,
        &agent_eff_map,
    );
    let plugin_count = plugins_json.len();
    let data = MyPluginsPageData {
        page: "my-plugins",
        title: "My Plugins",
        has_plugins: !plugins_json.is_empty(),
        plugins: plugins_json,
        categories,
        stats: PluginStats { plugin_count },
    };
    let data_value = serde_json::to_value(&data).unwrap_or_default();
    super::render_page(&engine, "my-plugins", &data_value, &user_ctx, &mkt_ctx)
}

pub(crate) async fn my_plugin_edit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let plugin_id = params.get("id");
    let is_edit = plugin_id.is_some();

    let plugin_with_assoc = if let Some(id) = plugin_id {
        repositories::find_plugin_with_associations(&pool, &user_ctx.user_id, id)
            .await
            .map_err(|e| {
                tracing::warn!(error = %e, plugin_id = %id, "Failed to fetch user plugin");
            })
            .ok()
            .flatten()
    } else {
        None
    };

    if let Some(ref pwa) = plugin_with_assoc {
        if pwa.plugin.base_plugin_id.as_deref() == Some("systemprompt") {
            if let Some(id) = plugin_id {
                return Redirect::to(&format!("/admin/my/plugins/view?id={id}")).into_response();
            }
        }
    }

    let (skills_list, agents_list, mcp_list) =
        build_association_lists(&pool, &user_ctx, plugin_with_assoc.as_ref()).await;
    let plugin = build_plugin_edit_data(plugin_with_assoc.as_ref());
    let keywords_csv = plugin_with_assoc
        .as_ref()
        .map_or(String::new(), |p| p.plugin.keywords.join(", "));

    let data = MyPluginEditPageData {
        page: "my-plugin-edit",
        title: if is_edit {
            "Edit My Plugin"
        } else {
            "Create My Plugin"
        },
        is_edit,
        plugin,
        keywords_csv,
        skills_list,
        agents_list,
        mcp_list,
    };
    let data_value = serde_json::to_value(&data).unwrap_or_default();
    super::render_page(&engine, "my-plugin-edit", &data_value, &user_ctx, &mkt_ctx)
}
