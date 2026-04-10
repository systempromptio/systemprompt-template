use std::path::Path;

use super::super::types::HookDetail;
use super::export_scripts::TRACKING_EVENTS;

pub fn list_hooks(services_path: &Path) -> Result<Vec<HookDetail>, std::io::Error> {
    let plugins_dir = services_path.join("plugins");
    let mut hooks = Vec::new();
    if !plugins_dir.exists() {
        return Ok(hooks);
    }

    collect_system_hooks("common-skills", &mut hooks);

    for entry in std::fs::read_dir(&plugins_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let config_path = path.join("config.yaml");
        if !config_path.exists() {
            continue;
        }
        let plugin_id = entry.file_name().to_string_lossy().into_owned();

        let content = std::fs::read_to_string(&config_path)?;
        let doc: serde_yaml::Value = match serde_yaml::from_str(&content) {
            Ok(c) => c,
            Err(_) => continue,
        };
        collect_custom_hooks(&plugin_id, &doc, &mut hooks);
    }
    hooks.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(hooks)
}

fn collect_system_hooks(plugin_id: &str, hooks: &mut Vec<HookDetail>) {
    let script_name = format!("track-{plugin_id}-usage.sh");
    let command = format!("${{CLAUDE_PLUGIN_ROOT}}/scripts/{script_name}");
    for event in TRACKING_EVENTS {
        hooks.push(HookDetail {
            id: format!("sys_{plugin_id}_{event}"),
            plugin_id: plugin_id.to_string(),
            name: format!("{event} Tracking"),
            description: format!("System usage tracking for {event} events"),
            event: (*event).to_string(),
            matcher: "*".to_string(),
            command: command.clone(),
            is_async: true,
            system: true,
            visible_to: vec!["admin".to_string()],
            enabled: true,
        });
    }
}

fn collect_custom_hooks(plugin_id: &str, doc: &serde_yaml::Value, hooks: &mut Vec<HookDetail>) {
    let Some(hooks_map) = doc
        .get("plugin")
        .and_then(|p| p.get("hooks"))
        .and_then(|h| h.as_mapping())
    else {
        return;
    };
    for (event_key, event_entries) in hooks_map {
        let Some(event) = event_key.as_str() else {
            tracing::debug!(key = ?event_key, "Skipping non-string hook event key");
            continue;
        };
        let event = event.to_string();
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
                hooks.push(parse_hook_detail(id, plugin_id, &event, &matcher, hook));
            }
        }
    }
}

fn parse_hook_detail(
    id: String,
    plugin_id: &str,
    event: &str,
    matcher: &str,
    hook: &serde_yaml::Value,
) -> HookDetail {
    HookDetail {
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
        event: event.to_string(),
        matcher: matcher.to_string(),
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
    }
}

pub fn get_hook(services_path: &Path, hook_id: &str) -> Result<Option<HookDetail>, std::io::Error> {
    let hooks = list_hooks(services_path)?;
    Ok(hooks.into_iter().find(|h| h.id == hook_id))
}
