//! Agentic Execution Strategy
//!
//! Multi-turn tool execution with Task.history persistence and synthesis-driven
//! continuation decisions.

use anyhow::Result;
use async_trait::async_trait;
use systemprompt_core_ai::{AiMessage, CallToolResult, ToolCall, ToolResultFormatter};
use uuid::Uuid;

use super::{ExecutionContext, ExecutionResult, ExecutionStrategy};
use crate::models::a2a::{Message as A2aMessage, Part, TextPart};
use crate::services::a2a_server::processing::ai_executor::process_with_agentic_tools;

#[derive(Debug, Clone, Copy)]
pub struct AgenticExecutionStrategy;

impl AgenticExecutionStrategy {
    pub fn new() -> Self {
        Self
    }

    fn build_conversation_history(
        execution_context: &ExecutionContext,
        accumulated_text: &str,
        tool_calls: &[ToolCall],
        tool_results: &[CallToolResult],
    ) -> Vec<A2aMessage> {
        let mut messages = Vec::new();

        for (call, result) in tool_calls.iter().zip(tool_results.iter()) {
            let result_text = ToolResultFormatter::format_single_for_ai(call, result);
            messages.push(A2aMessage {
                role: "agent".to_string(),
                parts: vec![Part::Text(TextPart { text: result_text })],
                message_id: Uuid::new_v4().to_string(),
                task_id: Some(execution_context.task_id.clone()),
                context_id: execution_context.context_id.clone(),
                kind: "message".to_string(),
                metadata: Some(serde_json::json!({
                    "type": "tool_result",
                    "tool_name": call.name,
                    "tool_call_id": call.ai_tool_call_id.as_ref(),
                })),
                extensions: None,
                reference_task_ids: None,
            });
        }

        if !accumulated_text.is_empty() {
            messages.push(A2aMessage {
                role: "agent".to_string(),
                parts: vec![Part::Text(TextPart {
                    text: accumulated_text.to_string(),
                })],
                message_id: Uuid::new_v4().to_string(),
                task_id: Some(execution_context.task_id.clone()),
                context_id: execution_context.context_id.clone(),
                kind: "message".to_string(),
                metadata: None,
                extensions: None,
                reference_task_ids: None,
            });
        }

        messages
    }
}

impl Default for AgenticExecutionStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExecutionStrategy for AgenticExecutionStrategy {
    async fn execute(
        &self,
        context: ExecutionContext,
        messages: Vec<AiMessage>,
    ) -> Result<ExecutionResult> {
        context
            .log
            .info(
                "agentic_strategy",
                "Processing with agentic multi-turn execution",
            )
            .await
            .ok();

        let (accumulated_text, tool_calls, tool_results, iterations) = process_with_agentic_tools(
            context.ai_service.clone(),
            &context.agent_name,
            &context.agent_runtime,
            messages,
            context.tx.clone(),
            context.log.clone(),
            context.request_ctx.clone(),
            context.skill_service.clone(),
        )
        .await
        .map_err(|_| anyhow::anyhow!("Agentic execution failed"))?;

        context
            .log
            .info(
                "agentic_strategy",
                &format!(
                    "Agentic execution completed: {} iterations, {} tool calls",
                    iterations,
                    tool_calls.len()
                ),
            )
            .await
            .ok();

        let conversation_history = Self::build_conversation_history(
            &context,
            &accumulated_text,
            &tool_calls,
            &tool_results,
        );

        Ok(ExecutionResult {
            accumulated_text,
            tool_calls,
            tool_results,
            iterations,
            conversation_history,
        })
    }

    fn name(&self) -> &'static str {
        "agentic"
    }
}
