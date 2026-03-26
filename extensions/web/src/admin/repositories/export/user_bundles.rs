use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::identifiers::{AgentId, McpServerId, SkillId, UserId};

use super::super::export_validation::{compute_bundle_counts, compute_content_version};
use super::super::user_plugins::find_plugin_with_associations;
use super::bundle_files;
use super::types::{PluginBundle, PluginFile};
use crate::admin::types::UserPlugin;

fn slugify(name: &str) -> String {
    name.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub struct UserBundleContext<'a> {
    pub pool: &'a Arc<PgPool>,
    pub user_id: &'a UserId,
    pub username: &'a str,
    pub email: &'a str,
    pub master_key: Option<&'a [u8; 32]>,
    pub platform_url: &'a str,
    pub all_user_skills: &'a [crate::admin::types::UserSkill],
    pub all_user_agents: &'a [crate::admin::types::UserAgent],
    pub all_user_mcp_servers: &'a [crate::admin::types::UserMcpServer],
}

pub async fn build_user_plugin_bundle(
    ctx: &UserBundleContext<'_>,
    user_plugin: &UserPlugin,
    claimed_skill_ids: &mut std::collections::HashSet<SkillId>,
    claimed_agent_ids: &mut std::collections::HashSet<AgentId>,
    claimed_mcp_server_ids: &mut std::collections::HashSet<McpServerId>,
) -> Option<PluginBundle> {
    if !user_plugin.enabled {
        return None;
    }
    let Ok(Some(assoc)) =
        find_plugin_with_associations(ctx.pool, ctx.user_id, &user_plugin.plugin_id).await
    else {
        return None;
    };

    let mut files = Vec::new();
    let slugified_name = slugify(&user_plugin.name);

    let token =
        super::super::plugin_jwt::generate_plugin_token(ctx.user_id, ctx.email, &slugified_name)
            .ok();

    collect_skill_files(
        ctx,
        &assoc.skill_ids,
        claimed_skill_ids,
        &mut files,
        &slugified_name,
        token.as_deref(),
    )
    .await;
    collect_agent_files(ctx, &assoc.agent_ids, claimed_agent_ids, &mut files);

    bundle_files::build_mcp_files(
        &mut files,
        &assoc.mcp_server_ids,
        ctx.all_user_mcp_servers,
        ctx.platform_url,
        claimed_mcp_server_ids,
    );

    Some(assemble_bundle_metadata(
        ctx,
        &mut files,
        &slugified_name,
        user_plugin,
        token.as_deref(),
    ))
}

async fn collect_skill_files(
    ctx: &UserBundleContext<'_>,
    skill_ids: &[SkillId],
    claimed: &mut std::collections::HashSet<SkillId>,
    files: &mut Vec<PluginFile>,
    plugin_id: &str,
    token: Option<&str>,
) {
    for skill_id in skill_ids {
        claimed.insert(skill_id.clone());
        if let Some(skill) = ctx
            .all_user_skills
            .iter()
            .find(|s| s.id == skill_id.as_str())
        {
            let bundle_ctx = bundle_files::BundleContext {
                master_key: ctx.master_key,
                pool: ctx.pool,
                user_id: ctx.user_id,
                platform_url: ctx.platform_url,
                plugin_id,
                token,
            };
            bundle_files::build_skill_files(files, skill, &bundle_ctx).await;
        }
    }
}

fn collect_agent_files(
    ctx: &UserBundleContext<'_>,
    agent_ids: &[AgentId],
    claimed: &mut std::collections::HashSet<AgentId>,
    files: &mut Vec<PluginFile>,
) {
    for agent_id in agent_ids {
        claimed.insert(agent_id.clone());
        if let Some(agent) = ctx
            .all_user_agents
            .iter()
            .find(|a| a.id == agent_id.as_str())
        {
            bundle_files::build_agent_file(files, agent);
        }
    }
}

fn assemble_bundle_metadata(
    ctx: &UserBundleContext<'_>,
    files: &mut Vec<PluginFile>,
    slugified_name: &str,
    user_plugin: &UserPlugin,
    pre_token: Option<&str>,
) -> PluginBundle {
    bundle_files::build_manifest(
        files,
        &bundle_files::ManifestInfo {
            name: slugified_name,
            description: &user_plugin.description,
            version: &user_plugin.version,
            username: ctx.username,
            email: ctx.email,
        },
    );

    let bundle_ctx = bundle_files::BundleContext {
        master_key: ctx.master_key,
        pool: ctx.pool,
        user_id: ctx.user_id,
        platform_url: ctx.platform_url,
        plugin_id: slugified_name,
        token: pre_token,
    };
    bundle_files::build_env_and_hook_files_with_token(files, &bundle_ctx, ctx.email);

    let content_version = compute_content_version(&user_plugin.version, files);
    let counts = compute_bundle_counts(files);
    PluginBundle {
        id: user_plugin.id.clone(),
        name: slugified_name.to_string(),
        description: user_plugin.description.clone(),
        version: content_version,
        counts,
        files: std::mem::take(files),
    }
}
