use std::path::Path;

use super::export::PluginFile;
use super::export_resolvers::{
    build_agent_md, build_skill_md, collect_skill_auxiliary_files, resolve_export_agents,
    resolve_export_skills,
};
use super::export_validation::{build_manifest, compute_content_version, validate_bundle};
use systemprompt_web_shared::error::MarketplaceError;

pub(super) struct PluginBuildContext<'a> {
    pub plugin_id: &'a str,
    pub plugin: &'a super::super::types::PlatformPluginConfig,
    pub plugins_path: &'a Path,
    pub skills_path: &'a Path,
    pub services_path: &'a Path,
    pub platform_url: &'a str,
    pub token: Option<&'a str>,
}

pub(super) fn build_plugin_files(
    ctx: &PluginBuildContext<'_>,
) -> Result<Vec<PluginFile>, MarketplaceError> {
    let mut files = Vec::new();

    let agents_path = ctx.services_path.join("agents");
    let skill_ids = resolve_export_skills(&ctx.plugin.base, ctx.skills_path, &agents_path)?;
    build_skill_files(
        &skill_ids,
        ctx.platform_url,
        ctx.token,
        ctx.plugin_id,
        &mut files,
    )?;
    build_agent_files(&ctx.plugin.base, ctx.services_path, &mut files)?;
    build_mcp_files(
        &ctx.plugin.base,
        ctx.services_path,
        ctx.platform_url,
        &mut files,
    )?;

    build_script_files(ctx, &mut files)?;

    let content_version = compute_content_version(&ctx.plugin.base.version, &files);
    let manifest = build_manifest(&ctx.plugin.base, Some(&content_version));
    files.push(PluginFile {
        path: ".claude-plugin/plugin.json".to_string(),
        content: serde_json::to_string_pretty(&manifest)?,
        executable: false,
    });

    validate_bundle(&files, skill_ids.len());

    Ok(files)
}

fn build_skill_files(
    skill_ids: &[(String, std::path::PathBuf)],
    _platform_url: &str,
    _token: Option<&str>,
    _plugin_id: &str,
    files: &mut Vec<PluginFile>,
) -> Result<(), MarketplaceError> {
    for (skill_id, skill_dir) in skill_ids {
        let kebab_name = skill_id.replace('_', "-");
        let content = build_skill_md(skill_id, skill_dir, None)?;
        files.push(PluginFile {
            path: format!("skills/{kebab_name}/SKILL.md"),
            content,
            executable: false,
        });

        for (aux_path, aux_content, aux_executable) in
            collect_skill_auxiliary_files(skill_id, skill_dir)
        {
            files.push(PluginFile {
                path: aux_path,
                content: aux_content,
                executable: aux_executable,
            });
        }
    }
    Ok(())
}

fn build_agent_files(
    plugin: &systemprompt::models::PluginConfig,
    services_path: &Path,
    files: &mut Vec<PluginFile>,
) -> Result<(), MarketplaceError> {
    let agent_ids = resolve_export_agents(plugin, services_path)?;
    let agents_dir = services_path.join("agents");
    for agent_id in &agent_ids {
        let agent_md = build_agent_md(agent_id, &agents_dir)?;
        files.push(PluginFile {
            path: format!("agents/{agent_id}.md"),
            content: agent_md,
            executable: false,
        });
    }
    Ok(())
}

fn build_mcp_files(
    plugin: &systemprompt::models::PluginConfig,
    services_path: &Path,
    platform_url: &str,
    files: &mut Vec<PluginFile>,
) -> Result<(), MarketplaceError> {
    if plugin.mcp_servers.is_empty() {
        return Ok(());
    }
    let mut mcp_server_entries = std::collections::HashMap::new();
    for mcp_name in &plugin.mcp_servers {
        let Ok(Some(server_detail)) = super::mcp_servers::find_mcp_server(services_path, mcp_name)
        else {
            tracing::warn!(mcp_server = %mcp_name, "MCP server not found during plugin export, skipping");
            continue;
        };
        let url = if !server_detail.endpoint.is_empty()
            && !server_detail.endpoint.starts_with("http://localhost")
        {
            server_detail.endpoint
        } else {
            format!("{platform_url}/api/v1/mcp/{mcp_name}/mcp")
        };
        mcp_server_entries.insert(
            mcp_name.clone(),
            super::export::McpServerEntry {
                server_type: "http".to_string(),
                url,
            },
        );
    }
    let mcp_config = super::export::McpConfigFile {
        mcp_servers: mcp_server_entries,
    };
    files.push(PluginFile {
        path: ".mcp.json".to_string(),
        content: serde_json::to_string_pretty(&mcp_config)?,
        executable: false,
    });
    Ok(())
}

fn build_script_files(
    ctx: &PluginBuildContext<'_>,
    files: &mut Vec<PluginFile>,
) -> Result<(), MarketplaceError> {
    for script in &ctx.plugin.base.scripts {
        if script.source == "generated:tracking" {
            continue;
        }
        let source_path = ctx.plugins_path.join(ctx.plugin_id).join(&script.source);
        if source_path.exists() {
            let content = std::fs::read_to_string(&source_path)?;
            files.push(PluginFile {
                path: format!("scripts/{}", script.name),
                content,
                executable: true,
            });
        }
    }
    Ok(())
}

pub(super) fn build_hook_files(
    platform_url: &str,
    token: &str,
    files: &mut Vec<PluginFile>,
) -> Result<(), MarketplaceError> {
    use std::collections::HashMap;

    use super::super::types::hooks_export::{
        HookEventType, HookHandler, HooksFile, HttpHook, MatcherGroup,
    };

    if platform_url.is_empty() || token.is_empty() {
        return Ok(());
    }

    let govern_url = format!("{platform_url}/api/public/hooks/govern");
    let track_url = format!("{platform_url}/api/public/hooks/track");
    let auth_header = format!("Bearer {token}");

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
        hooks: vec![HookHandler::Http(HttpHook {
            url: govern_url,
            headers: Some(HashMap::from([(
                "Authorization".to_string(),
                auth_header.clone(),
            )])),
            allowed_env_vars: None,
            timeout: Some(10),
            is_async: None,
        })],
    };
    hooks.insert(HookEventType::PreToolUse, vec![govern_group]);

    for event in tracking_events {
        let matcher_group = MatcherGroup {
            matcher: "*".to_string(),
            hooks: vec![HookHandler::Http(HttpHook {
                url: track_url.clone(),
                headers: Some(HashMap::from([(
                    "Authorization".to_string(),
                    auth_header.clone(),
                )])),
                allowed_env_vars: None,
                timeout: Some(30),
                is_async: Some(true),
            })],
        };
        hooks.insert(event, vec![matcher_group]);
    }

    let hooks_file = HooksFile {
        description: None,
        hooks,
    };
    files.push(PluginFile {
        path: "hooks/hooks.json".to_string(),
        content: serde_json::to_string_pretty(&hooks_file)?,
        executable: false,
    });

    Ok(())
}
