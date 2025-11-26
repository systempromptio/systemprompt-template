use anyhow::{Context, Result};
use rmcp::{
    handler::client::progress::ProgressDispatcher,
    model::{
        ClientCapabilities, ClientInfo, Implementation, ProgressNotificationParam, ProtocolVersion,
    },
    service::NotificationContext,
    transport::streamable_http_client::{
        StreamableHttpClientTransport, StreamableHttpClientTransportConfig,
    },
    ClientHandler, RoleClient, ServiceExt,
};
use std::time::Duration;
use systemprompt_models::ai::tools::McpTool;
use tokio::time::timeout;

mod http_client_with_context;
use http_client_with_context::HttpClientWithContext;

use crate::repository::tool_usage_repository::{
    ToolExecutionRequest, ToolExecutionResult, ToolUsageRepository,
};
use systemprompt_core_database::DbPool;

#[derive(Clone)]
pub struct McpClientHandler {
    progress_dispatcher: ProgressDispatcher,
    client_info: ClientInfo,
}

impl McpClientHandler {
    pub fn new(client_info: ClientInfo) -> Self {
        Self {
            progress_dispatcher: ProgressDispatcher::new(),
            client_info,
        }
    }

    pub const fn progress_dispatcher(&self) -> &ProgressDispatcher {
        &self.progress_dispatcher
    }
}

impl ClientHandler for McpClientHandler {
    async fn on_progress(
        &self,
        params: ProgressNotificationParam,
        _context: NotificationContext<RoleClient>,
    ) {
        tracing::info!(
            "MCP Progress: {:.1}% - {}",
            (params.progress / params.total.unwrap_or(100.0)) * 100.0,
            params.message.as_deref().unwrap_or("")
        );
        self.progress_dispatcher.handle_notification(params).await;
    }

    fn get_info(&self) -> ClientInfo {
        self.client_info.clone()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct McpClient;

impl McpClient {
    fn rewrite_url_for_internal_use(url: &str) -> Result<String> {
        use systemprompt_core_system::Config;

        let config = Config::global();
        let external_url = &config.api_external_url;
        let internal_url = &config.api_server_url;

        if url.starts_with(external_url) {
            Ok(url.replace(external_url, internal_url))
        } else {
            Ok(url.to_string())
        }
    }

    pub async fn validate_connection(
        service_name: &str,
        host: &str,
        port: u16,
    ) -> Result<McpConnectionResult> {
        let connection_start = std::time::Instant::now();
        let url = format!("http://{host}:{port}/mcp");

        let connection_result = timeout(
            Duration::from_secs(15),
            Self::connect_and_validate(&url, service_name),
        )
        .await;

        let connection_time = connection_start.elapsed().as_millis() as u32;

        match connection_result {
            Ok(Ok((server_info, validation_result))) => Ok(McpConnectionResult {
                service_name: service_name.to_string(),
                success: validation_result.success,
                error_message: validation_result.error_message,
                connection_time_ms: connection_time,
                server_info: Some(server_info),
                tools_count: validation_result.tools_count,
                validation_type: validation_result.validation_type,
            }),
            Ok(Err(e)) => Ok(McpConnectionResult {
                service_name: service_name.to_string(),
                success: false,
                error_message: Some(e.to_string()),
                connection_time_ms: connection_time,
                server_info: None,
                tools_count: 0,
                validation_type: "connection_failed".to_string(),
            }),
            Err(_) => Ok(McpConnectionResult {
                service_name: service_name.to_string(),
                success: false,
                error_message: Some("Connection timeout".to_string()),
                connection_time_ms: connection_time,
                server_info: None,
                tools_count: 0,
                validation_type: "timeout".to_string(),
            }),
        }
    }

    pub async fn validate_connection_with_auth(
        service_name: &str,
        host: &str,
        port: u16,
        requires_oauth: bool,
    ) -> Result<McpConnectionResult> {
        if requires_oauth {
            // For OAuth services, just check port availability and return auth_required status
            Self::validate_oauth_service(service_name, host, port).await
        } else {
            // For non-OAuth services, do full MCP validation
            Self::validate_connection(service_name, host, port).await
        }
    }

    async fn validate_oauth_service(
        service_name: &str,
        host: &str,
        port: u16,
    ) -> Result<McpConnectionResult> {
        let connection_start = std::time::Instant::now();

        // Just check if port is responding
        let port_check = std::net::TcpStream::connect(format!("{host}:{port}"));
        let connection_time = connection_start.elapsed().as_millis() as u32;

        match port_check {
            Ok(_) => Ok(McpConnectionResult {
                service_name: service_name.to_string(),
                success: true, // Port is responding
                error_message: None,
                connection_time_ms: connection_time,
                server_info: Some(McpProtocolInfo {
                    server_name: service_name.to_string(),
                    version: "unknown".to_string(),
                    protocol_version: "unknown".to_string(),
                }),
                tools_count: 0,
                validation_type: "auth_required".to_string(),
            }),
            Err(e) => Ok(McpConnectionResult {
                service_name: service_name.to_string(),
                success: false,
                error_message: Some(format!("Port not responding: {e}")),
                connection_time_ms: connection_time,
                server_info: None,
                tools_count: 0,
                validation_type: "port_unavailable".to_string(),
            }),
        }
    }

    async fn connect_and_validate(
        url: &str,
        service_name: &str,
    ) -> Result<(McpProtocolInfo, ValidationResult)> {
        // Create transport using reqwest client with from_uri method
        let transport = StreamableHttpClientTransport::from_uri(url);

        let client_info = ClientInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: format!("SystemPrompt MCP Validator for {service_name}"),
                title: None,
                version: "1.0.0".to_string(),
                website_url: None,
                icons: None,
            },
        };

        let client = client_info.serve(transport).await?;

        // Get server information and extract the data we need
        let server_info = {
            let peer_info = client.peer_info().unwrap();
            McpProtocolInfo {
                server_name: if peer_info.server_info.name.is_empty() {
                    service_name.to_string()
                } else {
                    peer_info.server_info.name.clone()
                },
                version: if peer_info.server_info.version.is_empty() {
                    "1.0.0".to_string()
                } else {
                    peer_info.server_info.version.clone()
                },
                protocol_version: peer_info.protocol_version.to_string(),
            }
        };

        // CRITICAL: Actually validate that tools are returned
        let validation_result = match client.list_tools(None).await {
            Ok(tools_response) => {
                let tools_count = tools_response.tools.len();

                if tools_count > 0 {
                    ValidationResult {
                        success: true,
                        error_message: None,
                        tools_count,
                        validation_type: "mcp_validated".to_string(),
                    }
                } else {
                    ValidationResult {
                        success: false,
                        error_message: Some(
                            "No tools returned - service may require authentication".to_string(),
                        ),
                        tools_count: 0,
                        validation_type: "no_tools".to_string(),
                    }
                }
            },
            Err(e) => ValidationResult {
                success: false,
                error_message: Some(format!("Tools request failed: {e}")),
                tools_count: 0,
                validation_type: "tools_request_failed".to_string(),
            },
        };

        client.cancel().await?;
        Ok((server_info, validation_result))
    }

