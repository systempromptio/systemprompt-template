use std::path::Path;

use super::export::PluginFile;
use super::export_builders_env::build_env_files;
use super::export_resolvers::{
    build_agent_md, build_skill_md, collect_skill_auxiliary_files, resolve_export_agents,
    resolve_export_skills,
};
use super::export_scripts::{
    build_hooks_file, build_transcript_script_from_template,
    build_transcript_script_ps1_from_template, collect_platform_hooks,
    collect_tracking_http_hooks, env_hook_entry, governance_http_hook, transcript_hook_entry,
};
use super::export_validation::{build_manifest, compute_content_version, validate_bundle};

pub(super) struct PluginBuildContext<'a> {
    pub plugin_id: &'a str,
    pub plugin: &'a super::super::types::PlatformPluginConfig,
    pub plugins_path: &'a Path,
    pub skills_path: &'a Path,
    pub services_path: &'a Path,
    pub plugin_token: &'a str,
    pub platform_url: &'a str,
    pub platform: &'a str,
    pub env_vars: &'a std::collections::HashMap<String, String>,
}

pub(super) fn build_plugin_files(
    ctx: &PluginBuildContext<'_>,
) -> Result<Vec<PluginFile>, anyhow::Error> {
    let is_windows = ctx.platform == "windows";
    let mut files = Vec::new();

    let agents_path = ctx.services_path.join("agents");
    let skill_ids = resolve_export_skills(&ctx.plugin.base, ctx.skills_path, &agents_path)?;
    build_skill_files(&skill_ids, &mut files)?;
    build_agent_files(&ctx.plugin.base, ctx.services_path, &mut files)?;
    build_mcp_files(
        &ctx.plugin.base,
        ctx.services_path,
        ctx.platform_url,
        &mut files,
    )?;

    // Transcript script (command hook — needs local file access)
    let transcript_script_name = format!("upload-{}-transcript.sh", ctx.plugin_id);
    if ctx.plugin_id == "common-skills" {
        if is_windows {
            let ps1_name = transcript_script_name.replace(".sh", ".ps1");
            files.push(PluginFile {
                path: format!("scripts/{ps1_name}"),
                content: build_transcript_script_ps1_from_template(
                    ctx.services_path,
                    ctx.plugin_token,
                    ctx.platform_url,
                    ctx.plugin_id,
                ),
                executable: true,
            });
        } else {
            files.push(PluginFile {
                path: format!("scripts/{transcript_script_name}"),
                content: build_transcript_script_from_template(
                    ctx.services_path,
                    ctx.plugin_token,
                    ctx.platform_url,
                    ctx.plugin_id,
                ),
                executable: true,
            });
        }
    }

    let env_files_generated = if ctx.plugin.variables.is_empty() {
        false
    } else {
        build_env_files(ctx, &mut files)
    };

    // HTTP hooks for tracking and governance (all plugins)
    let mut hook_entries =
        collect_tracking_http_hooks(ctx.platform_url, ctx.plugin_id, ctx.plugin_token);
    hook_entries.push(governance_http_hook(
        ctx.platform_url,
        ctx.plugin_id,
        ctx.plugin_token,
    ));
    if ctx.plugin_id == "common-skills" {
        hook_entries.push(transcript_hook_entry(&transcript_script_name, is_windows));
    }
    if !ctx.plugin.base.hooks.is_empty() {
        hook_entries.extend(collect_platform_hooks(&ctx.plugin.base, is_windows));
    }
    if env_files_generated {
        hook_entries.insert(0, env_hook_entry(is_windows));
    }
    let hooks_json = build_hooks_file(&hook_entries);
    files.push(PluginFile {
        path: "hooks/hooks.json".to_string(),
        content: serde_json::to_string_pretty(&hooks_json)?,
        executable: false,
    });

    build_script_files(ctx, is_windows, &mut files)?;

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
    files: &mut Vec<PluginFile>,
) -> Result<(), anyhow::Error> {
    for (skill_id, skill_dir) in skill_ids {
        let kebab_name = skill_id.replace('_', "-");
        let content = build_skill_md(skill_id, skill_dir)?;
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
) -> Result<(), anyhow::Error> {
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
) -> Result<(), anyhow::Error> {
    if plugin.mcp_servers.is_empty() {
        return Ok(());
    }
    let mut mcp_servers = serde_json::Map::new();
    for mcp_name in &plugin.mcp_servers {
        let url = match super::mcp_servers::get_mcp_server(services_path, mcp_name) {
            Ok(Some(server))
                if !server.endpoint.is_empty()
                    && !server.endpoint.starts_with("http://localhost") =>
            {
                server.endpoint
            }
            _ => format!("{platform_url}/api/v1/mcp/{mcp_name}/mcp"),
        };
        let mut server = serde_json::Map::new();
        server.insert(
            "type".to_string(),
            serde_json::Value::String("http".to_string()),
        );
        server.insert("url".to_string(), serde_json::Value::String(url));
        mcp_servers.insert(mcp_name.clone(), serde_json::Value::Object(server));
    }
    let mcp_json = serde_json::json!({ "mcpServers": mcp_servers });
    files.push(PluginFile {
        path: ".mcp.json".to_string(),
        content: serde_json::to_string_pretty(&mcp_json)?,
        executable: false,
    });
    Ok(())
}

fn build_script_files(
    ctx: &PluginBuildContext<'_>,
    _is_windows: bool,
    files: &mut Vec<PluginFile>,
) -> Result<(), anyhow::Error> {
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
