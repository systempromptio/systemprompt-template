use std::collections::HashMap;
use std::path::Path;

use super::super::types::HookOverview;

pub(crate) fn resolve_plugin_hooks(
    config_path: &Path,
    plugin_id: &str,
    enabled_map: &HashMap<String, bool>,
) -> Vec<HookOverview> {
    let Ok(content) = std::fs::read_to_string(config_path) else {
        return Vec::new();
    };
    let Ok(doc) = serde_yaml::from_str::<serde_yaml::Value>(&content) else {
        return Vec::new();
    };

    let Some(hooks_map) = doc
        .get("plugin")
        .and_then(|p| p.get("hooks"))
        .and_then(|h| h.as_mapping())
    else {
        return Vec::new();
    };

    let mut result = Vec::new();
    for (event_key, matchers) in hooks_map {
        let Some(event) = event_key.as_str() else {
            tracing::debug!(key = ?event_key, "Skipping non-string hook event key");
            continue;
        };
        let event = event.to_string();
        if let Some(matcher_seq) = matchers.as_sequence() {
            for (idx, matcher_entry) in matcher_seq.iter().enumerate() {
                let matcher = matcher_entry
                    .get("matcher")
                    .and_then(|m| m.as_str())
                    .unwrap_or("*")
                    .to_string();
                if let Some(hook_seq) = matcher_entry.get("hooks").and_then(|h| h.as_sequence()) {
                    for (hidx, hook) in hook_seq.iter().enumerate() {
                        let id = format!(
                            "{plugin_id}_{event}_{idx}{}",
                            if hidx > 0 {
                                format!("_{hidx}")
                            } else {
                                String::new()
                            }
                        );
                        let enabled = *enabled_map.get(&id).unwrap_or(&true);
                        let command = hook
                            .get("command")
                            .and_then(|c| c.as_str())
                            .unwrap_or("")
                            .to_string();
                        let is_async = hook
                            .get("async")
                            .and_then(serde_yaml::Value::as_bool)
                            .unwrap_or(false);
                        let name = hook
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or("")
                            .to_string();
                        let description = hook
                            .get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("")
                            .to_string();
                        result.push(HookOverview {
                            event: event.clone(),
                            matcher: matcher.clone(),
                            command,
                            is_async,
                            name,
                            description,
                            enabled,
                            id,
                        });
                    }
                }
            }
        }
    }
    result
}
