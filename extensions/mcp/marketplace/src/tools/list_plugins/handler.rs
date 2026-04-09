use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use systemprompt::database::DbPool;
use systemprompt::identifiers::{McpExecutionId, UserId};
use systemprompt::mcp::McpError;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::{Column, ColumnType, TableArtifact};
use systemprompt::models::execution::context::RequestContext;

use crate::tools::shared;

#[derive(Debug, Clone, Copy, Deserialize, JsonSchema)]
pub struct ListPluginsInput;

#[derive(Debug)]
pub struct ListPluginsHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for ListPluginsHandler {
    type Input = ListPluginsInput;
    type Output = TableArtifact;

    fn tool_name(&self) -> &'static str {
        "list_plugins"
    }

    fn description(&self) -> &'static str {
        "List all plugins for the authenticated user. Returns plugin metadata including \
         skills, agents, MCP servers, and onboarding configuration with interview questions \
         and data source details."
    }

    async fn handle(
        &self,
        _input: Self::Input,
        ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let pool = shared::require_pool(&self.db_pool)?;
        let user_id = UserId::new(ctx.user_id().to_string());

        let enriched_plugins =
            systemprompt_web_extension::admin::repositories::list_user_plugins_enriched(
                &pool, &user_id,
            )
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to list plugins: {e}"), None))?;

        let onboarding_configs =
            systemprompt_web_extension::admin::repositories::plugins::load_plugin_onboarding_configs();

        let columns = vec![
            Column::new("plugin_id", ColumnType::String),
            Column::new("name", ColumnType::String),
            Column::new("description", ColumnType::String),
            Column::new("version", ColumnType::String),
            Column::new("enabled", ColumnType::Boolean),
            Column::new("category", ColumnType::String),
            Column::new("keywords", ColumnType::String),
            Column::new("base_plugin_id", ColumnType::String),
            Column::new("skills", ColumnType::String),
            Column::new("agents", ColumnType::String),
            Column::new("mcp_servers", ColumnType::String),
            Column::new("onboarding", ColumnType::String),
            Column::new("created_at", ColumnType::Date),
            Column::new("updated_at", ColumnType::Date),
        ];

        let rows: Vec<serde_json::Value> = enriched_plugins
            .iter()
            .map(|ep| {
                let p = &ep.plugin;

                let skills: Vec<serde_json::Value> = ep
                    .skills
                    .iter()
                    .map(|s| serde_json::json!({ "id": s.id, "name": s.name }))
                    .collect();

                let agents: Vec<serde_json::Value> = ep
                    .agents
                    .iter()
                    .map(|a| serde_json::json!({ "id": a.id, "name": a.name }))
                    .collect();

                let mcp_servers: Vec<serde_json::Value> = ep
                    .mcp_servers
                    .iter()
                    .map(|m| serde_json::json!({ "id": m.id, "name": m.name }))
                    .collect();

                let onboarding = p
                    .base_plugin_id
                    .as_deref()
                    .and_then(|base_id| onboarding_configs.get(base_id))
                    .or_else(|| onboarding_configs.get(&p.plugin_id));

                let onboarding_value = match onboarding {
                    Some(o) => serde_json::to_string(o).map_err(|e| {
                        McpError::internal_error(
                            format!("Serialization failed for onboarding config: {e}"),
                            None,
                        )
                    })?,
                    None => String::new(),
                };

                let skills_json = serde_json::to_string(&skills).map_err(|e| {
                    McpError::internal_error(
                        format!("Serialization failed for skills: {e}"),
                        None,
                    )
                })?;
                let agents_json = serde_json::to_string(&agents).map_err(|e| {
                    McpError::internal_error(
                        format!("Serialization failed for agents: {e}"),
                        None,
                    )
                })?;
                let mcp_servers_json = serde_json::to_string(&mcp_servers).map_err(|e| {
                    McpError::internal_error(
                        format!("Serialization failed for mcp_servers: {e}"),
                        None,
                    )
                })?;

                Ok(serde_json::json!({
                    "plugin_id": p.plugin_id,
                    "name": p.name,
                    "description": p.description,
                    "version": p.version,
                    "enabled": p.enabled,
                    "category": p.category,
                    "keywords": p.keywords,
                    "base_plugin_id": p.base_plugin_id,
                    "skills": skills_json,
                    "agents": agents_json,
                    "mcp_servers": mcp_servers_json,
                    "onboarding": onboarding_value,
                    "created_at": p.created_at.to_rfc3339(),
                    "updated_at": p.updated_at.to_rfc3339(),
                }))
            })
            .collect::<Result<Vec<_>, McpError>>()?;

        let total = rows.len();
        let summary = format!("Found {total} plugin(s) for user '{user_id}'");
        let artifact = TableArtifact::new(columns, ctx).with_rows(rows);

        Ok((artifact, summary))
    }
}
