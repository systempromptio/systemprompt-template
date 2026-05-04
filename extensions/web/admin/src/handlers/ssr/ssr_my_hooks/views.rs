use crate::handlers::ssr::types::{EventBreakdownView, HookCodeEntry, HookCodeHook, HookView};
use crate::types::HOOK_TYPE_HTTP;

pub(super) fn build_event_breakdown_views(
    event_breakdown: Vec<crate::types::HookEventTypeStat>,
    quality_map: &std::collections::HashMap<
        &str,
        &crate::types::conversation_analytics::HookSessionQuality,
    >,
) -> Vec<EventBreakdownView> {
    let max_event_count = event_breakdown
        .iter()
        .map(|e| e.event_count)
        .max()
        .unwrap_or(1)
        .max(1);
    event_breakdown
        .into_iter()
        .map(|e| {
            let pct = e.event_count.saturating_mul(100) / max_event_count;
            let quality = quality_map.get(e.event_type.as_str()).copied();
            EventBreakdownView {
                event_type: e.event_type,
                event_count: e.event_count,
                error_count: e.error_count,
                content_input_bytes: e.content_input_bytes,
                content_output_bytes: e.content_output_bytes,
                pct,
                avg_quality: quality
                    .map_or_else(|| "0.0".to_string(), |q| format!("{:.1}", q.avg_quality)),
                quality_goal_pct: quality.map_or_else(
                    || "0.0".to_string(),
                    |q| format!("{:.0}", q.goal_achievement_pct),
                ),
                quality_sessions: quality.map_or(0, |q| q.session_count),
            }
        })
        .collect()
}

pub(super) fn build_hook_views(
    hooks: Vec<crate::types::UserHook>,
    plugin_name_map: &std::collections::HashMap<String, String>,
) -> Vec<HookView> {
    hooks
        .into_iter()
        .map(|h| build_hook_view(h, plugin_name_map))
        .collect()
}

fn build_hook_view(
    h: crate::types::UserHook,
    plugin_name_map: &std::collections::HashMap<String, String>,
) -> HookView {
    let hook_code_entry = if h.hook_type == HOOK_TYPE_HTTP {
        HookCodeEntry {
            matcher: h.matcher.clone(),
            hooks: vec![HookCodeHook {
                hook_type: "http".to_string(),
                url: Some(h.url.clone()),
                headers: Some(h.headers.clone()),
                command: None,
                is_async: None,
                timeout: Some(h.timeout),
            }],
        }
    } else {
        HookCodeEntry {
            matcher: h.matcher.clone(),
            hooks: vec![HookCodeHook {
                hook_type: "command".to_string(),
                url: None,
                headers: None,
                command: Some(h.command.clone()),
                is_async: Some(h.is_async),
                timeout: Some(h.timeout),
            }],
        }
    };
    let hook_code = serde_json::to_string_pretty(&[&hook_code_entry]).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to serialize hook code entry");
        String::new()
    });
    let plugin_name = h
        .plugin_id
        .as_ref()
        .and_then(|pid| plugin_name_map.get(pid))
        .cloned()
        .unwrap_or_default();
    HookView {
        id: h.id,
        hook_name: h.hook_name,
        description: h.description,
        event_type: h.event_type,
        hook_type: h.hook_type,
        matcher: h.matcher,
        url: h.url,
        command: h.command,
        headers: h.headers,
        timeout: h.timeout,
        is_async: h.is_async,
        enabled: h.enabled,
        is_default: h.is_default,
        plugin_id: h.plugin_id,
        plugin_name,
        hook_code,
    }
}
