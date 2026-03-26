use std::collections::HashMap;

use super::super::types::UserHook;

pub(super) use super::export_scripts_marketplace::{build_marketplace, load_marketplace_identity};
pub(super) use super::export_scripts_templates::{
    build_transcript_script_from_template, build_transcript_script_ps1_from_template,
    transcript_hook_entry,
};

pub(super) const TRACKING_EVENTS: &[&str] = &[
    "PostToolUse",
    "PostToolUseFailure",
    "UserPromptSubmit",
    "SessionStart",
    "SessionEnd",
    "Stop",
    "SubagentStart",
    "SubagentStop",
    "Notification",
    "TaskCompleted",
    "PreCompact",
    "TeammateIdle",
    "PermissionRequest",
    "ConfigChange",
    "WorktreeCreate",
    "WorktreeRemove",
];

pub(super) enum HookType {
    Command {
        command: String,
    },
    Http {
        url: String,
        headers: HashMap<String, String>,
        timeout: Option<u32>,
    },
}

pub(super) struct HookEntry {
    pub event: String,
    pub matcher: Option<String>,
    pub hook_type: HookType,
    pub is_async: bool,
}

pub(super) fn build_hooks_file(entries: &[HookEntry]) -> serde_json::Value {
    let mut events = serde_json::Map::new();
    for entry in entries {
        let mut handler = serde_json::Map::new();
        match &entry.hook_type {
            HookType::Command { command } => {
                handler.insert("type".into(), serde_json::Value::String("command".into()));
                handler.insert(
                    "command".into(),
                    serde_json::Value::String(command.clone()),
                );
            }
            HookType::Http {
                url,
                headers,
                timeout,
            } => {
                handler.insert("type".into(), serde_json::Value::String("http".into()));
                handler.insert("url".into(), serde_json::Value::String(url.clone()));
                if !headers.is_empty() {
                    let headers_map: serde_json::Map<String, serde_json::Value> = headers
                        .iter()
                        .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                        .collect();
                    handler.insert(
                        "headers".into(),
                        serde_json::Value::Object(headers_map),
                    );
                }
                if let Some(t) = timeout {
                    handler.insert(
                        "timeout".into(),
                        serde_json::Value::Number((*t).into()),
                    );
                }
            }
        }
        if entry.is_async {
            handler.insert("async".into(), serde_json::Value::Bool(true));
        }

        let mut matcher_obj = serde_json::Map::new();
        if let Some(ref m) = entry.matcher {
            if m != "*" {
                matcher_obj.insert("matcher".into(), serde_json::Value::String(m.clone()));
            }
        }
        matcher_obj.insert(
            "hooks".into(),
            serde_json::Value::Array(vec![serde_json::Value::Object(handler)]),
        );

        let arr = events
            .entry(entry.event.clone())
            .or_insert_with(|| serde_json::Value::Array(Vec::new()));
        if let serde_json::Value::Array(ref mut v) = arr {
            v.push(serde_json::Value::Object(matcher_obj));
        }
    }

    serde_json::json!({
        "hooks": events
    })
}

pub(super) fn collect_platform_hooks(
    plugin: &systemprompt::models::PluginConfig,
    is_windows: bool,
) -> Vec<HookEntry> {
    let events_with_matchers: &[(&str, &[systemprompt::models::HookMatcher])] = &[
        ("PreToolUse", &plugin.hooks.pre_tool_use),
        ("PostToolUse", &plugin.hooks.post_tool_use),
        ("PostToolUseFailure", &plugin.hooks.post_tool_use_failure),
        ("SessionStart", &plugin.hooks.session_start),
        ("SessionEnd", &plugin.hooks.session_end),
        ("UserPromptSubmit", &plugin.hooks.user_prompt_submit),
        ("Notification", &plugin.hooks.notification),
        ("Stop", &plugin.hooks.stop),
        ("SubagentStop", &plugin.hooks.subagent_stop),
    ];

    let mut entries = Vec::new();
    for &(event_name, matchers) in events_with_matchers {
        for matcher in matchers {
            for action in &matcher.hooks {
                if let Some(ref cmd) = action.command {
                    let command = if is_windows {
                        to_windows_command(cmd)
                    } else {
                        cmd.clone()
                    };
                    entries.push(HookEntry {
                        event: event_name.to_string(),
                        matcher: Some(matcher.matcher.clone()),
                        hook_type: HookType::Command { command },
                        is_async: action.r#async,
                    });
                }
            }
        }
    }
    entries
}

