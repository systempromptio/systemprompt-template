use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::tools::{CallToolResult, McpTool, ToolCall};
use crate::repository::AIRequestRepository;
use systemprompt_core_logging::LogService;
use systemprompt_core_mcp::services::client::{validate_connection, McpClient};
use systemprompt_core_mcp::services::deployment::DeploymentService;
use systemprompt_core_oauth::services::validation::jwt::validate_jwt_token;
use systemprompt_core_system::AppContext;
use systemprompt_models::repository::ServiceRepository;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpService {
    pub name: String,
    pub port: u16,
    pub status: String,
}

#[derive(Debug, Clone)]
struct ServiceConnection {
    service: McpService,
    _last_validated: std::time::Instant,
}

#[derive(Debug)]
pub struct McpClientManager {
    services: Arc<RwLock<HashMap<String, ServiceConnection>>>,
    app_context: Arc<AppContext>,
}

impl McpClientManager {
    pub fn new(app_context: Arc<AppContext>) -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            app_context,
        }
    }

    pub async fn connect_to_service(&self, service: &McpService) -> Result<()> {
        let logger = LogService::system(self.app_context.db_pool().clone());

        let connection_result =
            validate_connection(&service.name, "127.0.0.1", service.port).await?;

        if !connection_result.success {
            return Err(anyhow!(
                "Failed to connect to MCP service {}: {}",
                service.name,
                connection_result.error_message.unwrap_or_default()
            ));
        }

        let mut services = self.services.write().await;
        services.insert(
            service.name.clone(),
            ServiceConnection {
                service: service.clone(),
                _last_validated: std::time::Instant::now(),
            },
        );

        logger
            .info(
                "ai_mcp_client",
                &format!(
                    "Connected to MCP service {} at 127.0.0.1:{}",
                    service.name, service.port
                ),
            )
            .await
            .ok();
        Ok(())
    }

    pub async fn disconnect_from_service(&self, service_name: &str) -> Result<()> {
        let logger = LogService::system(self.app_context.db_pool().clone());

        let mut services = self.services.write().await;
        services.remove(service_name);

        logger
            .info(
                "ai_mcp_client",
                &format!("Disconnected from MCP service {service_name}"),
            )
            .await
            .ok();
        Ok(())
    }

    pub async fn list_tools_for_agent(
        &self,
        agent_name: &systemprompt_identifiers::AgentName,
        context: &systemprompt_core_system::RequestContext,
    ) -> Result<Vec<McpTool>> {
        let logger = LogService::system(self.app_context.db_pool().clone());

        let assigned_servers = self.load_agent_mcp_servers(agent_name).await?;

        logger
            .info(
                "ai_mcp_client",
                &format!(
                    "Listing tools for agent '{}' from servers: {}",
                    agent_name.as_str(),
                    assigned_servers.join(", ")
                ),
            )
            .await
            .ok();

        let deployment_config = DeploymentService::load_config().await?;
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "dev-secret-key-change-in-production".to_string());

        let user_permissions = match validate_jwt_token(context.auth_token().as_str(), &jwt_secret)
        {
            Ok(claims) => claims.get_permissions(),
            Err(e) => {
                logger
                    .error(
                        "ai_mcp_client",
                        &format!("JWT validation failed: {e}. No tools will be loaded."),
                    )
                    .await
                    .ok();
                return Ok(vec![]);
            },
        };

        let services = self.services.read().await;
        let mut agent_tools = Vec::new();

        for (service_name, _connection) in services.iter() {
            if !assigned_servers.contains(service_name) {
                continue;
            }

            if let Some(deployment) = deployment_config.mcp_servers.get(service_name) {
                if deployment.oauth.required && !deployment.oauth.scopes.is_empty() {
                    let has_permission = deployment
                        .oauth
                        .scopes
                        .iter()
                        .any(|required_scope| user_permissions.contains(required_scope));

                    if !has_permission {
                        continue;
                    }
                }
            }

            match McpClient::list_tools(service_name, context).await {
                Ok(tools) => {
                    logger
                        .info(
                            "ai_mcp_client",
                            &format!(
                                "Loaded {} tools from {} for agent {}",
                                tools.len(),
                                service_name,
                                agent_name.as_str()
                            ),
                        )
                        .await
                        .ok();
                    agent_tools.extend(tools);
                },
                Err(e) => {
                    logger
                        .warn(
                            "ai_mcp_client",
                            &format!("Failed to list tools from service {service_name}: {e}"),
                        )
                        .await
                        .ok();
                },
            }
        }

        logger
            .info(
                "ai_mcp_client",
                &format!(
                    "Total {} tools loaded for agent '{}'",
                    agent_tools.len(),
                    agent_name.as_str()
                ),
            )
            .await
            .ok();

        Ok(agent_tools)
    }

    pub async fn execute_tool(
        &self,
        tool_call: &ToolCall,
        service_name: &str,
        context: &systemprompt_core_system::RequestContext,
    ) -> Result<CallToolResult> {
        let enriched_context = context
            .clone()
            .with_ai_tool_call_id(tool_call.ai_tool_call_id.clone());

        let result = McpClient::call_tool(
            service_name,
            tool_call.name.clone(),
            Some(tool_call.arguments.clone()),
            &enriched_context,
            self.app_context.db_pool(),
        )
        .await
        .map_err(|e| anyhow!("Failed to execute tool {}: {}", tool_call.name, e))?;

        let ai_request_repo = AIRequestRepository::new(self.app_context.db_pool().clone());
        let logger = LogService::system(self.app_context.db_pool().clone());

        let ai_tool_call_id_str = tool_call.ai_tool_call_id.as_ref();
        match ai_request_repo
            .link_tool_calls_to_recent_executions(&[ai_tool_call_id_str.to_string()])
            .await
        {
            Ok(count) if count > 0 => {
                logger
                    .info(
                        "ai_mcp_client",
                        &format!("Linked AI tool call {ai_tool_call_id_str} to MCP execution"),
                    )
                    .await
                    .ok();
            },
            Ok(_) => {
                logger
                    .info(
                        "ai_mcp_client",
                        &format!("No AI request found to link for tool call {ai_tool_call_id_str}"),
                    )
                    .await
                    .ok();
            },
            Err(e) => {
                logger
                    .warn(
                        "ai_mcp_client",
                        &format!("Failed to link tool call {ai_tool_call_id_str}: {e}"),
                    )
                    .await
                    .ok();
            },
        }

        Ok(result)
    }

    pub async fn refresh_connections_for_agent(
        &self,
        agent_name: &systemprompt_identifiers::AgentName,
    ) -> Result<()> {
        let logger = LogService::system(self.app_context.db_pool().clone());

        logger
            .info(
                "ai_mcp_client",
                &format!("🔌 MCP CLIENT: Refreshing connections for agent: {agent_name}"),
            )
            .await
            .ok();

        let services = self
            .get_active_services_for_agent(Some(agent_name.as_str()))
            .await?;

        logger
            .info(
                "ai_mcp_client",
                &format!(
                    "🔌 MCP CLIENT: Found {} active MCP services for agent {}: {}",
                    services.len(),
                    agent_name,
                    services
                        .iter()
                        .map(|s| s.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
            .await
            .ok();

        for service in services {
            if self.is_connected(&service.name).await {
                logger
                    .info(
                        "ai_mcp_client",
                        &format!(
                            "🔌 MCP CLIENT: Already connected to service {}",
                            service.name
                        ),
                    )
                    .await
                    .ok();
            } else {
                logger
                    .info(
                        "ai_mcp_client",
                        &format!(
                            "🔌 MCP CLIENT: Connecting to service {} (127.0.0.1:{})",
                            service.name, service.port
                        ),
                    )
                    .await
                    .ok();

                if let Err(e) = self.connect_to_service(&service).await {
                    logger
                        .warn(
                            "ai_mcp_client",
                            &format!("Failed to connect to service {}: {}", service.name, e),
                        )
                        .await
                        .ok();
                }
            }
        }

        Ok(())
    }

    pub async fn is_connected(&self, service_name: &str) -> bool {
        let services = self.services.read().await;
        services.contains_key(service_name)
    }

    async fn get_active_services_for_agent(
        &self,
        agent_id: Option<&str>,
    ) -> Result<Vec<McpService>> {
        let logger = LogService::system(self.app_context.db_pool().clone());
        let service_repo = ServiceRepository::new(self.app_context.db_pool().clone());

        let all_services = service_repo.get_mcp_services().await?;

        let services = if let Some(agent_name) = agent_id {
            let agent_name_type = systemprompt_identifiers::AgentName::new(agent_name);
            let assigned_servers = self.load_agent_mcp_servers(&agent_name_type).await?;

            logger
                .info(
                    "ai_mcp_client",
                    &format!(
                        "🔍 Agent '{}' assigned MCP servers: {}",
                        agent_name,
                        assigned_servers.join(", ")
                    ),
                )
                .await
                .ok();

            all_services
                .into_iter()
                .filter(|svc| {
                    let is_running = svc.status == "running";
                    let is_assigned = assigned_servers.contains(&svc.name);
                    is_running && is_assigned
                })
                .map(|active_svc| McpService {
                    name: active_svc.name,
                    port: active_svc.port as u16,
                    status: active_svc.status,
                })
                .collect()
        } else {
            all_services
                .into_iter()
                .filter(|svc| svc.status == "running")
                .map(|active_svc| McpService {
                    name: active_svc.name,
                    port: active_svc.port as u16,
                    status: active_svc.status,
                })
                .collect()
        };

        Ok(services)
    }

    async fn load_agent_mcp_servers(
        &self,
        agent_name: &systemprompt_identifiers::AgentName,
    ) -> Result<Vec<String>> {
        use systemprompt_core_mcp::services::deployment::DeploymentService;

        let config = DeploymentService::load_config().await?;

        let agent = config
            .agents
            .get(agent_name.as_str())
            .ok_or_else(|| anyhow!("Agent '{agent_name}' not found in services.yaml"))?;

        Ok(agent.metadata.mcp_servers.clone())
    }

    pub async fn health_check(&self) -> Result<HashMap<String, bool>> {
        let services = self.services.read().await;
        let mut health_status = HashMap::new();

        for (service_name, connection) in services.iter() {
            let is_healthy = validate_connection(
                &connection.service.name,
                "127.0.0.1",
                connection.service.port,
            )
            .await
            .map(|r| r.success)
            .unwrap_or(false);

            health_status.insert(service_name.clone(), is_healthy);
        }

        Ok(health_status)
    }
}
