use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{McpExecutionId, UserId};
use systemprompt::mcp::McpError;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::TextArtifact;
use systemprompt::models::execution::context::RequestContext;
use systemprompt_web_extension::admin::types::UserPlugin;

use crate::tools::shared;

#[derive(Deserialize, JsonSchema)]
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

pub struct CreatePluginHandler {
    pub db_pool: DbPool,
}

#[async_trait]
#[allow(clippy::too_many_lines)]
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

        let pool = self.db_pool.write_pool().ok_or_else(|| {
            McpError::internal_error("Database pool not available".to_string(), None)
        })?;

        let user_id = UserId::new(ctx.user_id().to_string());

        // Resolve slug-based IDs to UUIDs before starting the transaction (read-only)
        let resolved_skill_ids = if !input.skill_ids.is_empty() {
            shared::resolve_skill_slugs(&pool, user_id.as_ref(), &input.skill_ids).await?
        } else {
            vec![]
        };
        let resolved_agent_ids = if !input.agent_ids.is_empty() {
            shared::resolve_agent_slugs(&pool, user_id.as_ref(), &input.agent_ids).await?
        } else {
            vec![]
        };
        let resolved_mcp_server_ids = if !input.mcp_server_ids.is_empty() {
            shared::resolve_mcp_server_slugs(&pool, user_id.as_ref(), &input.mcp_server_ids)
                .await?
        } else {
            vec![]
        };

        let version = input.version.unwrap_or_else(|| "1.0.0".to_string());
        let category = input.category.unwrap_or_default();
        let author_name = input.author_name.unwrap_or_default();

        if plugin_id == "systemprompt" {
            return Err(McpError::invalid_params(
                "The plugin_id 'systemprompt' is reserved for the platform plugin".to_string(),
                None,
            ));
        }

        // Start a transaction to ensure plugin creation and associations are atomic
        let mut tx = pool.begin().await.map_err(|e| {
            McpError::internal_error(format!("Failed to start transaction: {e}"), None)
        })?;

        let id = uuid::Uuid::new_v4().to_string();
        let plugin = sqlx::query_as::<_, UserPlugin>(
            r#"INSERT INTO user_plugins (id, user_id, plugin_id, name, description, version, category, keywords, author_name, base_plugin_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, user_id, plugin_id, name, description, version, enabled, category, COALESCE(keywords, '{}') as "keywords", author_name, base_plugin_id, created_at, updated_at"#,
        )
        .bind(&id)
        .bind(user_id.as_str())
        .bind(&plugin_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&version)
        .bind(&category)
        .bind(&input.keywords)
        .bind(&author_name)
        .bind(None::<&str>)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| McpError::internal_error(format!("Failed to create plugin: {e}"), None))?;

        // Set skill associations
        for (i, skill_id) in resolved_skill_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO user_plugin_skills (user_plugin_id, user_skill_id, sort_order) VALUES ($1, $2, $3)",
            )
            .bind(&plugin.id)
            .bind(skill_id.as_str())
            .bind(i32::try_from(i).unwrap_or(0))
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to set plugin skill: {e}"), None)
            })?;
        }

        // Set agent associations
        for (i, agent_id) in resolved_agent_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO user_plugin_agents (user_plugin_id, user_agent_id, sort_order) VALUES ($1, $2, $3)",
            )
            .bind(&plugin.id)
            .bind(agent_id.as_str())
            .bind(i32::try_from(i).unwrap_or(0))
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to set plugin agent: {e}"), None)
            })?;
        }

        // Set MCP server associations
        for (i, mcp_server_id) in resolved_mcp_server_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO user_plugin_mcp_servers (user_plugin_id, user_mcp_server_id, sort_order) VALUES ($1, $2, $3)",
            )
            .bind(&plugin.id)
            .bind(mcp_server_id.as_str())
            .bind(i32::try_from(i).unwrap_or(0))
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                McpError::internal_error(
                    format!("Failed to set plugin MCP server: {e}"),
                    None,
                )
            })?;
        }

        tx.commit().await.map_err(|e| {
            McpError::internal_error(format!("Failed to commit transaction: {e}"), None)
        })?;

        let (skill_slugs, agent_slugs, mcp_server_slugs) =
            shared::resolve_association_slugs(&pool, &user_id, &plugin.plugin_id).await;

        shared::invalidate_marketplace_cache(&pool, &user_id).await;

        let plugin_json = serde_json::to_string_pretty(&serde_json::json!({
            "_display": { "type": "card", "entity": "plugin", "action": "created" },
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

        let summary = format!("Created plugin '{}' ({})", plugin.name, plugin.plugin_id);
        let content = format!("{summary}\n\n{plugin_json}");
        let artifact =
            TextArtifact::new(&plugin_json, ctx).with_title(format!("Plugin: {}", plugin.name));

        Ok((artifact, content))
    }
}
