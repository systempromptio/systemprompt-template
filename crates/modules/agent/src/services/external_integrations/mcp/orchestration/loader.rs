use anyhow::Result;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::RequestContext;

use super::super::client::McpClientAdapter;
use super::super::service::ServiceStateManager;
use crate::models::a2a::{AgentExtension, McpServerMetadata};

#[derive(Debug, Clone)]
pub struct McpToolLoader {
    db_pool: DbPool,
    service_manager: ServiceStateManager,
}

impl McpToolLoader {
    pub fn new(db_pool: DbPool) -> Self {
        Self {
            service_manager: ServiceStateManager::new(db_pool.clone()),
            db_pool,
        }
    }

    /// Load tools from a list of MCP server names (config-driven approach)
    /// Logs errors with full context but continues to support degraded mode
    /// when servers are unavailable Filters servers based on user's JWT
    /// permissions
    pub async fn load_tools_for_servers(
        &self,
        server_names: &[String],
        context: &RequestContext,
    ) -> Result<std::collections::HashMap<String, Vec<systemprompt_models::ai::tools::McpTool>>>
    {
        use systemprompt_core_mcp::services::deployment::DeploymentService;
        use systemprompt_core_oauth::services::validation::jwt::validate_jwt_token;

        let logger = LogService::system(self.db_pool.clone());
        let mut tools_by_server = std::collections::HashMap::new();
        let mut load_errors = Vec::new();
        let mut skipped_servers = Vec::new();

        // Get deployment config to check OAuth requirements
        let deployment_config = DeploymentService::load_config().await?;

        // Decode JWT to get user permissions
        let jwt_secret = std::env::var("JWT_SECRET")
            .unwrap_or_else(|_| "dev-secret-key-change-in-production".to_string());

        let user_permissions = match validate_jwt_token(context.auth_token().as_str(), &jwt_secret)
        {
            Ok(claims) => claims.get_permissions(),
            Err(e) => {
                // Only log as error if token was provided but invalid
                // Empty or missing tokens are expected for unauthenticated requests
                if !context.auth_token().as_str().is_empty() {
                    logger
                        .error(
                            "mcp_tool_loader",
                            &format!("JWT validation failed: {}. Using empty permissions.", e),
                        )
                        .await
                        .ok();
                }
                vec![]
            },
        };

        for server_name in server_names {
            // Check if user has permission to access this server
            if let Some(deployment) = deployment_config.mcp_servers.get(server_name) {
                if deployment.oauth.required && !deployment.oauth.scopes.is_empty() {
                    // Check if user has ANY of the required scopes
                    let has_permission = deployment
                        .oauth
                        .scopes
                        .iter()
                        .any(|required_scope| user_permissions.contains(required_scope));

                    if !has_permission {
                        skipped_servers.push(server_name.clone());
                        continue;
                    }
                }
            }

            // User has permission, attempt to load tools
            let timeout_duration = tokio::time::Duration::from_secs(10);
            let load_result = tokio::time::timeout(
                timeout_duration,
                self.load_server_tools(server_name, context),
            )
            .await;

            match load_result {
                Ok(Ok(tools)) => {
                    tools_by_server.insert(server_name.clone(), tools);
                },
                Ok(Err(e)) => {
                    let error_msg = format!(
                        "Failed to load tools from MCP server '{}': {}",
                        server_name, e
                    );
                    logger.error("mcp_tool_loader", &error_msg).await.ok();
                    load_errors.push(error_msg);
                },
                Err(_) => {
                    let error_msg = format!(
                        "Timeout loading tools from MCP server '{}' (exceeded 10s)",
                        server_name
                    );
                    logger.error("mcp_tool_loader", &error_msg).await.ok();
                    load_errors.push(error_msg);
                },
            }
        }

        // Log summary only if there are errors (skip permission-denied cases)
        if !load_errors.is_empty() {
            let message = format!(
                "Tool loading: {}/{} servers succeeded. Failed: {}",
                tools_by_server.len(),
                server_names.len(),
                load_errors.join("; ")
            );
            logger.error("mcp_tool_loader", &message).await.ok();
        }

        Ok(tools_by_server)
    }

