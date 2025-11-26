//! Standard Execution Strategy - No Tools
//!
//! Handles pure text generation without tool capabilities.

use anyhow::Result;
use async_trait::async_trait;
use systemprompt_core_ai::AiMessage;
use uuid::Uuid;

use super::{ExecutionContext, ExecutionResult, ExecutionStrategy};
use crate::models::a2a::{Message as A2aMessage, Part, TextPart};
use crate::services::a2a_server::processing::ai_executor::process_without_tools;

#[derive(Debug, Clone, Copy)]
pub struct StandardExecutionStrategy;

impl StandardExecutionStrategy {
    pub fn new() -> Self {
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

        let (accumulated_text, tool_calls, tool_results) = process_without_tools(
            context.ai_service.clone(),
            &context.agent_runtime,
            messages,
            context.tx.clone(),
            context.request_ctx.clone(),
        )
        .await
        .map_err(|_| anyhow::anyhow!("Standard execution failed"))?;

        let agent_message = A2aMessage {
            role: "agent".to_string(),
            parts: vec![Part::Text(TextPart {
                text: accumulated_text.clone(),
            })],
            message_id: Uuid::new_v4().to_string(),
            task_id: Some(context.task_id.clone()),
            context_id: context.context_id.clone(),
            kind: "message".to_string(),
            metadata: None,
            extensions: None,
            reference_task_ids: None,
        };

        Ok(ExecutionResult {
            accumulated_text,
            tool_calls,
            tool_results,
            iterations: 1,
            conversation_history: vec![agent_message],
        })
    }

    fn name(&self) -> &'static str {
        "standard"
    }
}
