use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{McpExecutionId, UserId};
use systemprompt::mcp::McpError;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::TextArtifact;
use systemprompt::models::execution::context::RequestContext;

use crate::tools::shared;

const MAX_NAME_LEN: usize = 256;
const MAX_DESCRIPTION_LEN: usize = 4096;
const MAX_VERSION_LEN: usize = 64;
const MAX_CATEGORY_LEN: usize = 128;
const MAX_TAG_LEN: usize = 128;
const MAX_TAGS_COUNT: usize = 50;

#[derive(Deserialize, JsonSchema)]
pub struct UpdatePluginInput {
    pub plugin_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub enabled: Option<bool>,
    pub category: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub author_name: Option<String>,
    pub skill_ids: Option<Vec<String>>,
    pub agent_ids: Option<Vec<String>>,
    pub mcp_server_ids: Option<Vec<String>>,
}

pub struct UpdatePluginHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for UpdatePluginHandler {
    type Input = UpdatePluginInput;
    type Output = TextArtifact;

    fn tool_name(&self) -> &'static str {
        "update_plugin"
    }

    fn description(&self) -> &'static str {
        "Update an existing user plugin. Requires plugin_id. All other fields \
         are optional. Providing skill_ids, agent_ids, or mcp_server_ids replaces \
         existing associations."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        if let Some(ref name) = input.name {
            if name.len() > MAX_NAME_LEN {
                return Err(McpError::invalid_params(
                    format!("name exceeds maximum length of {MAX_NAME_LEN}"),
                    None,
                ));
            }
        }
        if let Some(ref description) = input.description {
            if description.len() > MAX_DESCRIPTION_LEN {
                return Err(McpError::invalid_params(
                    format!("description exceeds maximum length of {MAX_DESCRIPTION_LEN}"),
                    None,
                ));
            }
        }
        if let Some(ref version) = input.version {
            if version.len() > MAX_VERSION_LEN {
                return Err(McpError::invalid_params(
                    format!("version exceeds maximum length of {MAX_VERSION_LEN}"),
                    None,
                ));
            }
        }
        if let Some(ref category) = input.category {
            if category.len() > MAX_CATEGORY_LEN {
                return Err(McpError::invalid_params(
                    format!("category exceeds maximum length of {MAX_CATEGORY_LEN}"),
                    None,
                ));
            }
        }
        if let Some(ref author_name) = input.author_name {
            if author_name.len() > MAX_NAME_LEN {
                return Err(McpError::invalid_params(
                    format!("author_name exceeds maximum length of {MAX_NAME_LEN}"),
                    None,
                ));
            }
        }
        if let Some(ref keywords) = input.keywords {
            if keywords.len() > MAX_TAGS_COUNT {
                return Err(McpError::invalid_params(
                    format!("keywords count exceeds maximum of {MAX_TAGS_COUNT}"),
                    None,
                ));
            }
            for keyword in keywords {
                if keyword.len() > MAX_TAG_LEN {
                    return Err(McpError::invalid_params(
                        format!("keyword exceeds maximum length of {MAX_TAG_LEN}"),
                        None,
                    ));
                }
            }
        }

        let pool = self.db_pool.write_pool().ok_or_else(|| {
            McpError::internal_error("Database pool not available".to_string(), None)
        })?;

        let update_req = systemprompt_web_extension::admin::types::UpdateUserPluginRequest {
            name: input.name,
            description: input.description,
            version: input.version,
            enabled: input.enabled,
            category: input.category,
            keywords: input.keywords,
            author_name: input.author_name,
        };

        let user_id = UserId::new(ctx.user_id().to_string());
        let plugin =
            systemprompt_web_extension::admin::repositories::user_plugins::update_user_plugin(
                &pool,
                &user_id,
                &input.plugin_id,
                &update_req,
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to update plugin: {e}"), None))?
            .ok_or_else(|| {
                McpError::invalid_params(
                    format!(
                        "Plugin '{}' not found or does not belong to you",
                        input.plugin_id
                    ),
                    None,
                )
            })?;

        if let Some(ref skill_slugs) = input.skill_ids {
            let uuids = shared::resolve_skill_slugs(&pool, user_id.as_ref(), skill_slugs).await?;
            let typed: Vec<systemprompt::identifiers::SkillId> = uuids
                .into_iter()
                .map(systemprompt::identifiers::SkillId::new)
                .collect();
            systemprompt_web_extension::admin::repositories::set_plugin_skills(
                &pool, &plugin.id, &typed,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to set plugin skills: {e}"), None)
            })?;
        }
        if let Some(ref agent_slugs) = input.agent_ids {
            let uuids = shared::resolve_agent_slugs(&pool, user_id.as_ref(), agent_slugs).await?;
            let typed: Vec<systemprompt::identifiers::AgentId> = uuids
                .into_iter()
                .map(systemprompt::identifiers::AgentId::new)
                .collect();
            systemprompt_web_extension::admin::repositories::set_plugin_agents(
                &pool, &plugin.id, &typed,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to set plugin agents: {e}"), None)
            })?;
        }
        if let Some(ref mcp_server_slugs) = input.mcp_server_ids {
            let uuids =
                shared::resolve_mcp_server_slugs(&pool, user_id.as_ref(), mcp_server_slugs).await?;
            let typed: Vec<systemprompt::identifiers::McpServerId> = uuids
                .into_iter()
                .map(systemprompt::identifiers::McpServerId::new)
                .collect();
            systemprompt_web_extension::admin::repositories::set_plugin_mcp_servers(
                &pool, &plugin.id, &typed,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to set plugin MCP servers: {e}"), None)
            })?;
        }

        let (skill_slugs, agent_slugs, mcp_server_slugs) =
            shared::resolve_association_slugs(&pool, &user_id, &input.plugin_id).await;

        shared::invalidate_marketplace_cache(&pool, &user_id).await;

        let plugin_json = serde_json::to_string_pretty(&serde_json::json!({
            "_display": { "type": "card", "entity": "plugin", "action": "updated" },
            "plugin_id": plugin.plugin_id,
            "name": plugin.name,
            "description": plugin.description,
            "version": plugin.version,
            "enabled": plugin.enabled,
            "category": plugin.category,
            "keywords": plugin.keywords,
            "author_name": plugin.author_name,
            "skill_ids": skill_slugs,
            "agent_ids": agent_slugs,
            "mcp_server_ids": mcp_server_slugs,
            "created_at": plugin.created_at.to_rfc3339(),
            "updated_at": plugin.updated_at.to_rfc3339(),
        }))
        .map_err(|e| McpError::internal_error(format!("Failed to serialize plugin: {e}"), None))?;

        let summary = format!("Updated plugin '{}' ({})", plugin.name, plugin.plugin_id);
        let content = format!("{summary}\n\n{plugin_json}");
        let artifact =
            TextArtifact::new(&plugin_json, ctx).with_title(format!("Plugin: {}", plugin.name));

        Ok((artifact, content))
    }
}
