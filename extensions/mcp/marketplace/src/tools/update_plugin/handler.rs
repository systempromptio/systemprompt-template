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

#[derive(Debug, Deserialize, JsonSchema)]
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

#[derive(Debug)]
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
        let pool = shared::require_write_pool(&self.db_pool)?;

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

        shared::set_plugin_associations(
            &pool,
            &plugin.id,
            &user_id,
            input.skill_ids.as_deref(),
            input.agent_ids.as_deref(),
            input.mcp_server_ids.as_deref(),
        )
        .await?;

        let (skill_slugs, agent_slugs, mcp_server_slugs) =
            shared::resolve_association_slugs(&pool, &user_id, &input.plugin_id).await?;

        shared::invalidate_marketplace_cache(&pool, &user_id).await;

        shared::build_plugin_response(
            &plugin,
            ctx,
            "updated",
            &skill_slugs,
            &agent_slugs,
            &mcp_server_slugs,
        )
    }
}
