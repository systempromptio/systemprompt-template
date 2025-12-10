//! Planned Execution Strategy
//!
//! Three-stage execution: PLAN → VALIDATE → EXECUTE → RESPOND
//!
//! 1. PLAN: AI creates tool calls or direct response (1 AI call)
//! 2. VALIDATE: Check template references are valid (no AI call)
//! 3. EXECUTE: Sequential tool execution with template resolution (0 AI calls)
//! 4. RESPOND: AI generates response with full context (1 AI call)

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use systemprompt_core_ai::{AiMessage, AiRequest, McpTool};
use systemprompt_core_system::RequestContext;
use systemprompt_identifiers::TaskId;
use systemprompt_models::ai::{PlanningResult, TemplateValidator};
use systemprompt_models::PlannedTool;

use super::plan_executor::{
    convert_to_call_tool_results, convert_to_tool_calls, execute_tools_with_templates,
    format_results_for_response, ToolExecutorTrait,
};
use super::{ExecutionContext, ExecutionResult, ExecutionStrategy};
use crate::services::a2a_server::processing::message::StreamEvent;
use crate::services::ExecutionTrackingService;

#[derive(Debug, Clone, Copy)]
pub struct PlannedAgenticStrategy;

impl PlannedAgenticStrategy {
    pub const fn new() -> Self {
        Self
    }
}

impl Default for PlannedAgenticStrategy {
    fn default() -> Self {
        Self::new()
    }
}

struct ContextToolExecutor {
    context: ExecutionContext,
}

