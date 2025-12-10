//! AI Service
//!
//! Primary interface for AI generation, coordinating providers, tools, and
//! storage.

use anyhow::Result;
use futures::Stream;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::ai::{
    AiMessage, AiRequest, AiResponse, SamplingMetadata, SearchGroundedResponse,
};
use crate::models::tools::{CallToolResult, McpTool, ToolCall};
use crate::repository::AIRequestRepository;
use crate::services::config::{ConfigLoader, ConfigValidator};
use crate::services::mcp::{McpClientManager, ToolDiscovery};
use crate::services::providers::{AiProvider, ModelPricing, ProviderFactory};
use crate::services::sampling::SamplingRouter;
use crate::services::tooled::{ResponseStrategy, ResponseSynthesizer, TooledExecutor};

use super::request_logging;
use super::request_storage::RequestStorage;

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
    storage: RequestStorage,
    db_pool: systemprompt_core_database::DbPool,
    default_provider: String,
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

        let mut config = ConfigLoader::load_default()?;
        ConfigLoader::expand_env_vars(&mut config)?;
        ConfigValidator::validate(&config, &logger).await?;

        let providers = ProviderFactory::create_all(config.providers.clone(), Some(&db_pool))?;
        let default_provider = config.default_provider.clone();

        let sampling_router = Arc::new(SamplingRouter::new(
            providers.clone(),
            default_provider.clone(),
        ));

        let mcp_client_manager = Arc::new(McpClientManager::new(app_context.clone()));
        let tool_discovery = Arc::new(ToolDiscovery::new(mcp_client_manager.clone()));
        let tooled_executor = TooledExecutor::new(mcp_client_manager.clone());
        let synthesizer = ResponseSynthesizer::new();

        let storage = RequestStorage::new(
            AIRequestRepository::new(db_pool.clone()),
            AnalyticsSessionRepository::new(db_pool.clone()),
        );

        Ok(Self {
            providers,
            sampling_router,
            mcp_client_manager,
            tool_discovery,
            tooled_executor,
            synthesizer,
            storage,
            db_pool,
            default_provider,
        })
    }

    pub async fn generate(&self, request: AiRequest, ctx: RequestContext) -> Result<AiResponse> {
        let max_tokens = request.max_tokens.unwrap_or(4096);
        let sampling_request = request.with_context(&ctx).with_max_tokens(max_tokens);
        self.process_request(sampling_request, &ctx).await
    }

    pub async fn generate_direct(
        &self,
        provider: &str,
        model: &str,
        messages: Vec<AiMessage>,
        metadata: Option<SamplingMetadata>,
        ctx: RequestContext,
    ) -> Result<AiResponse> {
        let mut request = AiRequest::new(messages)
            .with_provider(provider)
            .with_model(model);

        if let Some(meta) = metadata {
            request = request.with_metadata(meta);
        }

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
        request: AiRequest,
        ctx: RequestContext,
    ) -> Result<AiResponse> {
        let enriched_request = request.with_context(&ctx);
        self.process_tooled_request(enriched_request, &ctx).await
    }

    pub async fn generate_single_turn(
        &self,
        request: AiRequest,
        ctx: &RequestContext,
    ) -> Result<(AiResponse, Vec<ToolCall>)> {
        let start = std::time::Instant::now();
        let request_id = Uuid::new_v4();
        let enriched_request = request.with_context(ctx);
        let (provider_name, provider, model) = self.select_provider_and_model(&enriched_request)?;

        let logger = LogService::new(self.db_pool.clone(), ctx.log_context());
        let tool_count = enriched_request
            .tools
            .as_ref()
            .map(|t| t.len())
            .unwrap_or(0);
        logger
            .info(
                "ai",
                &format!(
                    "AI request | request_id={}, provider={}, model={}, tools={}",
                    request_id, provider_name, model, tool_count
                ),
            )
            .await
            .ok();

        let metadata = enriched_request.metadata.clone();
        let result = self
            .call_ai_with_tools(provider.as_ref(), &enriched_request, &metadata, &model)
            .await;

        let latency_ms = start.elapsed().as_millis() as u64;
        self.finalize_single_turn(
            result,
            request_id,
            latency_ms,
            &enriched_request,
            ctx,
            &provider_name,
            &model,
        )
    }

    pub async fn execute_tools(
        &self,
        tool_calls: Vec<ToolCall>,
        tools: &[McpTool],
        ctx: &RequestContext,
        agent_overrides: Option<&systemprompt_models::ai::ToolModelOverrides>,
    ) -> (Vec<ToolCall>, Vec<CallToolResult>) {
        self.tooled_executor
            .execute_tool_calls(tool_calls, tools, ctx, agent_overrides)
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
            .values()
            .find(|p| p.supports_google_search())
            .ok_or_else(|| anyhow::anyhow!("No provider with Google Search support available"))?;

        let model = model.unwrap_or(provider.default_model());
        provider
            .generate_with_google_search(&messages, &metadata, model, urls, response_schema)
            .await
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
        request: AiRequest,
        ctx: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let (provider, model) = self.get_streaming_provider(&request)?;
        let enriched_request = request.with_context(&ctx);

        provider
            .generate_stream(
                &enriched_request.messages,
                &enriched_request.metadata,
                &model,
            )
            .await
    }

    pub async fn generate_with_tools_stream(
        &self,
        request: AiRequest,
        ctx: RequestContext,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let (provider, model) = self.get_streaming_provider(&request)?;
        let enriched_request = request.with_context(&ctx);
        let tools = enriched_request.tools.clone().unwrap_or_default();

        provider
            .generate_with_tools_stream(
                &enriched_request.messages,
                tools,
                &enriched_request.metadata,
                &model,
            )
            .await
    }
}

