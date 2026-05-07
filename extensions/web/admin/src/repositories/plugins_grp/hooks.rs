use std::path::Path;

use systemprompt::models::{HookEventsConfig, HookMatcher};
use systemprompt_web_shared::error::MarketplaceError;

use crate::types::ConfiguredHook;

pub fn list_configured_hooks(
    _services_path: &Path,
    roles: &[String],
) -> Result<Vec<ConfiguredHook>, MarketplaceError> {
    use crate::types::ROLE_ADMIN;
    let is_admin = roles.iter().any(|r| r == ROLE_ADMIN);
    let mut out = Vec::new();
    for (_id, plugin) in super::plugin_loader::load_all_plugins()? {
        if !plugin.base.enabled && !is_admin {
            continue;
        }
        if !is_admin && !plugin.roles.is_empty() && !plugin.roles.iter().any(|r| roles.contains(r))
        {
            continue;
        }
        let plugin_id = plugin.base.id.to_string();
        flatten_into(&plugin_id, &plugin.base.hooks, &mut out);
    }
    Ok(out)
}

fn flatten_into(plugin_id: &str, hooks: &HookEventsConfig, out: &mut Vec<ConfiguredHook>) {
    let groups: [(&str, &[HookMatcher]); 10] = [
        ("PreToolUse", &hooks.pre_tool_use),
        ("PostToolUse", &hooks.post_tool_use),
        ("PostToolUseFailure", &hooks.post_tool_use_failure),
        ("SessionStart", &hooks.session_start),
        ("SessionEnd", &hooks.session_end),
        ("UserPromptSubmit", &hooks.user_prompt_submit),
        ("Notification", &hooks.notification),
        ("Stop", &hooks.stop),
        ("SubagentStart", &hooks.subagent_start),
        ("SubagentStop", &hooks.subagent_stop),
    ];
    for (event, matchers) in groups {
        for matcher in matchers {
            for (idx, action) in matcher.hooks.iter().enumerate() {
                out.push(ConfiguredHook {
                    id: format!("{plugin_id}:{event}:{}:{idx}", matcher.matcher),
                    plugin_id: plugin_id.to_string(),
                    event: event.to_string(),
                    matcher: matcher.matcher.clone(),
                    command: action
                        .command
                        .clone()
                        .or_else(|| action.prompt.clone())
                        .unwrap_or_default(),
                    is_async: action.r#async,
                    timeout_ms: action.timeout,
                });
            }
        }
    }
}