#[async_trait]
impl ToolExecutorTrait for ContextToolExecutor {
    async fn execute_tool(
        &self,
        tool_name: &str,
        arguments: Value,
        tools: &[McpTool],
        ctx: &RequestContext,
    ) -> Result<Value> {
        let tool_call = systemprompt_core_ai::ToolCall {
            ai_tool_call_id: systemprompt_identifiers::AiToolCallId::new(format!(
                "call_{}",
                tool_name
            )),
            name: tool_name.to_string(),
            arguments,
        };

        let (_, results) = self
            .context
            .ai_service
            .execute_tools(
                vec![tool_call],
                tools,
                ctx,
                Some(&self.context.agent_runtime.tool_model_overrides),
            )
            .await;

        results
            .into_iter()
            .next()
            .and_then(|r| {
                if r.is_error.unwrap_or(false) {
                    return None;
                }
                r.content.into_iter().next().and_then(|c| {
                    if let rmcp::model::RawContent::Text(text_content) = c.raw {
                        let text = text_content.text;
                        serde_json::from_str(&text)
                            .ok()
                            .or(Some(Value::String(text)))
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| anyhow::anyhow!("Tool {} returned no result", tool_name))
    }
}

#[async_trait]
impl ExecutionStrategy for PlannedAgenticStrategy {
    async fn execute(
        &self,
        context: ExecutionContext,
        messages: Vec<AiMessage>,
    ) -> Result<ExecutionResult> {
        let tracking = ExecutionTrackingService::new(context.execution_step_repo.clone());
        let task_id = TaskId::new(context.task_id.as_str());

        context
            .log
            .info("planned", "Starting PLAN → EXECUTE → RESPOND flow")
            .await
            .ok();

        if let Ok(step) = tracking.track_understanding(task_id.clone()).await {
            context
                .tx
                .send(StreamEvent::ExecutionStepUpdate { step })
                .ok();
        }

        let tools = context
            .ai_service
            .list_available_tools_for_agent(&context.agent_name, &context.request_ctx)
            .await
            .unwrap_or_default();

        context
            .log
            .info("planned", &format!("Available tools: {}", tools.len()))
            .await
            .ok();

        // Track planning step (async so we can mark it completed/failed)
        let planning_tracked = tracking
            .track_planning_async(task_id.clone(), None, None)
            .await;

        if let Ok((_, ref step)) = planning_tracked {
            context
                .tx
                .send(StreamEvent::ExecutionStepUpdate { step: step.clone() })
                .ok();
        }

        let mut request = AiRequest::new(messages.clone());
        if let Some(provider) = &context.agent_runtime.provider {
            request = request.with_provider(provider.clone());
        }
        if let Some(model) = &context.agent_runtime.model {
            request = request.with_model(model.clone());
        }

        let planning_result = context
            .ai_service
            .generate_plan(request.clone(), &tools, &context.request_ctx)
            .await;

        let planning_result = match planning_result {
            Ok(result) => result,
            Err(e) => {
                if let Ok((tracked, _)) = planning_tracked {
                    tracking.fail(&tracked, e.to_string()).await.ok();
                }
                return Err(e);
            },
        };

        match planning_result {
            PlanningResult::DirectResponse { content } => {
                if let Ok((tracked, _)) = planning_tracked {
                    if let Ok(step) = tracking
                        .complete_planning(
                            tracked,
                            Some("Direct response - no tools needed".to_string()),
                            None,
                        )
                        .await
                    {
                        context
                            .tx
                            .send(StreamEvent::ExecutionStepUpdate { step })
                            .ok();
                    }
                }

                context
                    .log
                    .info("planned", "Direct response (no tools needed)")
                    .await
                    .ok();

                if let Ok(step) = tracking.track_completion(task_id).await {
                    context
                        .tx
                        .send(StreamEvent::ExecutionStepUpdate { step })
                        .ok();
                }

                context.tx.send(StreamEvent::Text(content.clone())).ok();

                Ok(ExecutionResult {
                    accumulated_text: content,
                    tool_calls: vec![],
                    tool_results: vec![],
                    iterations: 1,
                })
            },

            PlanningResult::ToolCalls { reasoning, calls } => {
                context
                    .log
                    .info(
                        "planned",
                        &format!(
                            "Tool calls planned: {} tools | Reasoning: {}",
                            calls.len(),
                            reasoning
                        ),
                    )
                    .await
                    .ok();

                let planned_tools: Vec<PlannedTool> = calls
                    .iter()
                    .map(|c| PlannedTool {
                        tool_name: c.tool_name.clone(),
                        arguments: c.arguments.clone(),
                    })
                    .collect();

                if let Ok((tracked, _)) = planning_tracked {
                    if let Ok(step) = tracking
                        .complete_planning(tracked, Some(reasoning.clone()), Some(planned_tools))
                        .await
                    {
                        context
                            .tx
                            .send(StreamEvent::ExecutionStepUpdate { step })
                            .ok();
                    }
                }

                let tool_output_schemas =
                    systemprompt_core_ai::services::execution_control::get_tool_output_schemas(
                        &calls, &tools,
                    );

                if let Err(validation_errors) =
                    TemplateValidator::validate_plan(&calls, &tool_output_schemas)
                {
                    let error_messages: Vec<String> =
                        validation_errors.iter().map(|e| e.to_string()).collect();

                    context
                        .log
                        .error(
                            "planned",
                            &format!("Template validation failed: {}", error_messages.join("; ")),
                        )
                        .await
                        .ok();

                    let validation_summary = format!(
                        "Plan validation failed:\n{}",
                        error_messages
                            .iter()
                            .map(|e| format!("- {e}"))
                            .collect::<Vec<_>>()
                            .join("\n")
                    );

                    let response = context
                        .ai_service
                        .generate_response_with_model(
                            messages,
                            &validation_summary,
                            &context.request_ctx,
                            context.agent_runtime.provider.as_deref(),
                            context.agent_runtime.model.as_deref(),
                        )
                        .await?;

                    context.tx.send(StreamEvent::Text(response.clone())).ok();

                    return Ok(ExecutionResult {
                        accumulated_text: response,
                        tool_calls: vec![],
                        tool_results: vec![],
                        iterations: 1,
                    });
                }

                context
                    .log
                    .info("planned", "Template validation passed")
                    .await
                    .ok();

                let (tool_name, tool_arguments) = if calls.len() == 1 {
                    (calls[0].tool_name.clone(), calls[0].arguments.clone())
                } else {
                    let tool_args_summary: Vec<Value> = calls
                        .iter()
                        .map(|c| {
                            serde_json::json!({
                                "tool": c.tool_name,
                                "arguments": c.arguments
                            })
                        })
                        .collect();
                    (
                        format!("{} tools", calls.len()),
                        serde_json::json!(tool_args_summary),
                    )
                };

                let (tracked, step) = tracking
                    .track_tool_execution(task_id.clone(), tool_name, tool_arguments)
                    .await?;

                context
                    .tx
                    .send(StreamEvent::ExecutionStepUpdate { step })
                    .ok();

                let tool_executor = ContextToolExecutor {
                    context: context.clone(),
                };

                let state = execute_tools_with_templates(
                    &calls,
                    &tools,
                    &context.request_ctx,
                    &tool_executor,
                    &context.log,
                )
                .await?;

                let execution_summary = format_results_for_response(&state);

                let has_failures = state.failed_results().len() > 0;

                if has_failures {
                    let error_message = state
                        .failed_results()
                        .iter()
                        .filter_map(|r| r.error.as_ref())
                        .map(|e| e.as_str())
                        .collect::<Vec<_>>()
                        .join("; ");

                    tracking.fail(&tracked, error_message).await.ok();
                } else {
                    let tool_result = if state.results.len() == 1 {
                        serde_json::json!({
                            "tool": state.results[0].tool_name,
                            "output": state.results[0].output,
                            "duration_ms": state.results[0].duration_ms
                        })
                    } else {
                        serde_json::json!({
                            "results": state.results.iter().map(|r| {
                                serde_json::json!({
                                    "tool": r.tool_name,
                                    "output": r.output,
                                    "duration_ms": r.duration_ms
                                })
                            }).collect::<Vec<_>>()
                        })
                    };

                    tracking.complete(tracked, Some(tool_result)).await.ok();
                }

                context
                    .log
                    .info(
                        "planned",
                        &format!(
                            "Execution complete: {} succeeded, {} failed",
                            state.successful_results().len(),
                            state.failed_results().len()
                        ),
                    )
                    .await
                    .ok();

                if let Ok(step) = tracking.track_completion(task_id).await {
                    context
                        .tx
                        .send(StreamEvent::ExecutionStepUpdate { step })
                        .ok();
                }

                let response = context
                    .ai_service
                    .generate_response_with_model(
                        messages,
                        &execution_summary,
                        &context.request_ctx,
                        context.agent_runtime.provider.as_deref(),
                        context.agent_runtime.model.as_deref(),
                    )
                    .await?;

                context.tx.send(StreamEvent::Text(response.clone())).ok();

                let tool_calls = convert_to_tool_calls(&calls);
                let tool_results = convert_to_call_tool_results(&state);

                Ok(ExecutionResult {
                    accumulated_text: response,
                    tool_calls,
                    tool_results,
                    iterations: 1,
                })
            },
        }
    }

    fn name(&self) -> &'static str {
        "planned"
    }
}