impl AiService {
    fn get_streaming_provider(&self, request: &AiRequest) -> Result<(Arc<dyn AiProvider>, String)> {
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
            .unwrap_or_else(|| provider.default_model())
            .to_string();

        Ok((provider.clone(), model))
    }

    fn finalize_single_turn(
        &self,
        result: Result<(AiResponse, Vec<ToolCall>)>,
        request_id: Uuid,
        latency_ms: u64,
        request: &AiRequest,
        ctx: &RequestContext,
        provider_name: &str,
        model: &str,
    ) -> Result<(AiResponse, Vec<ToolCall>)> {
        match result {
            Ok((mut response, tool_calls)) => {
                response.request_id = request_id;
                response.latency_ms = latency_ms;
                response.tool_calls = tool_calls.clone();

                let cost = self.estimate_cost(
                    &response.provider,
                    &response.model,
                    response.input_tokens.map(|t| t as i32),
                    response.output_tokens.map(|t| t as i32),
                );
                self.storage
                    .store(request, &response, ctx, "completed", None, cost);

                Ok((response, tool_calls))
            },
            Err(e) => {
                let error_response = AiResponse::new(
                    request_id,
                    String::new(),
                    provider_name.to_string(),
                    model.to_string(),
                )
                .with_latency(latency_ms);

                self.storage.store(
                    request,
                    &error_response,
                    ctx,
                    "failed",
                    Some(&e.to_string()),
                    0,
                );
                Err(e)
            },
        }
    }

    fn select_provider_and_model(
        &self,
        request: &AiRequest,
    ) -> Result<(String, Arc<dyn AiProvider>, String)> {
        let model_preferences = Self::build_model_preferences(request);

        let (provider_name, provider) = self
            .sampling_router
            .select_provider(&model_preferences, &request.metadata)?;

        let model = self
            .sampling_router
            .select_model(&provider_name, &model_preferences)?;

        Ok((provider_name, provider, model))
    }

