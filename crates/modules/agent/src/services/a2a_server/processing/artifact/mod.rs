use self::providers::{
    AiServiceToolProvider as ServiceToolProvider, DatabaseExecutionIdLookup as DbExecutionIdLookup,
};
use crate::models::a2a::{Artifact, Part, TextPart};
use crate::services::mcp::extract_skill_id;
use anyhow::Result;
use chrono::Utc;
use std::sync::Arc;
use systemprompt_core_ai::{CallToolResult, ToolCall};
use systemprompt_core_logging::LogService;
use systemprompt_core_system::RequestContext;
use systemprompt_identifiers::{ContextId, TaskId};
use systemprompt_models::a2a::ArtifactMetadata;
use uuid::Uuid;

pub mod mocks;
pub mod providers;
pub mod traits;

pub use providers::{AiServiceToolProvider, DatabaseExecutionIdLookup};
pub use traits::{ExecutionIdLookup, ToolProvider};

#[derive(Debug)]
pub struct ArtifactBuilder {
    tool_calls: Vec<ToolCall>,
    tool_results: Vec<CallToolResult>,
    _tool_provider: Arc<ServiceToolProvider>,
    execution_lookup: Arc<DbExecutionIdLookup>,
    context_id: String,
    task_id: String,
    _request_ctx: RequestContext,
    _log: LogService,
}

impl ArtifactBuilder {
    pub fn new(
        tool_calls: Vec<ToolCall>,
        tool_results: Vec<CallToolResult>,
        tool_provider: Arc<ServiceToolProvider>,
        execution_lookup: Arc<DbExecutionIdLookup>,
        context_id: String,
        task_id: String,
        request_ctx: RequestContext,
        log: LogService,
    ) -> Self {
        Self {
            tool_calls,
            tool_results,
            _tool_provider: tool_provider,
            execution_lookup,
            context_id,
            task_id,
            _request_ctx: request_ctx,
            _log: log,
        }
    }

    pub async fn build_artifacts(&self, _agent_name: &str) -> Result<Vec<Artifact>> {
        let mut artifacts = Vec::new();

        for (index, result) in self.tool_results.iter().enumerate() {
            if let Some(structured_content) = &result.structured_content {
                if let Some(tool_call) = self.tool_calls.get(index) {
                    let mcp_execution_id = self
                        .execution_lookup
                        .get_mcp_execution_id(tool_call.ai_tool_call_id.as_str())
                        .await
                        .unwrap_or(None);

                    // Extract skill ID - repository will lookup full details from database
                    let skill_id = extract_skill_id(structured_content);

                    let mut metadata = ArtifactMetadata {
                        artifact_type: "mcp_tool_result".to_string(),
                        context_id: ContextId::new(self.context_id.clone()),
                        created_at: Utc::now().to_rfc3339(),
                        task_id: TaskId::new(self.task_id.clone()),
                        rendering_hints: None,
                        source: Some("mcp_tool".to_string()),
                        mcp_execution_id,
                        mcp_schema: None,
                        is_internal: None,
                        fingerprint: None,
                        tool_name: Some(tool_call.name.clone()),
                        execution_index: Some(index),
                        render_behavior: "both".to_string(),
                        skill_id: None,
                        skill_name: None,
                    };

                    // Add skill ID - repository will join with agent_skills table
                    if let Some(sid) = skill_id {
                        metadata = metadata.with_skill_id(sid);
                    }

                    let artifact = Artifact {
                        artifact_id: Uuid::new_v4().to_string(),
                        name: Some(tool_call.name.clone()),
                        description: None,
                        parts: vec![Part::Text(TextPart {
                            text: structured_content.to_string(),
                        })],
                        extensions: Vec::new(),
                        metadata,
                    };
                    artifacts.push(artifact);
                }
            }
        }

        Ok(artifacts)
    }
}
