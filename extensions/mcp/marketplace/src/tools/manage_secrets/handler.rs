use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpError;
use systemprompt::mcp::McpToolHandler;
use systemprompt::models::artifacts::TextArtifact;
use systemprompt::models::execution::context::RequestContext;

use systemprompt::identifiers::UserId;

use crate::tools::shared;

#[derive(Deserialize, JsonSchema)]
pub struct ManageSecretsInput {
    pub action: String,
    pub plugin_id: String,
    pub var_name: Option<String>,
    pub var_value: Option<String>,
    #[serde(default)]
    pub is_secret: bool,
}

pub struct ManageSecretsHandler {
    pub db_pool: DbPool,
}

struct SecretContext {
    pool: Arc<systemprompt::database::PgPool>,
    user_id: UserId,
    plugin_id: String,
}

#[async_trait]
impl McpToolHandler for ManageSecretsHandler {
    type Input = ManageSecretsInput;
    type Output = TextArtifact;

    fn tool_name(&self) -> &'static str {
        "manage_secrets"
    }

    fn description(&self) -> &'static str {
        "Manage plugin environment variables and secrets. Supports list, set, and \
         delete actions. Secrets are encrypted at rest. Requires action and plugin_id."
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

        let secret_ctx = SecretContext {
            pool,
            user_id: UserId::new(ctx.user_id().to_string()),
            plugin_id: input.plugin_id.clone(),
        };

        match input.action.as_str() {
            "list" => handle_list(&secret_ctx, ctx).await,
            "set" => {
                let var_name = input.var_name.as_deref().ok_or_else(|| {
                    McpError::invalid_params(
                        "var_name is required for set action".to_string(),
                        None,
                    )
                })?;
                let var_value = input.var_value.as_deref().ok_or_else(|| {
                    McpError::invalid_params(
                        "var_value is required for set action".to_string(),
                        None,
                    )
                })?;
                handle_set(&secret_ctx, var_name, var_value, input.is_secret, ctx).await
            }
            "delete" => {
                let var_name = input.var_name.as_deref().ok_or_else(|| {
                    McpError::invalid_params(
                        "var_name is required for delete action".to_string(),
                        None,
                    )
                })?;
                handle_delete(&secret_ctx, var_name, ctx).await
            }
            _ => Err(McpError::invalid_params(
                format!(
                    "Invalid action: '{}'. Must be one of: list, set, delete",
                    input.action
                ),
                None,
            )),
        }
    }
}

async fn handle_list(
    secret_ctx: &SecretContext,
    ctx: &RequestContext,
) -> Result<(TextArtifact, String), McpError> {
    let vars = systemprompt_web_extension::admin::repositories::plugin_env::list_plugin_env_vars(
        &secret_ctx.pool,
        &secret_ctx.user_id,
        &secret_ctx.plugin_id,
    )
    .await
    .map_err(|e| McpError::internal_error(format!("Failed to list env vars: {e}"), None))?;

    let total = vars.len();
    let var_list: Vec<serde_json::Value> = vars
        .iter()
        .map(|v| {
            serde_json::json!({
                "id": v.id,
                "plugin_id": v.plugin_id,
                "var_name": v.var_name,
                "var_value": v.var_value,
                "is_secret": v.is_secret,
            })
        })
        .collect();

    let result_json = serde_json::to_string_pretty(&serde_json::json!({
        "_display": { "type": "secrets" },
        "action": "list",
        "plugin_id": secret_ctx.plugin_id,
        "variables": var_list,
        "total": total,
    }))
    .unwrap_or_else(|_| String::new());

    let summary = format!(
        "Found {total} environment variable(s) for plugin '{}'",
        secret_ctx.plugin_id
    );
    let content = format!("{summary}\n\n{result_json}");
    let artifact = TextArtifact::new(&result_json, ctx).with_title("Plugin Environment Variables");

    Ok((artifact, content))
}

async fn handle_set(
    secret_ctx: &SecretContext,
    var_name: &str,
    var_value: &str,
    is_secret: bool,
    ctx: &RequestContext,
) -> Result<(TextArtifact, String), McpError> {
    systemprompt_web_extension::admin::repositories::plugin_env::upsert_plugin_env_var(
        &secret_ctx.pool,
        &secret_ctx.user_id,
        &secret_ctx.plugin_id,
        var_name,
        var_value,
        is_secret,
    )
    .await
    .map_err(|e| McpError::internal_error(format!("Failed to set env var: {e}"), None))?;

    shared::invalidate_marketplace_cache(&secret_ctx.pool, &secret_ctx.user_id).await;

    let result_json = serde_json::to_string_pretty(&serde_json::json!({
        "_display": { "type": "confirmation", "action": "set" },
        "action": "set",
        "plugin_id": secret_ctx.plugin_id,
        "var_name": var_name,
        "is_secret": is_secret,
        "success": true,
    }))
    .unwrap_or_else(|_| String::new());

    let summary = format!(
        "Set environment variable '{var_name}' for plugin '{}'",
        secret_ctx.plugin_id
    );
    let content = format!("{summary}\n\n{result_json}");
    let artifact = TextArtifact::new(&result_json, ctx).with_title("Set Environment Variable");

    Ok((artifact, content))
}

async fn handle_delete(
    secret_ctx: &SecretContext,
    var_name: &str,
    ctx: &RequestContext,
) -> Result<(TextArtifact, String), McpError> {
    let deleted =
        systemprompt_web_extension::admin::repositories::plugin_env::delete_plugin_env_var(
            &secret_ctx.pool,
            &secret_ctx.user_id,
            &secret_ctx.plugin_id,
            var_name,
        )
        .await
        .map_err(|e| McpError::internal_error(format!("Failed to delete env var: {e}"), None))?;

    shared::invalidate_marketplace_cache(&secret_ctx.pool, &secret_ctx.user_id).await;

    let result_json = serde_json::to_string_pretty(&serde_json::json!({
        "_display": { "type": "confirmation", "action": "deleted" },
        "action": "delete",
        "plugin_id": secret_ctx.plugin_id,
        "var_name": var_name,
        "deleted": deleted,
        "success": true,
    }))
    .unwrap_or_else(|_| String::new());

    let summary = if deleted {
        format!(
            "Deleted environment variable '{var_name}' from plugin '{}'",
            secret_ctx.plugin_id
        )
    } else {
        format!(
            "Variable '{var_name}' not found for plugin '{}'",
            secret_ctx.plugin_id
        )
    };
    let content = format!("{summary}\n\n{result_json}");
    let artifact = TextArtifact::new(&result_json, ctx).with_title("Delete Environment Variable");

    Ok((artifact, content))
}
