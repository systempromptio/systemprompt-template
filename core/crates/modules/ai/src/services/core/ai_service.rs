use anyhow::Result;
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::ai::{
    AiMessage, GenerateRequest, GenerateResponse, MessageRole, SamplingMetadata, SamplingRequest,
    SamplingResponse, SearchGroundedResponse, TooledRequest, TooledResponse,
};
use crate::models::tools::{CallToolResult, McpTool, ToolCall};
use crate::repository::AiRequestRepository;
use crate::services::config::{ConfigLoader, ConfigValidator};
use crate::services::mcp::{McpClientManager, ToolDiscovery};
use crate::services::providers::{AiProvider, ProviderFactory};
use crate::services::sampling::SamplingRouter;
use crate::services::tooled::{ResponseStrategy, ResponseSynthesizer, TooledExecutor};

use systemprompt_core_logging::LogService;
use systemprompt_core_system::repository::AnalyticsSessionRepository;
use systemprompt_core_system::AppContext;

pub use systemprompt_core_system::RequestContext;

pub struct AiService {
    providers: HashMap<String, Arc<dyn AiProvider>>,
    sampling_router: Arc<SamplingRouter>,
    mcp_client_manager: Arc<McpClientManager>,
    tool_discovery: Arc<ToolDiscovery>,
    tooled_executor: TooledExecutor,
    synthesizer: ResponseSynthesizer,
    db_pool: systemprompt_core_database::DbPool,
    default_provider: String,
    ai_request_repo: AiRequestRepository,
    session_repo: AnalyticsSessionRepository,
}

impl std::fmt::Debug for AiService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AiService")
            .field("default_provider", &self.default_provider)
            .finish_non_exhaustive()
    }
}

impl AiService {
    pub async fn new(app_context: Arc<AppContext>) -> Result<Self> {
        let db_pool = app_context.db_pool().clone();
        let logger = LogService::system(db_pool.clone());
        let ai_request_repo = AiRequestRepository::new(db_pool.clone());
        let session_repo = AnalyticsSessionRepository::new(db_pool.clone());

        let mut config = ConfigLoader::load_default()?;
        ConfigLoader::expand_env_vars(&mut config)?;
        ConfigValidator::validate(&config, &logger).await?;

        let providers =
            ProviderFactory::create_all(config.providers.clone(), Some(&db_pool))?;
        let default_provider = config.default_provider.clone();

        let sampling_router = Arc::new(SamplingRouter::new(
            providers.clone(),
            default_provider.clone(),
        ));

        let mcp_client_manager = Arc::new(McpClientManager::new(app_context.clone()));
        let tool_discovery = Arc::new(ToolDiscovery::new(mcp_client_manager.clone()));
        let tooled_executor = TooledExecutor::new(mcp_client_manager.clone());
        let synthesizer = ResponseSynthesizer::new();

        Ok(Self {
            providers,
            sampling_router,
            mcp_client_manager,
            tool_discovery,
            tooled_executor,
            synthesizer,
            db_pool,
            default_provider,
            ai_request_repo,
            session_repo,
        })
    }

    pub async fn sample(
        &self,
        request: SamplingRequest,
        ctx: RequestContext,
    ) -> Result<SamplingResponse> {
        self.process_sampling_request(request, &ctx).await
    }

    pub async fn generate(
        &self,
        request: GenerateRequest,
        ctx: RequestContext,
    ) -> Result<GenerateResponse> {
        let mut hints = Vec::new();
        // Add ModelId hint first so it takes precedence over Provider hint
        if let Some(model) = &request.model {
            hints.push(crate::models::ai::ModelHint::ModelId(model.clone()));
        }
        if let Some(provider) = &request.provider {
            hints.push(crate::models::ai::ModelHint::Provider(provider.clone()));
        }

        let mut metadata = request.metadata.unwrap_or_default();

        // Inject request context into metadata
        metadata.user_id = Some(ctx.user_id().clone());
        metadata.session_id = Some(ctx.session_id().clone());
        metadata.trace_id = Some(ctx.trace_id().clone());

        let sampling_request = SamplingRequest {
            messages: request.messages,
            model_preferences: crate::models::ai::ModelPreferences {
                hints,
                cost_priority: None,
            },
            metadata,
            system_prompt: None,
            include_context: None,
            max_tokens: 4096,
            response_format: request.response_format,
            structured_output: request.structured_output,
        };

        let response = self
            .process_sampling_request(sampling_request, &ctx)
            .await?;

        Ok(GenerateResponse {
            request_id: response.request_id,
            content: response.content,
            provider: response.provider,
            model: response.model,
            tokens_used: response.tokens_used,
            latency_ms: response.latency_ms,
        })
    }