    fn build_model_preferences(request: &AiRequest) -> crate::models::ai::ModelPreferences {
        let mut hints = Vec::new();
        if let Some(model) = &request.model {
            hints.push(crate::models::ai::ModelHint::ModelId(model.clone()));
        }
        if let Some(provider) = &request.provider {
            hints.push(crate::models::ai::ModelHint::Provider(provider.clone()));
        }
        crate::models::ai::ModelPreferences {
            hints,
            cost_priority: None,
        }
    }

    async fn call_ai_with_tools(
        &self,
        provider: &dyn AiProvider,
        request: &AiRequest,
        metadata: &SamplingMetadata,
        model: &str,
    ) -> Result<(AiResponse, Vec<ToolCall>)> {
        let tools = request.tools.clone().unwrap_or_default();
        provider
            .generate_with_tools(&request.messages, tools, metadata, model)
            .await
    }

    fn estimate_cost(
        &self,
        provider_name: &str,
        model: &str,
        input_tokens: Option<i32>,
        output_tokens: Option<i32>,
    ) -> i32 {
        let input = f64::from(input_tokens.unwrap_or(0));
        let output = f64::from(output_tokens.unwrap_or(0));

        let pricing = self
            .providers
            .get(provider_name)
            .map_or(ModelPricing::new(0.001, 0.001), |provider| {
                provider.get_pricing(model)
            });

        let input_cost = (input / 1000.0) * f64::from(pricing.input_cost_per_1k);
        let output_cost = (output / 1000.0) * f64::from(pricing.output_cost_per_1k);

        ((input_cost + output_cost) * 1_000_000.0).round() as i32
    }
}

impl AiService {
    async fn process_request(
        &self,
        request: AiRequest,
        ctx: &RequestContext,
    ) -> Result<AiResponse> {
        let request_id = Uuid::new_v4();
        let start = std::time::Instant::now();
        let logger = LogService::new(self.db_pool.clone(), ctx.log_context());

        let (provider_name, provider, model) = self.select_provider_and_model(&request)?;
        request_logging::log_request_start(
            &logger,
            request_id,
            &request,
            &provider_name,
            &model,
            ctx,
        )
        .await;

        let sample_result = self
            .execute_sampling(&request, provider.as_ref(), &model)
            .await;
        let latency_ms = start.elapsed().as_millis() as u64;

        self.finalize_request(
            sample_result,
            request_id,
            latency_ms,
            &request,
            ctx,
            &provider_name,
            &model,
            &logger,
        )
        .await
    }

    async fn execute_sampling(
        &self,
        request: &AiRequest,
        provider: &dyn AiProvider,
        model: &str,
    ) -> Result<AiResponse> {
        if let Some(format) = request.response_format() {
            self.execute_structured_sampling(request, provider, model, format)
                .await
        } else {
            provider
                .generate(&request.messages, &request.metadata, model)
                .await
        }
    }

    async fn execute_structured_sampling(
        &self,
        request: &AiRequest,
        provider: &dyn AiProvider,
        model: &str,
        format: &crate::models::ai::ResponseFormat,
    ) -> Result<AiResponse> {
        if provider.supports_json_mode() || provider.supports_structured_output() {
            return provider
                .generate_structured(&request.messages, &request.metadata, model, format)
                .await;
        }

        let options = request.structured_output.clone().unwrap_or_default();
        let enhanced_messages =
            Self::enhance_messages_for_json(&request.messages, format, &options);

        provider
            .generate(&enhanced_messages, &request.metadata, model)
            .await
    }

    fn enhance_messages_for_json(
        messages: &[AiMessage],
        format: &crate::models::ai::ResponseFormat,
        options: &crate::models::ai::StructuredOutputOptions,
    ) -> Vec<AiMessage> {
        use crate::services::structured_output::StructuredOutputProcessor;

        let Some(last_msg) = messages.last() else {
            return messages.to_vec();
        };

        let mut enhanced = messages[..messages.len() - 1].to_vec();
        let enhanced_content =
            StructuredOutputProcessor::enhance_prompt_for_json(&last_msg.content, format, options);
        enhanced.push(AiMessage {
            role: last_msg.role,
            content: enhanced_content,
        });
        enhanced
    }

