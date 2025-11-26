use anyhow::{anyhow, Context as AnyhowContext, Result};
use std::sync::Arc;
use uuid::Uuid;

use crate::models::ai::{AiMessage, MessageRole, SamplingMetadata};
use crate::models::tools::{CallToolResult, McpTool, ToolCall};
use crate::repository::AiRequestRepository;
use crate::services::mcp::McpClientManager;
use crate::services::providers::AiProvider;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::RequestContext;
use systemprompt_models::ai::tools::ToolCallExt;

struct ExecutionState {
    conversation: Vec<AiMessage>,
    all_tool_calls: Vec<ToolCall>,
    all_tool_results: Vec<CallToolResult>,
    total_tokens: u64,
    final_response: String,
}

struct IterationOutcome {
    response_text: String,
    tool_calls: Vec<ToolCall>,
    tool_results: Vec<CallToolResult>,
    tokens: u64,
    should_continue: bool,
}

#[derive(Debug)]
pub struct AgenticExecutor {
    client_manager: Arc<McpClientManager>,
    ai_request_repo: AiRequestRepository,
    max_iterations: u32,
}

impl AgenticExecutor {
    pub const fn new(
        client_manager: Arc<McpClientManager>,
        ai_request_repo: AiRequestRepository,
        max_iterations: u32,
    ) -> Self {
        Self {
            client_manager,
            ai_request_repo,
            max_iterations,
        }
    }

    pub async fn execute_loop(
        &self,
        provider: &dyn AiProvider,
        initial_messages: Vec<AiMessage>,
        tools: Vec<McpTool>,
        metadata: &SamplingMetadata,
        model: &str,
        context: &RequestContext,
        logger: &LogService,
    ) -> Result<AgenticExecutionResult> {
        let mut state = ExecutionState::new(initial_messages);

        logger
            .info(
                "agentic_executor",
                &format!(
                    "Starting agentic execution loop: max_iterations={}, context_id={}",
                    self.max_iterations,
                    context.context_id().as_str()
                ),
            )
            .await
            .ok();

        for iteration in 1..=self.max_iterations {
            let outcome = self
                .execute_iteration(
                    iteration,
                    provider,
                    &state.conversation,
                    tools.clone(),
                    metadata,
                    model,
                    context,
                    logger,
                )
                .await?;

            state.apply_iteration_outcome(&outcome);

            if !outcome.should_continue {
                logger
                    .info(
                        "agentic_executor",
                        &format!("Stopping at iteration {iteration} - no executable tools"),
                    )
                    .await
                    .ok();
                break;
            }
        }

        let result = state.into_result();

        if result.final_response.is_empty() {
            logger
                .warn(
                    "agentic_executor",
                    &format!(
                        "Reached max iterations ({}) without completing. Task may be incomplete.",
                        self.max_iterations
                    ),
                )
                .await
                .ok();
        } else {
            logger.info("agentic_executor", &format!(
                "Agentic execution completed: total_iterations={}, total_tokens={}, total_tool_calls={}",
                result.total_iterations, result.total_tokens, result.tool_calls.len()
            )).await.ok();
        }

        Ok(result)
    }

    async fn execute_iteration(
        &self,
        iteration: u32,
        provider: &dyn AiProvider,
        conversation: &[AiMessage],
        tools: Vec<McpTool>,
        metadata: &SamplingMetadata,
        model: &str,
        context: &RequestContext,
        logger: &LogService,
    ) -> Result<IterationOutcome> {
        logger
            .info(
                "agentic_executor",
                &format!("Iteration {}/{} starting", iteration, self.max_iterations),
            )
            .await
            .ok();

        let (response, tool_calls) = provider
            .sample_with_tools(conversation, tools.clone(), metadata, model)
            .await
            .with_context(|| format!("AI call failed at iteration {iteration}"))?;

        let ai_request_id = self
            .track_ai_request(conversation, &response, &tool_calls, context, iteration)
            .await?;

        let tokens = u64::from(response.tokens_used.unwrap_or(0));
        let executable_tools = Self::filter_executable_tools(&tool_calls);

        self.log_iteration_summary(
            iteration,
            &tool_calls,
            &executable_tools,
            &response.content,
            logger,
        )
        .await;

        if executable_tools.is_empty() {
            return Ok(IterationOutcome::terminal(
                response.content,
                executable_tools,
                Vec::new(),
                tokens,
            ));
        }

        let tool_results = self
            .execute_tools(
                &tool_calls,
                &tools,
                context,
                &ai_request_id.to_string(),
                iteration as i32,
            )
            .await?;

        self.log_tool_results(iteration, &executable_tools, &tool_results, logger)
            .await;

        Ok(IterationOutcome::continue_with(
            response.content,
            executable_tools,
            tool_results,
            tokens,
        ))
    }

