use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{HookDetail, MarketplaceContext, UserContext};
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

fn list_hooks_from_filesystem(
    services_path: &std::path::Path,
) -> Vec<HookDetail> {
    let plugins_dir = services_path.join("plugins");
    let mut hooks = Vec::new();
    if !plugins_dir.exists() {
        return hooks;
    }
    let Ok(entries) = std::fs::read_dir(&plugins_dir) else {
        return hooks;
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
        let plugin_id = entry.file_name().to_string_lossy().to_string();
        let Ok(content) = std::fs::read_to_string(&config_path) else {
            continue;
        };
        let Ok(doc): Result<serde_yaml::Value, _> = serde_yaml::from_str(&content) else {
            continue;
        };
        collect_hooks_from_plugin(&plugin_id, &doc, &mut hooks);
    }
    hooks.sort_by(|a, b| a.id.cmp(&b.id));
    hooks
}

fn collect_hooks_from_plugin(
    plugin_id: &str,
    doc: &serde_yaml::Value,
    hooks: &mut Vec<HookDetail>,
) {
    let Some(hooks_map) = doc
        .get("plugin")
        .and_then(|p| p.get("hooks"))
        .and_then(|h| h.as_mapping())
    else {
        return;
    };
    for (event_key, event_entries) in hooks_map {
        let event = event_key.as_str().unwrap_or("unknown").to_string();
        let Some(entries) = event_entries.as_sequence() else {
            continue;
        };
        for (idx, entry_val) in entries.iter().enumerate() {
            let matcher = entry_val
                .get("matcher")
                .and_then(|m| m.as_str())
                .unwrap_or("")
                .to_string();
            let Some(hook_list) = entry_val.get("hooks").and_then(|h| h.as_sequence()) else {
                continue;
            };
            for (hidx, hook) in hook_list.iter().enumerate() {
                let id = format!(
                    "{plugin_id}_{event}_{idx}{}",
                    if hidx > 0 {
                        format!("_{hidx}")
                    } else {
                        String::new()
                    }
                );
                hooks.push(HookDetail {
                    id,
                    plugin_id: plugin_id.to_string(),
                    name: hook
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("")
                        .to_string(),
                    description: hook
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string(),
                    event: event.clone(),
                    matcher: matcher.clone(),
                    command: hook
                        .get("command")
                        .and_then(|c| c.as_str())
                        .unwrap_or("")
                        .to_string(),
                    is_async: hook
                        .get("async")
                        .and_then(serde_yaml::Value::as_bool)
                        .unwrap_or(false),
                    system: false,
                    visible_to: vec![],
                    enabled: true,
                });
            }
        }
    }
}

fn hook_detail_to_json(
    hook: &HookDetail,
    plugin_name_map: &std::collections::HashMap<String, String>,
) -> serde_json::Value {
    let plugin_name = plugin_name_map
        .get(&hook.plugin_id)
        .cloned()
        .unwrap_or_else(|| hook.plugin_id.clone());

    json!({
        "id": hook.id,
        "name": hook.name,
        "description": hook.description,
        "event": hook.event,
        "matcher": hook.matcher,
        "command": hook.command,
        "is_async": hook.is_async,
        "system": hook.system,
        "plugin_id": hook.plugin_id,
        "plugin_name": plugin_name,
        "visible_to": hook.visible_to,
        "enabled": hook.enabled,
    })
}

pub(crate) async fn hooks_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(_pool): State<Arc<PgPool>>,
) -> Response {
    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let plugin_name_map = build_plugin_name_map(&services_path);

    let mut all_hooks = list_hooks_from_filesystem(&services_path);

    if !user_ctx.is_admin {
        all_hooks.retain(|h| {
            if h.visible_to.is_empty() {
                return true;
            }
            h.visible_to
                .iter()
                .any(|role| user_ctx.roles.iter().any(|r| r == role))
        });
    }

    let system_hooks: Vec<&HookDetail> = all_hooks.iter().filter(|h| h.system).collect();
    let custom_hooks: Vec<&HookDetail> = all_hooks.iter().filter(|h| !h.system).collect();

    let system_json: Vec<_> = system_hooks
        .iter()
        .map(|h| hook_detail_to_json(h, &plugin_name_map))
        .collect();
    let custom_json: Vec<_> = custom_hooks
        .iter()
        .map(|h| hook_detail_to_json(h, &plugin_name_map))
        .collect();

    let mut unified_hooks = system_json.clone();
    unified_hooks.extend(custom_json);

    let tracked_plugin_count = {
        let mut plugin_set = std::collections::HashSet::new();
        for h in &system_hooks {
            plugin_set.insert(h.plugin_id.as_str());
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
    State(_pool): State<Arc<PgPool>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let hook_id = params.get("id");
    let is_edit = hook_id.is_some();
    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let hook: Option<HookDetail> = if let Some(id) = hook_id {
        let hooks = list_hooks_from_filesystem(&services_path);
        hooks.into_iter().find(|h| h.id == *id)
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
