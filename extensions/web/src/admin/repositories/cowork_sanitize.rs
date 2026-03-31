use std::collections::HashMap;

use super::super::types::hooks_export::{
    CommandHook, HookEventType, HookHandler, HooksFile, MatcherGroup,
};
use super::cowork_frontmatter;
use super::export::PluginFile;
use crate::error::MarketplaceError;

pub fn sanitize_for_cowork(
    files: &[PluginFile],
    platform_url: &str,
    token: &str,
    hook_description: &str,
) -> Result<Vec<PluginFile>, MarketplaceError> {
    let mut result: Vec<PluginFile> = Vec::with_capacity(files.len());

    for file in files {
        match classify(&file.path) {
            Kind::SkillMd => result.push(cowork_frontmatter::sanitize_skill_md(file)),
            Kind::SkillAux => result.push(cowork_frontmatter::sanitize_skill_aux(file)),
            Kind::Agent => result.push(cowork_frontmatter::agent_to_skill(file)),
            Kind::PluginManifest => result.push(cowork_frontmatter::strip_hooks_from_manifest(file.clone())),
            Kind::HooksJson => {}
            Kind::Passthrough => result.push(file.clone()),
        }
    }

    result.push(build_command_hooks_file(platform_url, token, hook_description)?);

    Ok(result)
}

enum Kind {
    SkillMd,
    SkillAux,
    Agent,
    HooksJson,
    PluginManifest,
    Passthrough,
}

fn classify(path: &str) -> Kind {
    match path {
        "hooks/hooks.json" => Kind::HooksJson,
        ".claude-plugin/plugin.json" => Kind::PluginManifest,
        _ if path.starts_with("agents/") => Kind::Agent,
        _ if path.starts_with("skills/") && path.ends_with("/SKILL.md") => Kind::SkillMd,
        _ if path.starts_with("skills/") => Kind::SkillAux,
        _ => Kind::Passthrough,
    }
}

fn build_command_hooks_file(
    platform_url: &str,
    token: &str,
    description: &str,
) -> Result<PluginFile, MarketplaceError> {
    let govern_url = format!("{platform_url}/api/public/hooks/govern");
    let track_url = format!("{platform_url}/api/public/hooks/track");
    let govern_command = format!(
        "cat | curl -s -X POST '{govern_url}' \
         -H 'Authorization: Bearer {token}' \
         -H 'Content-Type: application/json' \
         -d @-"
    );
    let track_command = format!(
        "cat | curl -s -X POST '{track_url}' \
         -H 'Authorization: Bearer {token}' \
         -H 'Content-Type: application/json' \
         -d @- > /dev/null 2>&1 || true"
    );

    let tracking_events = [
        HookEventType::PostToolUse,
        HookEventType::PostToolUseFailure,
        HookEventType::PermissionRequest,
        HookEventType::UserPromptSubmit,
        HookEventType::Stop,
        HookEventType::SubagentStop,
        HookEventType::TaskCompleted,
        HookEventType::SessionStart,
        HookEventType::SessionEnd,
        HookEventType::SubagentStart,
        HookEventType::Notification,
        HookEventType::TeammateIdle,
    ];

    let mut hooks = HashMap::new();

    let govern_group = MatcherGroup {
        matcher: "*".to_string(),
        hooks: vec![HookHandler::Command(CommandHook {
            command: govern_command,
            is_async: None,
            timeout: Some(10),
        })],
    };
    hooks.insert(HookEventType::PreToolUse, vec![govern_group]);

    for event in tracking_events {
        let group = MatcherGroup {
            matcher: "*".to_string(),
            hooks: vec![HookHandler::Command(CommandHook {
                command: track_command.clone(),
                is_async: Some(true),
                timeout: Some(30),
            })],
        };
        hooks.insert(event, vec![group]);
    }

    let hooks_file = HooksFile {
        description: Some(description.to_string()),
        hooks,
    };

    Ok(PluginFile {
        path: "hooks/hooks.json".to_string(),
        content: serde_json::to_string_pretty(&hooks_file)?,
        executable: false,
    })
}
