use anyhow::{anyhow, Result};
use std::fmt;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_identifiers::{ContextId, TaskId};
use systemprompt_models::ai::tools::CallToolResult;

use super::artifact_transformer::McpToA2aTransformer;

pub struct ToolResultHandler {
    #[allow(dead_code)]
    db_pool: DbPool,
    log: LogService,
}

impl fmt::Debug for ToolResultHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ToolResultHandler").finish_non_exhaustive()
    }
}

impl ToolResultHandler {
    pub fn new(db_pool: DbPool, log: LogService) -> Self {
        Self { db_pool, log }
    }

    /// Transform MCP tool result to A2A artifact (does NOT persist)
    ///
    /// SECURITY: context must come from validated JWT (enforced by middleware)
    /// GUARANTEE: task_id must exist (enforced by middleware)
    ///
    /// NOTE: This method only transforms the result. Persistence and broadcasting
    /// are handled by ArtifactPublishingService.
    pub async fn process_tool_result(
        &self,
        tool_name: &str,
        tool_result: &CallToolResult,
        output_schema: Option<&serde_json::Value>,
        tool_arguments: Option<&serde_json::Value>,
        task_id: &TaskId,
        context_id: &ContextId,
        context: &systemprompt_core_system::RequestContext,
    ) -> Result<crate::models::a2a::Artifact> {
        if !context.is_authenticated() || context.is_system() {
            return Err(anyhow!(
                "Invalid user - unauthenticated and system users cannot create artifacts"
            ));
        }

        self.log
            .info(
                "tool_result_handler",
                &format!(
                    "Transforming tool result to artifact: tool={}, task={}, user={}, context={}",
                    tool_name,
                    task_id,
                    context.user_id(),
                    context_id
                ),
            )
            .await
            .ok();

        let artifact = McpToA2aTransformer::transform(
            tool_name,
            tool_result,
            output_schema,
            context_id.as_str(),
            task_id.as_str(),
            tool_arguments,
        );

        use systemprompt_traits::validation::Validate;
        artifact.metadata.validate_or_panic();

        self.log
            .info(
                "tool_result_handler",
                &format!(
                    "✅ Transformed tool result to artifact {} (tool: {}, user: {}, task: {}, fingerprint: {:?})",
                    artifact.artifact_id, tool_name, context.user_id(), task_id, artifact.metadata.fingerprint
                ),
            )
            .await
            .ok();

        Ok(artifact)
    }
}