    /// Load tools from a specific MCP server (returns McpTool, not skills)
    /// Retries up to 3 times with backoff to account for database replication
    /// lag
    pub async fn load_server_tools(
        &self,
        server_name: &str,
        context: &RequestContext,
    ) -> Result<Vec<systemprompt_models::ai::tools::McpTool>> {
        let mut retries = 0;
        let max_retries = 3;

        loop {
            match self.service_manager.get_mcp_service(server_name).await {
                Ok(Some(service)) => {
                    if service.status != "running" {
                        return Err(anyhow::anyhow!(
                            "MCP server '{}' is not running (status: {})",
                            server_name,
                            service.status
                        ));
                    }
                    return McpClientAdapter::fetch_tools(server_name, context).await;
                },
                Ok(None) => {
                    // Service not found - might be DB replication lag
                    if retries < max_retries {
                        let backoff_ms = 100 * (2_u64.pow(retries as u32));
                        tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                        retries += 1;
                        continue;
                    }
                    return Err(anyhow::anyhow!(
                        "MCP server '{}' not found in services database (after {} retries with \
                         {}ms DB lag tolerance)",
                        server_name,
                        max_retries,
                        100 * (2_u64.pow(max_retries as u32) - 1)
                    ));
                },
                Err(e) => {
                    // Database query error - fail immediately, don't retry
                    return Err(anyhow::anyhow!(
                        "Database error querying MCP server '{}': {} (this indicates a database \
                         connectivity issue, not replication lag)",
                        server_name,
                        e
                    ));
                },
            }
        }
    }

    /// Create MCP extensions for agent card from server list
    /// Uses MCP registry and deployment config as source of truth
    pub async fn create_mcp_extensions(
        &self,
        server_names: &[String],
        base_url: &str,
        context: &RequestContext,
    ) -> Result<Vec<AgentExtension>> {
        use systemprompt_core_mcp::services::deployment::DeploymentService;
        use systemprompt_core_mcp::services::registry::manager::RegistryService;

        if server_names.is_empty() {
            return Ok(vec![]);
        }

        let deployment_config = DeploymentService::load_config().await?;
        let tools_by_server = self.load_tools_for_servers(server_names, context).await?;
        let mut servers_info = Vec::new();

        for server_name in server_names {
            // Check if server exists in deployment config first (primary source of truth)
            if let Some(deployment) = deployment_config.mcp_servers.get(server_name) {
                // Determine auth requirement from OAuth scopes
                let auth_value = if !deployment.oauth.required || deployment.oauth.scopes.is_empty()
                {
                    "anon".to_string()
                } else {
                    deployment
                        .oauth
                        .scopes
                        .first()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "user".to_string())
                };

                // Get runtime status from services table
                let runtime_status = self
                    .service_manager
                    .get_mcp_service(server_name)
                    .await?
                    .map(|s| s.status)
                    .unwrap_or_else(|| "not_started".to_string());

                // Try to get version from manifest (optional)
                let version = RegistryService::load_manifest(server_name)
                    .await
                    .ok()
                    .map(|m| m.version);

                // Get tools for this server
                let tools = tools_by_server.get(server_name).cloned();

                servers_info.push(McpServerMetadata {
                    name: server_name.to_string(),
                    endpoint: format!("{}/api/v1/mcp/{}/mcp", base_url, server_name),
                    auth: auth_value,
                    status: runtime_status,
                    version,
                    tools,
                });
            } else {
                // Server not in deployment config
                servers_info.push(McpServerMetadata {
                    name: server_name.to_string(),
                    endpoint: format!("{}/api/v1/mcp/{}/mcp", base_url, server_name),
                    auth: "unknown".to_string(),
                    status: "not_in_config".to_string(),
                    version: None,
                    tools: None,
                });
            }
        }

        // MCP protocol version from rmcp specification
        let mcp_protocol_version = systemprompt_core_mcp::mcp_protocol_version();

        Ok(vec![AgentExtension {
            uri: "systemprompt:mcp-tools".to_string(),
            description: Some("MCP tool execution capabilities with server endpoints".to_string()),
            required: Some(true),
            params: Some(serde_json::json!({
                "supported_protocols": [mcp_protocol_version],
                "servers": servers_info
            })),
        }])
    }
}