    fn filter_executable_tools(tool_calls: &[ToolCall]) -> Vec<ToolCall> {
        tool_calls.filter_executable()
    }

    async fn log_iteration_summary(
        &self,
        iteration: u32,
        tool_calls: &[ToolCall],
        executable_tools: &[ToolCall],
        response_content: &str,
        logger: &LogService,
    ) {
        logger
            .info(
                "agentic_executor",
                &format!(
                    "Iteration {}: total_tools={}, executable_tools={}, ai_response_length={}",
                    iteration,
                    tool_calls.len(),
                    executable_tools.len(),
                    response_content.len()
                ),
            )
            .await
            .ok();

        for tc in executable_tools {
            let has_execute = tc
                .arguments
                .as_object()
                .is_some_and(|o| o.contains_key("execute"));

            logger
                .info(
                    "agentic_executor",
                    &format!(
                        "Iteration {}: tool_call name={}, has_execute_param={}",
                        iteration, tc.name, has_execute
                    ),
                )
                .await
                .ok();

            if let Some(execute_value) = tc.arguments.get("execute") {
                logger
                    .info(
                        "agentic_executor",
                        &format!(
                            "Iteration {}: tool={}, execute={}",
                            iteration, tc.name, execute_value
                        ),
                    )
                    .await
                    .ok();
            }
        }
    }