    pub async fn generate_direct(
        &self,
        provider: &str,
        model: &str,
        messages: Vec<AiMessage>,
        metadata: Option<SamplingMetadata>,
        ctx: RequestContext,
    ) -> Result<GenerateResponse> {
        let request = GenerateRequest {
            provider: Some(provider.to_string()),
            model: Some(model.to_string()),
            messages,
            metadata,
            response_format: None,
            structured_output: None,
        };

        self.generate(request, ctx).await
    }

    pub async fn list_available_tools_for_agent(
        &self,
        agent_name: &systemprompt_identifiers::AgentName,
        context: &RequestContext,
    ) -> Result<Vec<McpTool>> {
        self.tool_discovery
            .discover_tools(agent_name, context)
            .await
    }

    pub async fn generate_with_tools(
        &self,
        request: TooledRequest,
        ctx: RequestContext,
    ) -> Result<TooledResponse> {
        self.process_tooled_request(request, &ctx).await
    }

    /// Makes an AI call with tools but does NOT execute the tool calls.
    /// Returns the AI's response and requested tool calls for later execution.
    /// Use this when you need to inspect tool calls before deciding whether to execute them.
    pub async fn sample_without_execution(
        &self,
        request: TooledRequest,
        ctx: &RequestContext,
    ) -> Result<(SamplingResponse, Vec<ToolCall>)> {
        let logger = LogService::new(self.db_pool.clone(), ctx.log_context());
        let (provider_name, provider, model) = self.select_provider_and_model(&request)?;

        logger
            .info(
                "ai",
                &format!(
                    "🔍 sample_without_execution: provider={}, model={}, tools={}",
                    provider_name,
                    model,
                    request.tools.len()
                ),
            )
            .await
            .ok();

        let metadata = request.metadata.clone().unwrap_or_default();
        self.call_ai_with_tools(provider.as_ref(), &request, &metadata, &model)
            .await
    }

    /// Executes tool calls that were previously returned by `sample_without_execution`.
    /// This separates the AI call from tool execution, allowing inspection of tool calls
    /// before deciding whether/how to execute them.
    pub async fn execute_tools(
        &self,
        tool_calls: Vec<ToolCall>,
        tools: &[McpTool],
        ctx: &RequestContext,
    ) -> (Vec<ToolCall>, Vec<CallToolResult>) {
        self.tooled_executor
            .execute_tool_calls(tool_calls, tools, ctx)
            .await
    }

    pub async fn generate_with_google_search(
        &self,
        messages: Vec<AiMessage>,
        metadata: Option<SamplingMetadata>,
        model: Option<&str>,
        ctx: RequestContext,
        urls: Option<Vec<String>>,
        response_schema: Option<serde_json::Value>,
    ) -> Result<SearchGroundedResponse> {
        let mut metadata = metadata.unwrap_or_default();

        metadata.user_id = Some(ctx.user_id().clone());
        metadata.session_id = Some(ctx.session_id().clone());
        metadata.trace_id = Some(ctx.trace_id().clone());

        let provider = self
            .providers
            .get("gemini")
            .ok_or_else(|| anyhow::anyhow!("Gemini provider not available for Google Search"))?;

        let model = model.unwrap_or(provider.default_model());
        let response = provider
            .sample_with_google_search(&messages, &metadata, model, urls, response_schema)
            .await?;

        Ok(response)
    }

    pub async fn health_check(&self) -> Result<HashMap<String, bool>> {
        let mut health = HashMap::new();

        for name in self.providers.keys() {
            health.insert(format!("provider_{name}"), true);
        }

        let mcp_health = self.mcp_client_manager.health_check().await?;
        for (service_id, is_healthy) in mcp_health {
            health.insert(format!("mcp_{service_id}"), is_healthy);
        }

        Ok(health)
    }

    pub fn default_provider(&self) -> &str {
        &self.default_provider
    }

