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

#[derive(Deserialize, JsonSchema)]
pub struct GetSecretsInput {
    pub plugin_id: String,
}

pub struct GetSecretsHandler {
    pub db_pool: DbPool,
}

#[async_trait]
impl McpToolHandler for GetSecretsHandler {
    type Input = GetSecretsInput;
    type Output = TextArtifact;

    fn tool_name(&self) -> &'static str {
        "get_secrets"
    }

    fn description(&self) -> &'static str {
        "Retrieve decrypted secrets for a given plugin. Returns secret names and \
         their decrypted values. Requires plugin_id. Only accessible by the authenticated user."
    }

    async fn handle(
        &self,
        input: Self::Input,
        ctx: &RequestContext,
        _exec_id: &McpExecutionId,
    ) -> Result<(Self::Output, String), McpError> {
        let pool = shared::require_pool(&self.db_pool)?;
        let user_id = UserId::new(ctx.user_id().to_string());

        let master_key =
            systemprompt_web_extension::admin::repositories::secret_crypto::load_master_key()
                .map_err(|e| {
                    McpError::internal_error(format!("Encryption not configured: {e}"), None)
                })?;

        let secrets =
            systemprompt_web_extension::admin::repositories::secret_resolve::resolve_secrets_for_plugin(
                &pool, &user_id, &input.plugin_id, &master_key,
            )
            .await
            .map_err(|e| {
                McpError::internal_error(format!("Failed to resolve secrets: {e}"), None)
            })?;

        let count = secrets.len();
        let result_json = serde_json::to_string_pretty(&serde_json::json!({
            "_display": { "type": "secrets" },
            "plugin_id": input.plugin_id,
            "secrets": secrets,
            "count": count,
        }))
        .map_err(|e| McpError::internal_error(format!("Failed to serialize result: {e}"), None))?;

        let summary = format!(
            "Retrieved {count} secret(s) for plugin '{}'",
            input.plugin_id
        );
        let content = format!("{summary}\n\n{result_json}");
        let artifact = TextArtifact::new(&result_json, ctx).with_title("Plugin Secrets");

        Ok((artifact, content))
    }
}