    pub async fn list_tools(
        service_id: impl Into<String>,
        context: &systemprompt_core_system::RequestContext,
    ) -> Result<Vec<McpTool>> {
        use crate::services::registry::RegistryManager;

        let service_id = service_id.into();

        // Get server config from registry (same as call_tool)
        let registry = RegistryManager::new().await?;
        let server_config = registry
            .get_server_by_name(&service_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("MCP server '{service_id}' not found in registry"))?;

        // Use endpoint from config and rewrite for internal use (same as call_tool)
        let url = server_config.endpoint();
        let url = Self::rewrite_url_for_internal_use(&url)?;
        let requires_auth = server_config.oauth.required;

        // Create transport with HttpClientWithContext wrapper (same as call_tool)
        let client = HttpClientWithContext::new(context.clone());
        let transport = if requires_auth {
            let user_token = context.auth_token();
            if user_token.as_str().is_empty() {
                return Err(anyhow::anyhow!(
                    "User JWT required for authenticated MCP calls"
                ));
            }
            let config = StreamableHttpClientTransportConfig::with_uri(url.as_str())
                .auth_header(format!("Bearer {}", user_token.as_str()));
            StreamableHttpClientTransport::with_client(client, config)
        } else {
            let config = StreamableHttpClientTransportConfig::with_uri(url.as_str());
            StreamableHttpClientTransport::with_client(client, config)
        };

        let client_info = ClientInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: "systemprompt-mcp-client".to_string(),
                title: None,
                version: "1.0.0".to_string(),
                website_url: None,
                icons: None,
            },
        };

        // Initialize the service
        let client = client_info.serve(transport).await?;

        // List tools
        let tools_response = client.list_tools(None).await?;

        let mut tools = Vec::new();
        for tool in tools_response.tools {
            let input_schema = serde_json::to_value(tool.input_schema).with_context(|| {
                format!("Failed to serialize input schema for tool '{}'", tool.name)
            })?;

            let output_schema = tool
                .output_schema
                .map(|schema| {
                    serde_json::to_value(schema.as_ref()).with_context(|| {
                        format!("Failed to serialize output schema for tool '{}'", tool.name)
                    })
                })
                .transpose()?;

            tools.push(McpTool {
                name: tool.name.to_string(),
                description: tool.description.map(|d| d.to_string()),
                input_schema: Some(input_schema),
                output_schema,
                service_id: service_id.clone(),
            });
        }

        client.cancel().await?;
        Ok(tools)
    }

    pub async fn call_tool(
        service_name: &str,
        name: String,
        arguments: Option<serde_json::Value>,
        context: &systemprompt_core_system::RequestContext,
        db_pool: &DbPool,
    ) -> Result<rmcp::model::CallToolResult> {
        use crate::services::registry::RegistryManager;

        let registry = RegistryManager::new().await?;
        let server_config = registry
            .get_server_by_name(service_name)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!("MCP server '{service_name}' not found in registry")
            })?;

        let url = server_config.endpoint();
        let requires_auth = server_config.oauth.required;

        let url = Self::rewrite_url_for_internal_use(&url)?;

        let tool_repo = ToolUsageRepository::new(db_pool.clone());
        let started_at = chrono::Utc::now();

        let request = ToolExecutionRequest {
            tool_name: name.clone(),
            mcp_server_name: service_name.to_string(),
            input: arguments.clone().unwrap_or(serde_json::json!({})),
            started_at,
            context: context.clone(),
            request_method: Some("mcp".to_string()),
            request_source: Some("ai_service".to_string()),
            ai_tool_call_id: context.ai_tool_call_id().cloned(),
        };

        let mcp_execution_id = tool_repo.start_execution(&request).await?;

        let client = HttpClientWithContext::new(context.clone());
        let transport = if requires_auth {
            let user_token = context.auth_token();
            if user_token.as_str().is_empty() {
                return Err(anyhow::anyhow!(
                    "User JWT required for authenticated MCP calls"
                ));
            }
            let config = StreamableHttpClientTransportConfig::with_uri(url.as_str())
                .auth_header(format!("Bearer {}", user_token.as_str()));
            StreamableHttpClientTransport::with_client(client, config)
        } else {
            let config = StreamableHttpClientTransportConfig::with_uri(url.as_str());
            StreamableHttpClientTransport::with_client(client, config)
        };

        let client_info = ClientInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: "systemprompt-ai-mcp-client".to_string(),
                title: None,
                version: "1.0.0".to_string(),
                website_url: None,
                icons: None,
            },
        };

        let handler = McpClientHandler::new(client_info);

        let client_service = match timeout(Duration::from_secs(30), handler.serve(transport)).await
        {
            Ok(Ok(c)) => c,
            Ok(Err(e)) => return Err(e.into()),
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "MCP transport serve timed out after 30 seconds"
                ))
            },
        };

        let params = rmcp::model::CallToolRequestParam {
            name: name.clone().into(),
            arguments: arguments.and_then(|v| v.as_object().cloned()),
        };

        let tool_result: Result<systemprompt_models::CallToolResult, anyhow::Error> =
            client_service
                .call_tool(params)
                .await
                .map_err(|e| anyhow::anyhow!("MCP tool call failed: {e}"));

        client_service.cancel().await?;

        let result = match &tool_result {
            Ok(res) => ToolExecutionResult {
                output: Some(serde_json::to_value(&res.content).unwrap_or(serde_json::json!({}))),
                output_schema: None,
                status: "success".to_string(),
                error_message: None,
                completed_at: chrono::Utc::now(),
            },
            Err(e) => ToolExecutionResult {
                output: None,
                output_schema: None,
                status: "failed".to_string(),
                error_message: Some(e.to_string()),
                completed_at: chrono::Utc::now(),
            },
        };

        tool_repo
            .complete_execution(&mcp_execution_id, &result)
            .await?;

        tool_result.map_err(|e| anyhow::anyhow!("Tool execution failed: {e}"))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpConnectionResult {
    pub service_name: String,
    pub success: bool,
    pub error_message: Option<String>,
    pub connection_time_ms: u32,
    pub server_info: Option<McpProtocolInfo>,
    pub tools_count: usize,
    pub validation_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpProtocolInfo {
    pub server_name: String,
    pub version: String,
    pub protocol_version: String,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub success: bool,
    pub error_message: Option<String>,
    pub tools_count: usize,
    pub validation_type: String,
}

#[derive(Debug, Clone)]
pub struct ToolExecutionWithId {
    pub result: rmcp::model::CallToolResult,
    pub mcp_execution_id: String,
}

impl McpConnectionResult {
    pub const fn is_healthy(&self) -> bool {
        self.success && self.connection_time_ms < 2000
    }

    pub fn health_status(&self) -> &'static str {
        match self.validation_type.as_str() {
            "mcp_validated" => {
                if self.connection_time_ms < 1000 {
                    "healthy"
                } else {
                    "slow"
                }
            },
            "auth_required" | "no_tools" => "auth_required",
            "tools_request_failed"
            | "connection_failed"
            | "port_unavailable"
            | "timeout" => "unhealthy",
            _ => "unknown",
        }
    }

    pub fn status_description(&self) -> String {
        match self.validation_type.as_str() {
            "mcp_validated" => format!("MCP validated with {} tools", self.tools_count),
            "auth_required" => "Port responding, OAuth authentication required".to_string(),
            "no_tools" => "Connected but no tools returned (likely requires auth)".to_string(),
            "tools_request_failed" => {
                let error = self
                    .error_message
                    .as_deref()
                    .filter(|e| !e.is_empty())
                    .unwrap_or("[no error message]");
                format!("Tools request failed: {error}")
            },
            "connection_failed" => {
                let error = self
                    .error_message
                    .as_deref()
                    .filter(|e| !e.is_empty())
                    .unwrap_or("[no error message]");
                format!("Connection failed: {error}")
            },
            "port_unavailable" => "Port not responding".to_string(),
            "timeout" => "Connection timeout".to_string(),
            _ => "Unknown validation result".to_string(),
        }
    }
}