    async fn finalize_request(
        &self,
        result: Result<AiResponse>,
        request_id: Uuid,
        latency_ms: u64,
        request: &AiRequest,
        ctx: &RequestContext,
        provider_name: &str,
        model: &str,
        logger: &LogService,
    ) -> Result<AiResponse> {
        match result {
            Ok(mut response) => {
                response = self
                    .validate_structured_output(response, request, logger)
                    .await;
                response.request_id = request_id;
                response.latency_ms = latency_ms;

                let cost = self.estimate_cost(
                    &response.provider,
                    &response.model,
                    response.input_tokens.map(|t| t as i32),
                    response.output_tokens.map(|t| t as i32),
                );
                self.storage
                    .store(request, &response, ctx, "completed", None, cost);
                request_logging::log_request_success(logger, &response).await;

                Ok(response)
            },
            Err(e) => {
                let error_response = AiResponse::new(
                    request_id,
                    String::new(),
                    provider_name.to_string(),
                    model.to_string(),
                )
                .with_latency(latency_ms);

                self.storage.store(
                    request,
                    &error_response,
                    ctx,
                    "failed",
                    Some(&e.to_string()),
                    0,
                );
                request_logging::log_request_error(
                    logger,
                    request_id,
                    provider_name,
                    latency_ms,
                    &e,
                )
                .await;

                Err(e)
            },
        }
    }

    async fn validate_structured_output(
        &self,
        mut response: AiResponse,
        request: &AiRequest,
        logger: &LogService,
    ) -> AiResponse {
        let (Some(format), Some(ref options)) =
            (request.response_format(), &request.structured_output)
        else {
            return response;
        };

        if !format.is_json() {
            return response;
        }

        use crate::services::structured_output::StructuredOutputProcessor;
        match StructuredOutputProcessor::process_response(&response.content, format, options) {
            Ok(json_value) => {
                if let Ok(formatted) = serde_json::to_string(&json_value) {
                    response.content = formatted;
                }
            },
            Err(e) => {
                logger
                    .warn(
                        "ai",
                        &format!("Structured output validation failed | error={e}"),
                    )
                    .await
                    .ok();
            },
        }

        response
    }
}

impl AiService {
    pub async fn generate_plan(
        &self,
        request: AiRequest,
        available_tools: &[McpTool],
        ctx: &RequestContext,
    ) -> Result<systemprompt_models::ai::PlanningResult> {
        self.generate_plan_with_model(request, available_tools, ctx, None, None)
            .await
    }

