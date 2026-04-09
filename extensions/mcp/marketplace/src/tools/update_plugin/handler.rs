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
#[allow(clippy::too_many_lines)]
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
        let pool = self.db_pool.write_pool().ok_or_else(|| {
            McpError::internal_error("Database pool not available".to_string(), None)
        })?;

        let user_id = UserId::new(ctx.user_id().to_string());

        // Resolve slug-based IDs to UUIDs before starting the transaction (read-only)
        let resolved_skill_ids = if let Some(ref skill_slugs) = input.skill_ids {
            Some(shared::resolve_skill_slugs(&pool, user_id.as_ref(), skill_slugs).await?)
        } else {
            None
        };
        let resolved_agent_ids = if let Some(ref agent_slugs) = input.agent_ids {
            Some(shared::resolve_agent_slugs(&pool, user_id.as_ref(), agent_slugs).await?)
        } else {
            None
        };
        let resolved_mcp_server_ids = if let Some(ref mcp_server_slugs) = input.mcp_server_ids {
            Some(
                shared::resolve_mcp_server_slugs(&pool, user_id.as_ref(), mcp_server_slugs)
                    .await?,
            )
        } else {
            None
        };

        // Start a transaction to ensure update and associations are atomic
        let mut tx = pool.begin().await.map_err(|e| {
            McpError::internal_error(format!("Failed to start transaction: {e}"), None)
        })?;

        let plugin = sqlx::query_as::<_, UserPlugin>(
            r#"UPDATE user_plugins SET
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                version = COALESCE($5, version),
                enabled = COALESCE($6, enabled),
                category = COALESCE($7, category),
                keywords = COALESCE($8, keywords),
                author_name = COALESCE($9, author_name),
                updated_at = NOW()
            WHERE user_id = $1 AND plugin_id = $2
            RETURNING id, user_id, plugin_id, name, description, version, enabled, category, COALESCE(keywords, '{}') as "keywords", author_name, base_plugin_id, created_at, updated_at"#,
        )
        .bind(user_id.as_str())
        .bind(&input.plugin_id)
        .bind(input.name.as_deref())
        .bind(input.description.as_deref())
        .bind(input.version.as_deref())
        .bind(input.enabled)
        .bind(input.category.as_deref())
        .bind(&input.keywords)
        .bind(input.author_name.as_deref())
        .fetch_optional(&mut *tx)
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

        // Set skill associations (delete + re-insert)
        if let Some(ref skill_ids) = resolved_skill_ids {
            sqlx::query("DELETE FROM user_plugin_skills WHERE user_plugin_id = $1")
                .bind(&plugin.id)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    McpError::internal_error(
                        format!("Failed to clear plugin skills: {e}"),
                        None,
                    )
                })?;
            for (i, skill_id) in skill_ids.iter().enumerate() {
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
        }

        // Set agent associations (delete + re-insert)
        if let Some(ref agent_ids) = resolved_agent_ids {
            sqlx::query("DELETE FROM user_plugin_agents WHERE user_plugin_id = $1")
                .bind(&plugin.id)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    McpError::internal_error(
                        format!("Failed to clear plugin agents: {e}"),
                        None,
                    )
                })?;
            for (i, agent_id) in agent_ids.iter().enumerate() {
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
        }

        // Set MCP server associations (delete + re-insert)
        if let Some(ref mcp_server_ids) = resolved_mcp_server_ids {
            sqlx::query("DELETE FROM user_plugin_mcp_servers WHERE user_plugin_id = $1")
                .bind(&plugin.id)
                .execute(&mut *tx)
                .await
                .map_err(|e| {
                    McpError::internal_error(
                        format!("Failed to clear plugin MCP servers: {e}"),
                        None,
                    )
                })?;
            for (i, mcp_server_id) in mcp_server_ids.iter().enumerate() {
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
        }

        tx.commit().await.map_err(|e| {
            McpError::internal_error(format!("Failed to commit transaction: {e}"), None)
        })?;

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
