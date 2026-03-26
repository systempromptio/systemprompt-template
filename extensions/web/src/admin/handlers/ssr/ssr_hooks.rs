use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::repositories::hook_catalog;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{HookCatalogEntry, HookDetail, MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

fn build_plugin_name_map(
    services_path: &std::path::Path,
) -> std::collections::HashMap<String, String> {
    use systemprompt::models::PluginConfigFile;

    let plugins_path = services_path.join("plugins");
    let mut map = std::collections::HashMap::new();
    if !plugins_path.exists() {
        return map;
    }
    let Ok(entries) = std::fs::read_dir(&plugins_path) else {
        return map;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let config_path = path.join("config.yaml");
        if !config_path.exists() {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&config_path) else {
            continue;
        };
        let Ok(plugin_file): Result<PluginConfigFile, _> = serde_yaml::from_str(&content) else {
            continue;
        };
        map.insert(
            plugin_file.plugin.id.clone(),
            plugin_file.plugin.name.clone(),
        );
    }
    map
}

fn catalog_entry_to_json(
    hook: &HookCatalogEntry,
    plugin_name_map: &std::collections::HashMap<String, String>,
) -> serde_json::Value {
    let is_system = hook.category == "system";

    let (plugin_names, plugin_ids): (Vec<String>, Vec<String>) = if hook.plugins.is_empty() {
        (vec![], vec![])
    } else {
        let names: Vec<String> = hook
            .plugins
            .iter()
            .map(|pid| {
                plugin_name_map
                    .get(pid)
                    .cloned()
                    .unwrap_or_else(|| pid.clone())
            })
            .collect();
        (names, hook.plugins.clone())
    };

    json!({
        "id": hook.id,
        "name": hook.name,
        "description": hook.description,
        "event": hook.event,
        "matcher": hook.matcher,
        "command": hook.command,
        "is_async": hook.is_async,
        "system": is_system,
        "category": hook.category,
        "plugins": plugin_names,
        "plugin_ids": plugin_ids,
        "visible_to": hook.visible_to,
        "enabled": hook.enabled,
        "version": hook.version,
        "tags": hook.tags,
    })
}

pub(crate) async fn hooks_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let plugin_name_map = build_plugin_name_map(&services_path);

    let hook_overrides_map = repositories::get_hook_overrides_enabled_map(&pool)
        .await
        .unwrap_or_default();

    let mut catalog_hooks = match hook_catalog::list_catalog_hooks(&pool).await {
        Ok(hooks) if !hooks.is_empty() => hooks,
        _ => {
            hook_catalog::list_file_hooks(&services_path).unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Failed to list hooks from disk");
                vec![]
            })
        }
    };

    for hook in &mut catalog_hooks {
        if let Some(&enabled) = hook_overrides_map.get(&hook.id) {
            hook.enabled = enabled;
        }
    }

    if !user_ctx.is_admin {
        catalog_hooks.retain(|h| {
            if h.visible_to.is_empty() {
                return true;
            }
            h.visible_to
                .iter()
                .any(|role| user_ctx.roles.iter().any(|r| r == role))
        });
    }

    let system_hooks: Vec<_> = catalog_hooks
        .iter()
        .filter(|h| h.category == "system")
        .collect();
    let custom_hooks: Vec<_> = catalog_hooks
        .iter()
        .filter(|h| h.category != "system")
        .collect();

    let system_json: Vec<_> = system_hooks
        .iter()
        .map(|h| catalog_entry_to_json(h, &plugin_name_map))
        .collect();
    let custom_json: Vec<_> = custom_hooks
        .iter()
        .map(|h| catalog_entry_to_json(h, &plugin_name_map))
        .collect();

    let mut unified_hooks = system_json.clone();
    unified_hooks.extend(custom_json);

    let tracked_plugin_count = {
        let mut plugin_set = std::collections::HashSet::new();
        for h in &system_hooks {
            for pid in &h.plugins {
                plugin_set.insert(pid.as_str());
            }
        }
        plugin_set.len()
    };

    let data = json!({
        "page": "hooks",
        "title": "Hooks",
        "hooks": unified_hooks,
        "has_hooks": !unified_hooks.is_empty(),
        "system_event_count": system_hooks.len(),
        "system_plugin_count": tracked_plugin_count,
        "custom_count": custom_hooks.len(),
        "total_hooks": system_hooks.len() + custom_hooks.len(),
    });
    super::render_page(&engine, "hooks", &data, &user_ctx, &mkt_ctx)
}

pub(crate) async fn hook_edit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let hook_id = params.get("id");
    let is_edit = hook_id.is_some();
    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let hook: Option<HookDetail> = if let Some(id) = hook_id {
        match hook_catalog::get_catalog_hook(&pool, id).await {
            Ok(Some(entry)) => Some(hook_catalog::catalog_to_detail(&entry)),
            _ => repositories::hooks::get_hook(&services_path, id)
                .map_err(|e| {
                    tracing::warn!(error = %e, hook_id = %id, "Failed to fetch hook");
                })
                .ok()
                .flatten(),
        }
    } else {
        None
    };

    let roles = user_ctx.roles.clone();
    let plugins =
        repositories::list_plugins_for_roles(&services_path, &roles).unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list plugins for roles");
            vec![]
        });

    let data = json!({
        "page": "hook-edit",
        "title": if is_edit { "Edit Hook" } else { "Create Hook" },
        "is_edit": is_edit,
        "hook": hook,
        "plugins": plugins,
        "hook_events": [
            "PostToolUse", "PostToolUseFailure", "PreToolUse",
            "UserPromptSubmit", "SessionStart", "SessionEnd",
            "Stop", "SubagentStart", "SubagentStop", "Notification"
        ],
    });
    super::render_page(&engine, "hook-edit", &data, &user_ctx, &mkt_ctx)
}
