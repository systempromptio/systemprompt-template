use std::collections::HashMap;

use sqlx::PgPool;
use systemprompt::identifiers::{McpServerId, UserId};

use crate::repositories::skill_secrets::resolve_secrets_for_skill;
use super::types::{ManifestAuthor, McpConfigFile, McpServerEntry, PluginFile, PluginManifest};

#[derive(Debug, Clone, Copy)]
pub struct BundleContext<'a> {
    pub master_key: Option<&'a [u8; 32]>,
    pub pool: &'a PgPool,
    pub user_id: &'a UserId,
    pub platform_url: &'a str,
    pub plugin_id: &'a str,
    pub token: Option<&'a str>,
}

pub async fn build_skill_files(
    files: &mut Vec<PluginFile>,
    skill: &crate::types::UserSkill,
    ctx: &BundleContext<'_>,
) {
    let kebab_name = skill.skill_id.as_str().replace('_', "-");
    let skill_md = format!(
        "---\nname: {}\ndescription: \"{}\"\n---\n\n{}\n",
        kebab_name,
        skill.description.replace('"', "\\\""),
        skill.content.trim()
    );
    files.push(PluginFile {
        path: format!("skills/{}/SKILL.md", skill.skill_id),
        content: skill_md,
        executable: false,
    });

    if let Some(mk) = ctx.master_key {
        if let Ok(secrets) =
            resolve_secrets_for_skill(ctx.pool, ctx.user_id, &skill.skill_id, mk).await
        {
            if !secrets.is_empty() {
                let env_content: String = secrets
                    .iter()
                    .map(|(k, v)| format!("{k}={v}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                files.push(PluginFile {
                    path: format!("skills/{}/.env", skill.skill_id),
                    content: env_content,
                    executable: false,
                });
            }
        }
    }
}

pub fn build_agent_file(files: &mut Vec<PluginFile>, agent: &crate::types::UserAgent) {
    let escaped_desc = agent.description.replace('"', "\\\"");
    let agent_md = format!(
        "---\nname: {}\ndescription: \"{escaped_desc}\"\n---\n\n{}\n",
        agent.agent_id,
        agent.system_prompt.trim()
    );
    files.push(PluginFile {
        path: format!("agents/{}.md", agent.agent_id),
        content: agent_md,
        executable: false,
    });
}

pub fn build_mcp_files(
    files: &mut Vec<PluginFile>,
    mcp_server_ids: &[McpServerId],
    all_user_mcp_servers: &[crate::types::UserMcpServer],
    platform_url: &str,
    claimed_mcp_server_ids: &mut std::collections::HashSet<McpServerId>,
) {
    let mut mcp_server_entries = HashMap::new();
    for mcp_id in mcp_server_ids {
        claimed_mcp_server_ids.insert(mcp_id.clone());
        if let Some(mcp) = all_user_mcp_servers
            .iter()
            .find(|m| m.id == mcp_id.as_str())
        {
            let url = if mcp.endpoint.is_empty() {
                format!("{platform_url}/api/v1/mcp/{}/mcp", mcp.mcp_server_id)
            } else {
                mcp.endpoint.clone()
            };
            mcp_server_entries.insert(
                mcp.mcp_server_id.to_string(),
                McpServerEntry {
                    server_type: "http".to_string(),
                    url,
                },
            );
        }
    }
    if !mcp_server_entries.is_empty() {
        let mcp_config = McpConfigFile {
            mcp_servers: mcp_server_entries,
        };
        if let Ok(content) = serde_json::to_string_pretty(&mcp_config) {
            files.push(PluginFile {
                path: ".mcp.json".to_string(),
                content,
                executable: false,
            });
        }
    }
}

pub fn build_env_and_hook_files_with_token(
    files: &mut Vec<PluginFile>,
    ctx: &BundleContext<'_>,
    email: &str,
) {
    let token = ctx
        .token
        .map(String::from)
        .or_else(|| {
            crate::repositories::plugin_jwt::generate_plugin_token(ctx.user_id, email, ctx.plugin_id)
                .map_err(|e| tracing::warn!(error = %e, plugin_id = %ctx.plugin_id, "Failed to generate plugin JWT"))
                .ok()
        });

    if let Some(ref token) = token {
        files.push(PluginFile {
            path: ".env.plugin".to_string(),
            content: format!(
                "SYSTEMPROMPT_PLUGIN_TOKEN={token}\nSYSTEMPROMPT_API_URL={}\n",
                ctx.platform_url
            ),
            executable: false,
        });
        if let Err(e) =
            crate::repositories::export_builders::build_hook_files(ctx.platform_url, token, files)
        {
            tracing::warn!(error = %e, plugin_id = %ctx.plugin_id, "Failed to generate hook files");
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ManifestInfo<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub version: &'a str,
    pub username: &'a str,
    pub email: &'a str,
}

pub fn build_manifest(files: &mut Vec<PluginFile>, info: &ManifestInfo<'_>) {
    let author = if info.username.is_empty() {
        None
    } else {
        Some(ManifestAuthor {
            name: info.username.to_string(),
            email: info.email.to_string(),
        })
    };
    let manifest = PluginManifest {
        name: info.name.to_string(),
        description: info.description.to_string(),
        version: info.version.to_string(),
        author,
        hooks: Some("./hooks/hooks.json".to_string()),
        keywords: Vec::new(),
    };
    if let Ok(content) = serde_json::to_string_pretty(&manifest) {
        files.push(PluginFile {
            path: ".claude-plugin/plugin.json".to_string(),
            content,
            executable: false,
        });
    }
}