    /// Generate a plan using native function calling with AUTO mode.
    /// The model decides whether to call tools or respond directly with text.
    pub async fn generate_plan_with_model(
        &self,
        request: AiRequest,
        available_tools: &[McpTool],
        ctx: &RequestContext,
        provider_override: Option<&str>,
        model_override: Option<&str>,
    ) -> Result<systemprompt_models::ai::PlanningResult> {
        let mut request = request;
        if let Some(p) = provider_override {
            request = request.with_provider(p.to_string());
        }
        if let Some(m) = model_override {
            request = request.with_model(m.to_string());
        }

        let start = std::time::Instant::now();
        let request_id = Uuid::new_v4();
        let logger = LogService::new(self.db_pool.clone(), ctx.log_context());

        let (provider_name, provider, model) = self.select_provider_and_model(&request)?;

        logger
            .info(
                "ai",
                &format!(
                    "Planning request (function calling) | request_id={}, provider={}, model={}, \
                     available_tools={}",
                    request_id,
                    provider_name,
                    model,
                    available_tools.len()
                ),
            )
            .await
            .ok();

        let result = provider
            .generate_with_tools(
                &request.messages,
                available_tools.to_vec(),
                &request.metadata,
                &model,
            )
            .await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok((mut response, tool_calls)) => {
                response.request_id = request_id;
                response.latency_ms = latency_ms;

                let cost = self.estimate_cost(
                    &provider_name,
                    &model,
                    response.input_tokens.map(|t| t as i32),
                    response.output_tokens.map(|t| t as i32),
                );
                self.storage
                    .store(&request, &response, ctx, "completed", None, cost);

                let planning_result = if tool_calls.is_empty() {
                    systemprompt_models::ai::PlanningResult::DirectResponse {
                        content: response.content.clone(),
                    }
                } else {
                    systemprompt_models::ai::PlanningResult::ToolCalls {
                        reasoning: response.content.clone(),
                        calls: tool_calls
                            .into_iter()
                            .map(|tc| systemprompt_models::ai::PlannedToolCall {
                                tool_name: tc.name,
                                arguments: tc.arguments,
                            })
                            .collect(),
                    }
                };

                logger
                    .info(
                        "ai",
                        &format!(
                            "Planning complete (function calling) | request_id={}, latency={}ms, \
                             type={}",
                            request_id,
                            latency_ms,
                            if planning_result.is_direct() {
                                "direct_response"
                            } else {
                                "tool_calls"
                            }
                        ),
                    )
                    .await
                    .ok();

                Ok(planning_result)
            },
            Err(e) => {
                let error_response = AiResponse::new(
                    request_id,
                    String::new(),
                    provider_name.to_string(),
                    model.to_string(),
                )
                .with_latency(latency_ms);

                self.storage.store(
                    &request,
                    &error_response,
                    ctx,
                    "failed",
                    Some(&e.to_string()),
                    0,
                );

                logger
                    .error(
                        "ai",
                        &format!(
                            "Planning failed (function calling) | request_id={}, latency={}ms, \
                             error={}",
                            request_id, latency_ms, e
                        ),
                    )
                    .await
                    .ok();
                Err(e)
            },
        }
    }

    pub async fn generate_response(
        &self,
        messages: Vec<AiMessage>,
        execution_summary: &str,
        ctx: &RequestContext,
    ) -> Result<String> {
        self.generate_response_with_model(messages, execution_summary, ctx, None, None)
            .await
    }

    pub async fn generate_response_with_model(
        &self,
        messages: Vec<AiMessage>,
        execution_summary: &str,
        ctx: &RequestContext,
        provider: Option<&str>,
        model: Option<&str>,
    ) -> Result<String> {
        let start = std::time::Instant::now();
        let logger = LogService::new(self.db_pool.clone(), ctx.log_context());

        let mut response_messages = messages;
        response_messages.push(AiMessage::user(format!(
            "## Tool Execution Complete\n\nThe following tools have been executed:\n\n{}\n\n## \
             Response Phase Instructions\n\nThis is the RESPONSE PHASE - your task is to \
             synthesize results and respond to the user.\n\n**CRITICAL: Do NOT attempt to call \
             any tools.** Tools are not available in this phase.\nThe tool results above are now \
             saved in conversation history and will be available for future planning phases if \
             additional tool calls are needed.\n\nYour task now: Analyze the tool results and \
             provide a helpful response to the user. If the results indicate more actions are \
             needed, explain what was accomplished and what the next steps would be - the user \
             can then request those actions in a follow-up message.",
            execution_summary
        )));

        let mut request = AiRequest::new(response_messages).with_max_tokens(4096);
        if let Some(p) = provider {
            request = request.with_provider(p.to_string());
        }
        if let Some(m) = model {
            request = request.with_model(m.to_string());
        }
        let response = self.generate(request, ctx.clone()).await?;

        let latency_ms = start.elapsed().as_millis() as u64;
        logger
            .info(
                "ai",
                &format!("Response generated | latency={}ms", latency_ms),
            )
            .await
            .ok();

        Ok(response.content)
    }
}

