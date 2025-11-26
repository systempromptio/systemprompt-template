//! Tool-Based Execution Strategy
//!
//! Handles single-turn tool execution. Detects if AI requests agentic mode
//! and delegates to AgenticExecutionStrategy BEFORE executing any tools.
//!
//! Key design: Check execution mode BEFORE tool execution to prevent duplicate
//! tool calls when delegating to agentic mode.

use anyhow::Result;
use async_trait::async_trait;
use systemprompt_core_ai::{AiMessage, SamplingMetadata, ToolCall, TooledRequest};
use systemprompt_identifiers::AgentName;
use uuid::Uuid;

use super::agentic::AgenticExecutionStrategy;
use super::{ExecutionContext, ExecutionResult, ExecutionStrategy};
use crate::models::a2a::{Message as A2aMessage, Part, TextPart};
use crate::services::a2a_server::processing::ai_executor::process_without_tools;
use crate::services::a2a_server::processing::message::StreamEvent;

#[derive(Debug, Clone, Copy)]
pub struct ToolExecutionStrategy;

impl ToolExecutionStrategy {
    pub fn new() -> Self {
        Self
    }

    fn wants_agentic(tool_calls: &[ToolCall]) -> bool {
        tool_calls.iter().any(|tc| {
            tc.is_meta_tool()
                && tc
                    .arguments
                    .get("mode")
                    .and_then(|m| m.as_str())
                    .map(|s| s == "agentic")
                    .unwrap_or(false)
        })
    }
}

impl Default for ToolExecutionStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExecutionStrategy for ToolExecutionStrategy {
    async fn execute(
        &self,
        context: ExecutionContext,
        messages: Vec<AiMessage>,
    ) -> Result<ExecutionResult> {
        context
            .log
            .info("tool_strategy", "Processing with tools - checking execution mode BEFORE tool execution")
            .await
            .ok();

        // Step 1: Get available tools for this agent
        let agent_name = AgentName::new(context.agent_name.as_ref());
        let tools = match context
            .ai_service
            .list_available_tools_for_agent(&agent_name, &context.request_ctx)
            .await
        {
            Ok(tools) if !tools.is_empty() => tools,
            _ => {
                // No tools available - fall back to standard execution
                context
                    .log
                    .warn("tool_strategy", "No tools available - falling back to standard execution")
                    .await
                    .ok();
                let (text, calls, results) = process_without_tools(
                    context.ai_service.clone(),
                    &context.agent_runtime,
                    messages,
                    context.tx.clone(),
                    context.request_ctx.clone(),
                )
                .await
                .map_err(|_| anyhow::anyhow!("Standard execution failed"))?;
                return Ok(ExecutionResult {
                    accumulated_text: text,
                    tool_calls: calls,
                    tool_results: results,
                    iterations: 1,
                    conversation_history: Vec::new(),
                });
            },
        };

        context
            .log
            .info(
                "tool_strategy",
                &format!("Found {} tools for agent {}", tools.len(), context.agent_name),
            )
            .await
            .ok();

        // Step 2: Build the tooled request
        let tooled_request = TooledRequest {
            provider: context.agent_runtime.provider.clone(),
            model: context.agent_runtime.model.clone(),
            messages: messages.clone(),
            tools: tools.clone(),
            metadata: Some(SamplingMetadata::default()),
            response_format: None,
            structured_output: None,
            context_id: context.request_ctx.execution.context_id.clone(),
            task_id: context
                .request_ctx
                .task_id()
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("task_id required"))?,
        };

        // Step 3: Get AI response WITHOUT executing tools
        let (response, tool_calls) = context
            .ai_service
            .sample_without_execution(tooled_request, &context.request_ctx)
            .await
            .map_err(|e| anyhow::anyhow!("AI call failed: {}", e))?;

        context
            .log
            .info(
                "tool_strategy",
                &format!(
                    "AI returned {} tool calls (checking for agentic mode BEFORE execution)",
                    tool_calls.len()
                ),
            )
            .await
            .ok();

        // Step 4: Check if AI wants agentic mode BEFORE any tool execution
        if Self::wants_agentic(&tool_calls) {
            context
                .log
                .info(
                    "tool_strategy",
                    "AI requested agentic mode - delegating BEFORE tool execution (no tools executed yet)",
                )
                .await
                .ok();

            // Delegate immediately - no tools have been executed!
            let agentic_strategy = AgenticExecutionStrategy::new();
            return agentic_strategy.execute(context, messages).await;
        }

        // Step 5: Not agentic - execute tools now
        context
            .log
            .info(
                "tool_strategy",
                &format!("Tool mode confirmed - executing {} tool calls", tool_calls.len()),
            )
            .await
            .ok();

        let (tool_calls, tool_results) = context
            .ai_service
            .execute_tools(tool_calls, &tools, &context.request_ctx)
            .await;

        // Step 6: Send stream events for tool execution
        for call in &tool_calls {
            context.tx.send(StreamEvent::ToolCallStarted(call.clone())).ok();
        }
        for (idx, result) in tool_results.iter().enumerate() {
            let call_id = tool_calls
                .get(idx)
                .map(|c| c.ai_tool_call_id.as_ref().to_string())
                .unwrap_or_else(|| format!("unknown_{}", idx));
            context
                .tx
                .send(StreamEvent::ToolResult {
                    call_id,
                    result: result.clone(),
                })
                .ok();
        }

        // Step 7: Stream the AI response text if no executable tools or no results
        let executable_calls: Vec<_> = tool_calls.iter().filter(|tc| tc.is_executable()).cloned().collect();
        if executable_calls.is_empty() || tool_results.is_empty() {
            // Stream the response content in chunks
            let mut chunk = String::new();
            for c in response.content.chars() {
                chunk.push(c);
                if chunk.len() >= 20 {
                    context.tx.send(StreamEvent::Text(chunk.clone())).ok();
                    chunk.clear();
                }
            }
            if !chunk.is_empty() {
                context.tx.send(StreamEvent::Text(chunk)).ok();
            }
        }

        // Step 8: Build conversation history
        let agent_message = A2aMessage {
            role: "agent".to_string(),
            parts: vec![Part::Text(TextPart {
                text: response.content.clone(),
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
            accumulated_text: response.content,
            tool_calls,
            tool_results,
            iterations: 1,
            conversation_history: vec![agent_message],
        })
    }

    fn name(&self) -> &'static str {
        "tool_based"
    }
}
