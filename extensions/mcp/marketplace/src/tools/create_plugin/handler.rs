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
pub struct CreatePluginInput {
    pub name: String,
    pub description: String,
    pub category: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    pub version: Option<String>,
    pub author_name: Option<String>,
    #[serde(default)]
    pub skill_ids: Vec<String>,
    #[serde(default)]
    pub agent_ids: Vec<String>,
    #[serde(default)]
    pub mcp_server_ids: Vec<String>,
}

#[derive(Debug)]
pub struct CreatePluginHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for CreatePluginHandler {
    type Input = CreatePluginInput;
    type Output = TextArtifact;

    fn tool_name(&self) -> &'static str {
        "create_plugin"
    }

    fn description(&self) -> &'static str {
        "Create a new user plugin. Requires name and description. \
         Optionally provide category, keywords, version, author_name, and \
         association arrays (skill_ids, agent_ids, mcp_server_ids)."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let plugin_id = shared::generate_slug(&input.name);
        let pool = shared::require_write_pool(&self.db_pool)?;

        let create_req = systemprompt_web_extension::admin::types::CreateUserPluginRequest {
            plugin_id: plugin_id.clone(),
            name: input.name.clone(),
            description: input.description.clone(),
            version: input.version.unwrap_or_else(|| "1.0.0".to_string()),
            category: input.category.unwrap_or_default(),
            keywords: input.keywords.clone(),
            author_name: input.author_name.unwrap_or_default(),
            base_plugin_id: None,
        };

        let user_id = UserId::new(ctx.user_id().to_string());
        let mut tx = pool.begin().await.map_err(|e| {
            McpError::internal_error(format!("Failed to begin transaction: {e}"), None)
        })?;

        let plugin =
            systemprompt_web_extension::admin::repositories::user_plugins::create_user_plugin(
                &mut *tx,
                &user_id,
                &create_req,
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to create plugin: {e}"), None))?;

        shared::set_plugin_associations(
            &mut *tx,
            &plugin.id,
            &user_id,
            Some(&input.skill_ids),
            Some(&input.agent_ids),
            Some(&input.mcp_server_ids),
        )
        .await?;

        tx.commit().await.map_err(|e| {
            McpError::internal_error(format!("Failed to commit transaction: {e}"), None)
        })?;

        let (skill_slugs, agent_slugs, mcp_server_slugs) =
            shared::resolve_association_slugs(&pool, &user_id, &plugin.plugin_id).await?;

        shared::invalidate_marketplace_cache(&pool, &user_id).await;

        shared::build_plugin_response(
            &plugin,
            ctx,
            "created",
            &skill_slugs,
            &agent_slugs,
            &mcp_server_slugs,
        )
    }
}