    pub async fn generate_stream(
        &self,
        request: GenerateRequest,
        ctx: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let provider_name = request
            .provider
            .as_deref()
            .unwrap_or(&self.default_provider);
        let provider = self
            .providers
            .get(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Provider {provider_name} not found"))?;

        if !provider.supports_streaming() {
            return Err(anyhow::anyhow!(
                "Provider {provider_name} does not support streaming"
            ));
        }

        let model = request
            .model
            .as_deref()
            .unwrap_or_else(|| provider.default_model());
        let mut metadata = request.metadata.unwrap_or_default();

        // Inject request context into metadata
        metadata.user_id = Some(ctx.user_id().clone());
        metadata.session_id = Some(ctx.session_id().clone());
        metadata.trace_id = Some(ctx.trace_id().clone());

        provider
            .sample_stream(&request.messages, &metadata, model)
            .await
    }

    pub async fn generate_with_tools_stream(
        &self,
        request: TooledRequest,
        ctx: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let provider_name = request
            .provider
            .as_deref()
            .unwrap_or(&self.default_provider);
        let provider = self
            .providers
            .get(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Provider {provider_name} not found"))?;

        if !provider.supports_streaming() {
            return Err(anyhow::anyhow!(
                "Provider {provider_name} does not support streaming"
            ));
        }

        let model = request
            .model
            .as_deref()
            .unwrap_or_else(|| provider.default_model());
        let mut metadata = request.metadata.unwrap_or_default();

        // Inject request context into metadata
        metadata.user_id = Some(ctx.user_id().clone());
        metadata.session_id = Some(ctx.session_id().clone());
        metadata.trace_id = Some(ctx.trace_id().clone());

        provider
            .sample_with_tools_stream(&request.messages, request.tools, &metadata, model)
            .await
    }

    pub async fn execute_agentic_loop(
        &self,
        messages: Vec<AiMessage>,
        tools: Vec<McpTool>,
        context: &RequestContext,
    ) -> Result<super::agentic_executor::AgenticExecutionResult> {
        let logger = LogService::new(self.db_pool.clone(), context.log_context());

        let executor = super::agentic_executor::AgenticExecutor::new(
            self.mcp_client_manager.clone(),
            self.ai_request_repo.clone(),
            10,
        );

        let (provider_name, provider) = self.sampling_router.select_provider(
            &crate::models::ai::ModelPreferences {
                hints: vec![],
                cost_priority: None,
            },
            &SamplingMetadata::default(),
        )?;

        let model = self.sampling_router.select_model(
            &provider_name,
            &crate::models::ai::ModelPreferences {
                hints: vec![],
                cost_priority: None,
            },
        )?;

        let mut tools_with_control = tools.clone();
        tools_with_control.insert(
            0,
            crate::services::execution_control::create_execution_control_tool(),
        );

        let mut messages_with_instructions = messages.clone();
        Self::inject_backend_system_instructions(&mut messages_with_instructions);

        executor
            .execute_loop(
                provider.as_ref(),
                messages_with_instructions,
                tools_with_control,
                &SamplingMetadata::default(),
                &model,
                context,
                &logger,
            )
            .await
    }

    async fn process_sampling_request(
        &self,
        request: SamplingRequest,
        ctx: &RequestContext,
    ) -> Result<SamplingResponse> {
        let request_id = Uuid::new_v4();
        let start = std::time::Instant::now();

        let logger = LogService::new(self.db_pool.clone(), ctx.log_context());

        let (provider_name, provider) = self
            .sampling_router
            .select_provider(&request.model_preferences, &request.metadata)?;

        let model = self
            .sampling_router
            .select_model(&provider_name, &request.model_preferences)?;

        // Enhanced logging with request context
        let log_message = format!(
            "Request {}: {} messages to {}/{} (user: {}, session: {}, trace: {})",
            request_id,
            request.messages.len(),
            provider_name,
            model,
            ctx.user_id().as_str(),
            ctx.session_id(),
            ctx.trace_id()
        );

        logger.info("ai", &log_message).await.ok();

        let sample_result = if let Some(ref format) = request.response_format {
            // Use structured sampling if provider supports it
            if provider.supports_json_mode() || provider.supports_structured_output() {
                provider
                    .sample_structured(&request.messages, &request.metadata, &model, format)
                    .await
            } else {
                // Fall back to regular sampling with prompt enhancement
                use crate::services::structured_output::StructuredOutputProcessor;
                let options = request
                    .structured_output
                    .clone()
                    .unwrap_or_default();
                let enhanced_messages = if let Some(last_msg) = request.messages.last() {
                    let mut messages = request.messages[..request.messages.len() - 1].to_vec();
                    let enhanced_content = StructuredOutputProcessor::enhance_prompt_for_json(
                        &last_msg.content,
                        format,
                        &options,
                    );
                    messages.push(AiMessage {
                        role: last_msg.role,
                        content: enhanced_content,
                    });
                    messages
                } else {
                    request.messages.clone()
                };
                provider
                    .sample(&enhanced_messages, &request.metadata, &model)
                    .await
            }
        } else {
            provider
                .sample(&request.messages, &request.metadata, &model)
                .await
        };

        match sample_result {
            Ok(mut response) => {
                // If we have a response format and structured output options, validate the response
                if let (Some(ref format), Some(ref options)) =
                    (&request.response_format, &request.structured_output)
                {
                    if format.is_json() {
                        use crate::services::structured_output::StructuredOutputProcessor;
                        match StructuredOutputProcessor::process_response(
                            &response.content,
                            format,
                            options,
                        ) {
                            Ok(json_value) => {
                                // Update response content with properly formatted JSON
                                response.content = serde_json::to_string(&json_value)?;
                            },
                            Err(e) => {
                                logger
                                    .warn(
                                        "ai",
                                        &format!("Failed to validate structured output: {e}"),
                                    )
                                    .await
                                    .ok();
                                // Optionally, we could retry here or return an error
                            },
                        }
                    }
                }

                response.request_id = request_id;
                let latency = start.elapsed().as_millis() as u64;

                // Store AI usage in ai_requests table
                self.store_ai_usage(&request, &response, ctx, latency, &logger)
                    .await
                    .ok();

                logger
                    .info(
                        "ai",
                        &format!(
                            "Response {}: {} chars, {} tokens in {}ms",
                            request_id,
                            response.content.len(),
                            response.tokens_used.unwrap_or(0),
                            latency
                        ),
                    )
                    .await
                    .ok();

                Ok(response)
            },
            Err(e) => {
                let latency = start.elapsed().as_millis() as u64;

                // Store failed request in ai_requests table
                self.store_failed_ai_request(
                    &request,
                    ctx,
                    &provider_name,
                    &model,
                    request_id,
                    latency as i32,
                    &e.to_string(),
                )
                .await
                .ok();

                logger
                    .error(
                        "ai",
                        &format!(
                            "Error {request_id}: {provider_name} failed after {latency}ms: {e}"
                        ),
                    )
                    .await
                    .ok();

                Err(e)
            },
        }
    }

    async fn process_tooled_request(
        &self,
        request: TooledRequest,
        ctx: &RequestContext,
    ) -> Result<TooledResponse> {
        let request_id = Uuid::new_v4();
        let start = std::time::Instant::now();
        let logger = LogService::new(self.db_pool.clone(), ctx.log_context());

        let (provider_name, provider, model) = self.select_provider_and_model(&request)?;
        self.log_tooled_request_start(&logger, request_id, &request, &provider_name, &model, ctx)
            .await;

        let metadata = request.metadata.clone().unwrap_or_default();

        match self
            .call_ai_with_tools(provider.as_ref(), &request, &metadata, &model)
            .await
        {
            Ok((response, tool_calls)) => {
                self.log_ai_response(&logger, &response, tool_calls.len())
                    .await;

                let (tool_calls, tool_results) = self
                    .tooled_executor
                    .execute_tool_calls(tool_calls, &request.tools, ctx)
                    .await;

                let latency = start.elapsed().as_millis() as u64;

                self.store_tooled_ai_usage(
                    &request,
                    &response,
                    ctx,
                    latency as i32,
                    &tool_calls,
                    &tool_results,
                    &provider_name,
                    &model,
                    request_id,
                    &logger,
                )
                .await
                .ok();

                let strategy = ResponseStrategy::from_response(
                    response.content.clone(),
                    tool_calls.clone(),
                    tool_results.clone(),
                );

                logger
                    .info(
                        "ai",
                        &format!(
                            "📋 STRATEGY: {} (content_len={}, tools={}, results={})",
                            match &strategy {
                                ResponseStrategy::ContentProvided { .. } =>
                                    "ContentProvided - using AI's response",
                                ResponseStrategy::ArtifactsProvided { .. } =>
                                    "ArtifactsProvided - valid artifacts, skipping synthesis",
                                ResponseStrategy::ToolsOnly { .. } =>
                                    "ToolsOnly - synthesizing from tool results",
                            },
                            response.content.len(),
                            tool_calls.len(),
                            tool_results.len()
                        ),
                    )
                    .await
                    .ok();

                let final_content = match strategy {
                    ResponseStrategy::ContentProvided { content, .. } => {
                        logger
                            .info(
                                "ai",
                                &format!("✅ Using AI content: {} chars", content.len()),
                            )
                            .await
                            .ok();
                        content
                    },
                    ResponseStrategy::ArtifactsProvided {
                        tool_calls: _,
                        tool_results,
                    } => {
                        logger
                            .info(
                                "ai",
                                &format!(
                                    "📦 Valid artifacts provided - skipping synthesis ({} results)",
                                    tool_results.len()
                                ),
                            )
                            .await
                            .ok();

                        String::new()
                    },
                    ResponseStrategy::ToolsOnly {
                        tool_calls,
                        tool_results,
                    } => {
                        logger
                            .info("ai", "🔄 Synthesizing response from tool results")
                            .await
                            .ok();

                        self.synthesizer
                            .synthesize_or_fallback(
                                provider.as_ref(),
                                &request.messages,
                                &tool_calls,
                                &tool_results,
                                &metadata,
                                &model,
                                &logger,
                            )
                            .await
                    },
                };

                let tooled_response = TooledResponse {
                    request_id,
                    content: final_content.clone(),
                    provider: provider_name.to_string(),
                    model: model.clone(),
                    tool_calls: tool_calls.clone(),
                    tool_results,
                    tokens_used: response.tokens_used,
                    latency_ms: latency,
                };

                self.log_tooled_response(&logger, &tooled_response).await;

                Ok(tooled_response)
            },
            Err(e) => {
                let latency = start.elapsed().as_millis() as u64;

                self.store_failed_tooled_request(
                    &request,
                    ctx,
                    &provider_name,
                    &model,
                    request_id,
                    latency as i32,
                    &e.to_string(),
                )
                .await
                .ok();

                logger
                    .error(
                        "ai",
                        &format!(
                            "Tooled Error {request_id}: {provider_name} failed after {latency}ms: {e}"
                        ),
                    )
                    .await
                    .ok();

                Err(e)
            },
        }
    }

    fn select_provider_and_model(
        &self,
        request: &TooledRequest,
    ) -> Result<(String, Arc<dyn AiProvider>, String)> {
        let mut hints = Vec::new();
        if let Some(model) = &request.model {
            hints.push(crate::models::ai::ModelHint::ModelId(model.clone()));
        }
        if let Some(provider) = &request.provider {
            hints.push(crate::models::ai::ModelHint::Provider(provider.clone()));
        }

        let model_preferences = crate::models::ai::ModelPreferences {
            hints,
            cost_priority: None,
        };

        let metadata = request.metadata.clone().unwrap_or_default();

        let (provider_name, provider) = self
            .sampling_router
            .select_provider(&model_preferences, &metadata)?;

        let model = self
            .sampling_router
            .select_model(&provider_name, &model_preferences)?;

        Ok((provider_name, provider, model))
    }

    async fn log_tooled_request_start(
        &self,
        logger: &LogService,
        request_id: Uuid,
        request: &TooledRequest,
        provider_name: &str,
        model: &str,
        ctx: &RequestContext,
    ) {
        let tool_names = request
            .tools
            .iter()
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let log_message = format!(
            "Tooled Request {}: {} messages with {} tools to {}/{} (user: {}, session: {}, trace: {})",
            request_id, request.messages.len(), request.tools.len(), provider_name, model,
            ctx.user_id().as_str(), ctx.session_id(), ctx.trace_id()
        );

        logger.info("ai", &log_message).await.ok();
        logger
            .info("ai", &format!("🔧 TOOLS PASSED TO AI: [{tool_names}]"))
            .await
            .ok();
    }

    async fn call_ai_with_tools(
        &self,
        provider: &dyn AiProvider,
        request: &TooledRequest,
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<(SamplingResponse, Vec<ToolCall>)> {
        let mut tools_with_control = request.tools.clone();
        tools_with_control.insert(
            0,
            crate::services::execution_control::create_execution_control_tool(),
        );

        let mut messages_with_instructions = request.messages.clone();
        Self::inject_backend_system_instructions(&mut messages_with_instructions);

        provider
            .sample_with_tools(
                &messages_with_instructions,
                tools_with_control,
                metadata,
                model,
            )
            .await
    }

    async fn log_ai_response(
        &self,
        logger: &LogService,
        response: &SamplingResponse,
        tool_call_count: usize,
    ) {
        logger
            .info(
                "ai",
                &format!(
                    "🤖 AI RESPONSE: {} chars, {} tool calls",
                    response.content.len(),
                    tool_call_count
                ),
            )
            .await
            .ok();

        if tool_call_count == 0 && !response.content.is_empty() {
            logger
                .warn(
                    "ai",
                    &format!(
                        "⚠️  AI responded with TEXT instead of calling tools. Response preview: {}",
                        &response.content.chars().take(200).collect::<String>()
                    ),
                )
                .await
                .ok();
        }
    }

    async fn log_tooled_response(&self, logger: &LogService, response: &TooledResponse) {
        logger
            .info(
                "ai",
                &format!(
                    "Tooled Response {}: {} chars, {} tokens, {} tools in {}ms",
                    response.request_id,
                    response.content.len(),
                    response.tokens_used.unwrap_or(0),
                    response.tool_calls.len(),
                    response.latency_ms
                ),
            )
            .await
            .ok();
    }

    // AI Usage Storage Methods

    async fn store_ai_usage(
        &self,
        request: &SamplingRequest,
        response: &SamplingResponse,
        ctx: &RequestContext,
        latency_ms: u64,
        logger: &LogService,
    ) -> Result<()> {
        use crate::repository::ai_requests::{AiRequestMessage, SamplingParams};

        let cost_cents = self.estimate_cost(
            &response.provider,
            &response.model,
            response.tokens_used.map(|t| t as i32),
        );

        let messages: Vec<AiRequestMessage> = request.messages.iter().map(Into::into).collect();

        let sampling_params = SamplingParams::from(&request.metadata);

        let repo = self.ai_request_repo.clone();
        let request_id = response.request_id;
        let user_id = ctx.user_id().clone();
        let session_id = ctx.session_id().clone();
        let task_id = ctx.task_id().cloned();
        let context_id = ctx.context_id().clone();
        let trace_id = ctx.trace_id().clone();
        let provider = response.provider.clone();
        let model = response.model.clone();
        let messages_clone = messages.clone();
        let sampling_params_clone = sampling_params.clone();
        let content = response.content.clone();
        let tokens_used = response.tokens_used.map(|t| t as i32);
        let input_tokens = response.input_tokens.map(|t| t as i32);
        let output_tokens = response.output_tokens.map(|t| t as i32);
        let cache_hit = response.cache_hit;
        let cache_read_tokens = response.cache_read_tokens.map(|t| t as i32);
        let cache_creation_tokens = response.cache_creation_tokens.map(|t| t as i32);
        let is_streaming = response.is_streaming;

        tokio::spawn(async move {
            if let Err(e) = repo
                .store_ai_request(
                    request_id,
                    &user_id,
                    &session_id,
                    task_id.as_ref(),
                    if context_id.as_str().is_empty() {
                        None
                    } else {
                        Some(&context_id)
                    },
                    Some(&trace_id),
                    &provider,
                    &model,
                    &messages_clone,
                    &sampling_params_clone,
                    None,
                    tokens_used,
                    input_tokens,
                    output_tokens,
                    cache_hit,
                    cache_read_tokens,
                    cache_creation_tokens,
                    is_streaming,
                    cost_cents,
                    latency_ms as i32,
                    "completed",
                    None,
                )
                .await
            {
                tracing::error!("Failed to store AI request {}: {}", request_id, e);
            }

            if let Err(e) = repo.add_response_message(request_id, &content).await {
                tracing::error!(
                    "Failed to add response message for request {}: {}",
                    request_id,
                    e
                );
            }
        });

        let tokens = response.tokens_used.unwrap_or(0) as i32;

        let is_system_user = ctx.user_id().as_str() == "system";

        if !is_system_user
            && !self
                .session_repo
                .session_exists(ctx.session_id().as_str())
                .await
                .unwrap_or(false)
        {
            logger
                .info(
                    "ai",
                    &format!(
                        "Creating session: {} with user_id: {:?}",
                        ctx.session_id(),
                        ctx.user_id()
                    ),
                )
                .await
                .ok();
            let jwt_expiration_seconds =
                systemprompt_core_system::Config::global().jwt_access_token_expiration;
            let expires_at = chrono::Utc::now() + chrono::Duration::seconds(jwt_expiration_seconds);

            if let Err(e) = self
                .session_repo
                .create_session(
                    ctx.session_id().as_str(),
                    Some(ctx.user_id().as_str()),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    false,
                    expires_at,
                )
                .await
            {
                logger
                    .error("ai", &format!("Failed to create session: {e}"))
                    .await
                    .ok();
            } else {
                logger.info("ai", "Session created successfully").await.ok();
            }
        }

        let session_repo = self.session_repo.clone();
        let session_id_for_usage = ctx.session_id().clone();

        tokio::spawn(async move {
            if let Err(e) = session_repo
                .increment_ai_usage(session_id_for_usage.as_str(), tokens, cost_cents)
                .await
            {
                tracing::error!(
                    "Failed to increment AI usage for session {}: {}",
                    session_id_for_usage,
                    e
                );
            }
        });

        Ok(())
    }

    async fn store_failed_ai_request(
        &self,
        request: &SamplingRequest,
        ctx: &RequestContext,
        provider: &str,
        model: &str,
        request_id: Uuid,
        latency_ms: i32,
        error_message: &str,
    ) -> Result<()> {
        use crate::repository::ai_requests::{AiRequestMessage, SamplingParams};

        let messages: Vec<AiRequestMessage> = request.messages.iter().map(Into::into).collect();

        let sampling_params = SamplingParams::from(&request.metadata);

        {
            let repo = self.ai_request_repo.clone();
            let user_id = ctx.user_id().clone();
            let session_id = ctx.session_id().clone();
            let task_id = ctx.task_id().cloned();
            let context_id = ctx.context_id().clone();
            let trace_id = ctx.trace_id().clone();
            let provider = provider.to_string();
            let model = model.to_string();
            let messages_clone = messages;
            let sampling_params_clone = sampling_params;
            let error_msg = error_message.to_string();

            tokio::spawn(async move {
                if let Err(e) = repo
                    .store_ai_request(
                        request_id,
                        &user_id,
                        &session_id,
                        task_id.as_ref(),
                        if context_id.as_str().is_empty() {
                            None
                        } else {
                            Some(&context_id)
                        },
                        Some(&trace_id),
                        &provider,
                        &model,
                        &messages_clone,
                        &sampling_params_clone,
                        None,
                        None,
                        None,
                        None,
                        false,
                        None,
                        None,
                        false,
                        0,
                        latency_ms,
                        "failed",
                        Some(&error_msg),
                    )
                    .await
                {
                    tracing::error!("Failed to store failed AI request {}: {}", request_id, e);
                }
            });
        }

        Ok(())
    }

    async fn store_tooled_ai_usage(
        &self,
        request: &TooledRequest,
        response: &SamplingResponse,
        ctx: &RequestContext,
        latency_ms: i32,
        tool_calls: &[ToolCall],
        _tool_results: &[CallToolResult],
        provider: &str,
        model: &str,
        request_id: Uuid,
        _logger: &LogService,
    ) -> Result<()> {
        use crate::repository::ai_requests::{AiRequestMessage, AiRequestToolCall, SamplingParams};

        let cost_cents =
            self.estimate_cost(provider, model, response.tokens_used.map(|t| t as i32));

        let messages: Vec<AiRequestMessage> = request.messages.iter().map(Into::into).collect();

        let sampling_params = request
            .metadata
            .as_ref()
            .map(SamplingParams::from)
            .unwrap_or_default();

        let tool_call_records: Vec<AiRequestToolCall> = tool_calls
            .iter()
            .map(|tc| AiRequestToolCall {
                tool_name: tc.name.clone(),
                tool_input: serde_json::to_string(&tc.arguments).unwrap_or_default(),
                mcp_execution_id: None,
                ai_tool_call_id: Some(tc.ai_tool_call_id.as_ref().to_string()),
            })
            .collect();

        {
            let repo = self.ai_request_repo.clone();
            let user_id = ctx.user_id().clone();
            let session_id = ctx.session_id().clone();
            let task_id = request.task_id.clone();
            let context_id = request.context_id.clone();
            let trace_id = ctx.trace_id().clone();
            let provider = provider.to_string();
            let model = model.to_string();
            let messages_clone = messages;
            let sampling_params_clone = sampling_params;
            let tool_calls_clone = tool_call_records;
            let tokens = response.tokens_used.map(|t| t as i32);
            let input_tokens = response.input_tokens.map(|t| t as i32);
            let output_tokens = response.output_tokens.map(|t| t as i32);
            let cache_hit = response.cache_hit;
            let cache_read_tokens = response.cache_read_tokens.map(|t| t as i32);
            let cache_creation_tokens = response.cache_creation_tokens.map(|t| t as i32);
            let is_streaming = response.is_streaming;
            let response_content = response.content.clone();

            tokio::spawn(async move {
                if let Err(e) = repo
                    .store_ai_request(
                        request_id,
                        &user_id,
                        &session_id,
                        Some(&task_id),
                        Some(&context_id),
                        Some(&trace_id),
                        &provider,
                        &model,
                        &messages_clone,
                        &sampling_params_clone,
                        Some(&tool_calls_clone),
                        tokens,
                        input_tokens,
                        output_tokens,
                        cache_hit,
                        cache_read_tokens,
                        cache_creation_tokens,
                        is_streaming,
                        cost_cents,
                        latency_ms,
                        "completed",
                        None,
                    )
                    .await
                {
                    tracing::error!("Failed to store tooled AI request {}: {}", request_id, e);
                }

                if let Err(e) = repo
                    .add_response_message(request_id, &response_content)
                    .await
                {
                    tracing::error!(
                        "Failed to add response message for tooled request {}: {}",
                        request_id,
                        e
                    );
                }
            });
        }

        let tokens = response.tokens_used.unwrap_or(0) as i32;

        {
            let session_repo = self.session_repo.clone();
            let session_id = ctx.session_id().clone();
            let user_id = ctx.user_id().clone();
            let is_system_user = user_id.as_str() == "system";

            tokio::spawn(async move {
                let jwt_expiration_seconds =
                    systemprompt_core_system::Config::global().jwt_access_token_expiration;
                let expires_at =
                    chrono::Utc::now() + chrono::Duration::seconds(jwt_expiration_seconds);

                if is_system_user {
                    return;
                }

                match session_repo.session_exists(session_id.as_str()).await {
                    Ok(exists) => {
                        if !exists {
                            if let Err(e) = session_repo
                                .create_session(
                                    session_id.as_str(),
                                    Some(user_id.as_str()),
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    None,
                                    false,
                                    expires_at,
                                )
                                .await
                            {
                                tracing::error!(
                                    "Failed to create session {} for tooled AI request: {}",
                                    session_id,
                                    e
                                );
                            }
                        }
                    },
                    Err(e) => {
                        tracing::error!("Failed to check if session {} exists: {}", session_id, e);
                    },
                }

                if let Err(e) = session_repo
                    .increment_ai_usage(session_id.as_str(), tokens, cost_cents)
                    .await
                {
                    tracing::error!(
                        "Failed to increment AI usage for session {}: {}",
                        session_id,
                        e
                    );
                }
            });
        }

        Ok(())
    }

    async fn store_failed_tooled_request(
        &self,
        request: &TooledRequest,
        ctx: &RequestContext,
        provider: &str,
        model: &str,
        request_id: Uuid,
        latency_ms: i32,
        error_message: &str,
    ) -> Result<()> {
        use crate::repository::ai_requests::{AiRequestMessage, SamplingParams};

        let messages: Vec<AiRequestMessage> = request.messages.iter().map(Into::into).collect();

        let sampling_params = request
            .metadata
            .as_ref()
            .map(SamplingParams::from)
            .unwrap_or_default();

        {
            let repo = self.ai_request_repo.clone();
            let user_id = ctx.user_id().clone();
            let session_id = ctx.session_id().clone();
            let task_id = request.task_id.clone();
            let context_id = request.context_id.clone();
            let trace_id = ctx.trace_id().clone();
            let provider = provider.to_string();
            let model = model.to_string();
            let messages_clone = messages;
            let sampling_params_clone = sampling_params;
            let error_msg = error_message.to_string();

            tokio::spawn(async move {
                if let Err(e) = repo
                    .store_ai_request(
                        request_id,
                        &user_id,
                        &session_id,
                        Some(&task_id),
                        Some(&context_id),
                        Some(&trace_id),
                        &provider,
                        &model,
                        &messages_clone,
                        &sampling_params_clone,
                        None,
                        None,
                        None,
                        None,
                        false,
                        None,
                        None,
                        false,
                        0,
                        latency_ms,
                        "failed",
                        Some(&error_msg),
                    )
                    .await
                {
                    tracing::error!(
                        "Failed to store failed tooled AI request {}: {}",
                        request_id,
                        e
                    );
                }
            });
        }

        Ok(())
    }

    fn inject_backend_system_instructions(messages: &mut Vec<AiMessage>) {
        let backend_instructions =
            crate::services::execution_control::EXECUTION_CONTROL_SYSTEM_INSTRUCTIONS;

        if let Some(system_msg) = messages
            .iter_mut()
            .find(|m| matches!(m.role, MessageRole::System))
        {
            system_msg.content = format!("{}\n\n{}", backend_instructions, system_msg.content);
        } else {
            messages.insert(
                0,
                AiMessage {
                    role: MessageRole::System,
                    content: backend_instructions.to_string(),
                },
            );
        }
    }

    fn estimate_cost(&self, provider_name: &str, model: &str, tokens_used: Option<i32>) -> i32 {
        let tokens = f64::from(tokens_used.unwrap_or(0));

        let cost_per_1k_tokens = self
            .providers
            .get(provider_name)
            .map_or(0.01, |provider| f64::from(provider.get_cost_per_1k_tokens(model)));

        ((tokens / 1000.0) * cost_per_1k_tokens * 1_000_000.0).round() as i32
    }
}
