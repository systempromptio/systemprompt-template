//! Standard Execution Strategy - No Tools
//!
//! Handles pure text generation without tool capabilities.

use anyhow::Result;
use async_trait::async_trait;
use systemprompt_core_ai::AiMessage;
use systemprompt_identifiers::TaskId;

use super::{ExecutionContext, ExecutionResult, ExecutionStrategy};
use crate::services::a2a_server::processing::ai_executor::process_without_tools;
use crate::services::ExecutionTrackingService;

#[derive(Debug, Clone, Copy)]
pub struct StandardExecutionStrategy;

impl StandardExecutionStrategy {
    pub const fn new() -> Self {
        Self
    }
}

impl Default for StandardExecutionStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExecutionStrategy for StandardExecutionStrategy {
    async fn execute(
        &self,
        context: ExecutionContext,
        messages: Vec<AiMessage>,
    ) -> Result<ExecutionResult> {
        context
            .log
            .info("standard_strategy", "Processing without tools")
            .await
            .ok();

        let tracking = ExecutionTrackingService::new(context.execution_step_repo.clone());
        let task_id = TaskId::new(context.task_id.as_str());

        tracking.track_understanding(task_id.clone()).await.ok();

        let (accumulated_text, tool_calls, tool_results) = process_without_tools(
            context.ai_service.clone(),
            &context.agent_runtime,
            messages,
            context.tx.clone(),
            context.request_ctx.clone(),
        )
        .await
        .map_err(|_| anyhow::anyhow!("Standard execution failed"))?;

        tracking.track_completion(task_id).await.ok();

        Ok(ExecutionResult {
            accumulated_text,
            tool_calls,
            tool_results,
            iterations: 1,
        })
    }

    fn name(&self) -> &'static str {
        "standard"
    }
}