impl AiService {
    async fn process_tooled_request(
        &self,
        request: AiRequest,
        ctx: &RequestContext,
    ) -> Result<AiResponse> {
        let request_id = Uuid::new_v4();
        let start = std::time::Instant::now();
        let logger = LogService::new(self.db_pool.clone(), ctx.log_context());

        let (provider_name, provider, model) = self.select_provider_and_model(&request)?;
        request_logging::log_tooled_request_start(
            &logger,
            request_id,
            &request,
            &provider_name,
            &model,
            ctx,
        )
        .await;

        let metadata = request.metadata.clone();
        let tools = request.tools.as_deref().unwrap_or(&[]);

        let ai_result = self
            .call_ai_with_tools(provider.as_ref(), &request, &metadata, &model)
            .await;
        let latency_ms = start.elapsed().as_millis() as u64;

        self.finalize_tooled_request(
            ai_result,
            request_id,
            latency_ms,
            &request,
            ctx,
            &provider_name,
            &model,
            provider.as_ref(),
            &metadata,
            tools,
            &logger,
        )
        .await
    }

    async fn finalize_tooled_request(
        &self,
        ai_result: Result<(AiResponse, Vec<ToolCall>)>,
        request_id: Uuid,
        latency_ms: u64,
        request: &AiRequest,
        ctx: &RequestContext,
        provider_name: &str,
        model: &str,
        provider: &dyn AiProvider,
        metadata: &SamplingMetadata,
        tools: &[McpTool],
        logger: &LogService,
    ) -> Result<AiResponse> {
        let (response, tool_calls) = match ai_result {
            Ok(result) => result,
            Err(e) => {
                let error_response = AiResponse::new(
                    request_id,
                    String::new(),
                    provider_name.to_string(),
                    model.to_string(),
                )
                .with_latency(latency_ms);
                self.storage.store(
                    request,
                    &error_response,
                    ctx,
                    "failed",
                    Some(&e.to_string()),
                    0,
                );
                return Err(e);
            },
        };

        request_logging::log_ai_response(logger, &response, tool_calls.len()).await;

        let (tool_calls, tool_results) = self
            .tooled_executor
            .execute_tool_calls(tool_calls, tools, ctx, None)
            .await;
        let final_content = self
            .determine_final_content(
                &response,
                &tool_calls,
                &tool_results,
                provider,
                metadata,
                model,
                logger,
            )
            .await;

        let cost = self.estimate_cost(
            provider_name,
            model,
            response.input_tokens.map(|t| t as i32),
            response.output_tokens.map(|t| t as i32),
        );
        let mut storage_response = response.clone();
        storage_response.request_id = request_id;
        storage_response.latency_ms = latency_ms;
        storage_response.tool_calls = tool_calls.clone();
        storage_response.tool_results = tool_results.clone();
        self.storage
            .store(request, &storage_response, ctx, "completed", None, cost);

        let final_response = AiResponse::new(
            request_id,
            final_content,
            provider_name.to_string(),
            model.to_string(),
        )
        .with_latency(latency_ms)
        .with_tool_calls(tool_calls)
        .with_tool_results(tool_results);

        request_logging::log_tooled_response(logger, &final_response).await;
        Ok(final_response)
    }

    async fn determine_final_content(
        &self,
        response: &AiResponse,
        tool_calls: &[ToolCall],
        tool_results: &[CallToolResult],
        provider: &dyn AiProvider,
        metadata: &SamplingMetadata,
        model: &str,
        logger: &LogService,
    ) -> String {
        let strategy = ResponseStrategy::from_response(
            response.content.clone(),
            tool_calls.to_vec(),
            tool_results.to_vec(),
        );

        match strategy {
            ResponseStrategy::ContentProvided { content, .. } => content,
            ResponseStrategy::ArtifactsProvided { .. } => String::new(),
            ResponseStrategy::ToolsOnly {
                tool_calls,
                tool_results,
            } => {
                self.synthesizer
                    .synthesize_or_fallback(
                        provider,
                        &[],
                        &tool_calls,
                        &tool_results,
                        metadata,
                        model,
                        logger,
                    )
                    .await
            },
        }
    }
}