    async fn log_tool_results(
        &self,
        iteration: u32,
        executable_tools: &[ToolCall],
        tool_results: &[CallToolResult],
        logger: &LogService,
    ) {
        use rmcp::model::RawContent;

        logger
            .info(
                "agentic_executor",
                &format!(
                    "Iteration {}: executed {} tools, got {} results",
                    iteration,
                    executable_tools.len(),
                    tool_results.len()
                ),
            )
            .await
            .ok();

        for (call, result) in executable_tools.iter().zip(tool_results.iter()) {
            let content_text: String = result
                .content
                .iter()
                .filter_map(|c| match &c.raw {
                    RawContent::Text(text_content) => Some(text_content.text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n");

            let result_preview = if content_text.len() > 200 {
                format!("{}...", &content_text[..200])
            } else {
                content_text
            };
            logger
                .info(
                    "agentic_executor",
                    &format!(
                        "Iteration {}: tool_result call={}, is_error={}, content_preview={}",
                        iteration,
                        call.name,
                        result.is_error.unwrap_or(false),
                        result_preview
                    ),
                )
                .await
                .ok();
        }
    }

    async fn track_ai_request(
        &self,
        messages: &[AiMessage],
        response: &crate::models::ai::SamplingResponse,
        tool_calls: &[ToolCall],
        context: &RequestContext,
        _iteration: u32,
    ) -> Result<Uuid> {
        use crate::repository::ai_requests::{AiRequestMessage, AiRequestToolCall, SamplingParams};

        let request_id = Uuid::new_v4();

        let repo_messages: Vec<AiRequestMessage> = messages.iter().map(Into::into).collect();

        let repo_tool_calls: Vec<AiRequestToolCall> = tool_calls
            .filter_storable()
            .iter()
            .map(|tc| AiRequestToolCall {
                tool_name: tc.name.clone(),
                tool_input: serde_json::to_string(&tc.arguments).unwrap_or_default(),
                mcp_execution_id: None,
                ai_tool_call_id: Some(tc.ai_tool_call_id.as_ref().to_string()),
            })
            .collect();

        let sampling_params = SamplingParams::default();

        self.ai_request_repo
            .store_ai_request(
                request_id,
                context.user_id(),
                context.session_id(),
                context.task_id(),
                Some(context.context_id()),
                Some(context.trace_id()),
                &response.provider,
                &response.model,
                &repo_messages,
                &sampling_params,
                if repo_tool_calls.is_empty() {
                    None
                } else {
                    Some(&repo_tool_calls)
                },
                response.tokens_used.map(|t| t as i32),
                response.input_tokens.map(|t| t as i32),
                response.output_tokens.map(|t| t as i32),
                response.cache_hit,
                response.cache_read_tokens.map(|t| t as i32),
                response.cache_creation_tokens.map(|t| t as i32),
                response.is_streaming,
                0,
                response.latency_ms as i32,
                "completed",
                None,
            )
            .await?;

        self.ai_request_repo
            .add_response_message(request_id, &response.content)
            .await?;

        Ok(request_id)
    }

    async fn execute_tools(
        &self,
        tool_calls: &[ToolCall],
        tools: &[McpTool],
        context: &RequestContext,
        _ai_request_id: &str,
        _iteration: i32,
    ) -> Result<Vec<CallToolResult>> {
        let mut results = Vec::new();

        for call in tool_calls {
            if call.is_meta_tool() {
                continue;
            }

            let tool = tools
                .iter()
                .find(|t| t.name == call.name)
                .ok_or_else(|| anyhow!("Tool '{}' not found in provided tools list", call.name))?;

            let result = self
                .client_manager
                .execute_tool(call, &tool.service_id, context)
                .await?;

            results.push(result);
        }

        Ok(results)
    }
}

impl ExecutionState {
    const fn new(initial_messages: Vec<AiMessage>) -> Self {
        Self {
            conversation: initial_messages,
            all_tool_calls: Vec::new(),
            all_tool_results: Vec::new(),
            total_tokens: 0,
            final_response: String::new(),
        }
    }

    fn apply_iteration_outcome(&mut self, outcome: &IterationOutcome) {
        use rmcp::model::RawContent;

        self.final_response.clone_from(&outcome.response_text);
        self.total_tokens += outcome.tokens;

        let assistant_msg = AiMessage {
            role: MessageRole::Assistant,
            content: outcome.response_text.clone(),
        };

        let tool_results_text = outcome
            .tool_calls
            .iter()
            .zip(outcome.tool_results.iter())
            .map(|(call, r)| {
                let content_text: String = r
                    .content
                    .iter()
                    .filter_map(|c| match &c.raw {
                        RawContent::Text(text_content) => Some(text_content.text.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                if r.is_error.unwrap_or(false) {
                    format!("Error from tool call {}: {}", call.name, content_text)
                } else {
                    format!("Tool call {} result: {}", call.name, content_text)
                }
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        let user_msg = AiMessage {
            role: MessageRole::User,
            content: tool_results_text,
        };

        self.conversation.push(assistant_msg);
        self.conversation.push(user_msg);

        self.all_tool_calls.extend(outcome.tool_calls.clone());
        self.all_tool_results.extend(outcome.tool_results.clone());
    }

    fn into_result(self) -> AgenticExecutionResult {
        AgenticExecutionResult {
            final_response: self.final_response,
            tool_calls: self.all_tool_calls,
            tool_results: self.all_tool_results,
            total_iterations: self.conversation.len() / 2,
            total_tokens: self.total_tokens,
            total_latency_ms: 0,
        }
    }
}

impl IterationOutcome {
    const fn terminal(
        response_text: String,
        tool_calls: Vec<ToolCall>,
        tool_results: Vec<CallToolResult>,
        tokens: u64,
    ) -> Self {
        Self {
            response_text,
            tool_calls,
            tool_results,
            tokens,
            should_continue: false,
        }
    }

    const fn continue_with(
        response_text: String,
        tool_calls: Vec<ToolCall>,
        tool_results: Vec<CallToolResult>,
        tokens: u64,
    ) -> Self {
        Self {
            response_text,
            tool_calls,
            tool_results,
            tokens,
            should_continue: true,
        }
    }
}

#[derive(Debug)]
pub struct AgenticExecutionResult {
    pub final_response: String,
    pub tool_calls: Vec<ToolCall>,
    pub tool_results: Vec<CallToolResult>,
    pub total_iterations: usize,
    pub total_tokens: u64,
    pub total_latency_ms: u64,
}