pub(super) fn collect_user_hooks(hooks: &[&UserHook]) -> Vec<HookEntry> {
    hooks
        .iter()
        .map(|h| HookEntry {
            event: h.event.clone(),
            matcher: Some(h.matcher.clone()),
            hook_type: HookType::Command {
                command: h.command.clone(),
            },
            is_async: h.is_async,
        })
        .collect()
}

pub(super) fn env_hook_entry(is_windows: bool) -> HookEntry {
    let command = if is_windows {
        "if ($env:CLAUDE_ENV_FILE) { Get-Content \"$env:CLAUDE_PLUGIN_ROOT\\.env.plugin\" | Where-Object { $_ -match '^([^#][^=]*)=(.*)$' } | ForEach-Object { \"export $($matches[1])=$($matches[2])\" } | Add-Content $env:CLAUDE_ENV_FILE }; Write-Output '{\"result\":\"env loaded\"}'".to_string()
    } else {
        "if [ -n \"$CLAUDE_ENV_FILE\" ]; then grep -v '^#' \"${CLAUDE_PLUGIN_ROOT}/.env.plugin\" | grep -v '^$' | sed 's/^/export /' >> \"$CLAUDE_ENV_FILE\"; fi && echo '{\"result\":\"env loaded\"}'".to_string()
    };
    HookEntry {
        event: "SessionStart".to_string(),
        matcher: None,
        hook_type: HookType::Command { command },
        is_async: false,
    }
}

pub(super) fn format_script_command(script_name: &str, is_windows: bool) -> String {
    if is_windows {
        let ps1_name = script_name.replace(".sh", ".ps1");
        format!(
            "powershell -ExecutionPolicy Bypass -Command \"& '$env:CLAUDE_PLUGIN_ROOT/scripts/{ps1_name}'\""
        )
    } else {
        format!("${{CLAUDE_PLUGIN_ROOT}}/scripts/{script_name}")
    }
}

fn to_windows_command(cmd: &str) -> String {
    let win_cmd = cmd
        .replace(".sh", ".ps1")
        .replace("${CLAUDE_PLUGIN_ROOT}", "$env:CLAUDE_PLUGIN_ROOT");
    format!("powershell -ExecutionPolicy Bypass -Command \"& '{win_cmd}'\"")
}

fn http_headers(token: &str) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), format!("Bearer {token}"));
    headers.insert("Content-Type".to_string(), "application/json".to_string());
    headers
}

pub(super) fn collect_tracking_http_hooks(
    platform_url: &str,
    plugin_id: &str,
    token: &str,
) -> Vec<HookEntry> {
    let url = format!("{platform_url}/api/public/hooks/track?plugin_id={plugin_id}");
    let headers = http_headers(token);

    TRACKING_EVENTS
        .iter()
        .map(|event| HookEntry {
            event: (*event).to_string(),
            matcher: None,
            hook_type: HookType::Http {
                url: url.clone(),
                headers: headers.clone(),
                timeout: None,
            },
            is_async: true,
        })
        .collect()
}

pub(super) fn governance_http_hook(
    platform_url: &str,
    plugin_id: &str,
    token: &str,
) -> HookEntry {
    let url = format!("{platform_url}/api/public/hooks/govern?plugin_id={plugin_id}");
    HookEntry {
        event: "PreToolUse".to_string(),
        matcher: None,
        hook_type: HookType::Http {
            url,
            headers: http_headers(token),
            timeout: Some(10),
        },
        is_async: false,
    }
}
