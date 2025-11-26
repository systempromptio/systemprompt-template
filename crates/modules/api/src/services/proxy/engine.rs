use axum::{
    body::Body,
    extract::{Path, Request, State},
    http::StatusCode,
    response::Response,
};
use systemprompt_core_logging::LogService;
use systemprompt_core_system::AppContext;
use systemprompt_models::repository::{ServiceConfig, ServiceRepository};

use super::auth::{AuthValidator, OAuthChallengeBuilder};
use super::backend::{HeaderInjector, ProxyError, RequestBuilder, ResponseHandler, UrlResolver};
use super::client::ClientPool;

#[derive(Debug, Clone)]
pub struct ProxyEngine {
    client_pool: ClientPool,
}

impl ProxyEngine {
    pub fn new() -> Self {
        Self {
            client_pool: ClientPool::new(),
        }
    }

    fn resolve_service<'a>(
        &'a self,
        service_name: &'a str,
        logger: &'a LogService,
        ctx: &'a AppContext,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<ServiceConfig, ProxyError>> + Send + 'a>,
    > {
        Box::pin(async move { self.resolve_service_impl(service_name, logger, ctx).await })
    }

    async fn resolve_service_impl(
        &self,
        service_name: &str,
        logger: &LogService,
        ctx: &AppContext,
    ) -> Result<ServiceConfig, ProxyError> {
        let service_repo = ServiceRepository::new(ctx.db_pool().clone());

        let service = match service_repo.get_service_by_name(service_name).await {
            Ok(svc) => svc,
            Err(e) => {
                logger
                    .error(
                        "api_proxy",
                        &format!(
                            "Database error when looking up service {}: {}",
                            service_name, e
                        ),
                    )
                    .await
                    .ok();
                return Err(ProxyError::DatabaseError {
                    service: service_name.to_string(),
                    source: e,
                });
            },
        };

        let service = match service {
            Some(svc) => svc,
            None => {
                logger
                    .warn("api_proxy", &format!("Service not found: {}", service_name))
                    .await
                    .ok();
                return Err(ProxyError::ServiceNotFound {
                    service: service_name.to_string(),
                });
            },
        };

        if service.status != "running" {
            if service.status == "crashed" {
                logger
                    .info(
                        "api_proxy",
                        &format!("Service {} crashed, attempting restart", service_name),
                    )
                    .await
                    .ok();

                if let Ok(_) = self
                    .attempt_service_restart(service_name, ctx, &logger)
                    .await
                {
                    logger
                        .info(
                            "api_proxy",
                            "Service restarted successfully, retrying proxy",
                        )
                        .await
                        .ok();
                    return Box::pin(self.resolve_service_impl(service_name, logger, ctx)).await;
                }
            }

            logger
                .warn(
                    "api_proxy",
                    &format!(
                        "Service {} not running (status: {})",
                        service_name, service.status
                    ),
                )
                .await
                .ok();
            return Err(ProxyError::ServiceNotRunning {
                service: service_name.to_string(),
                status: service.status.clone(),
            });
        }

        Ok(service)
    }

    async fn validate_access(
        &self,
        headers: &http::HeaderMap,
        service_name: &str,
        service: &ServiceConfig,
        ctx: &AppContext,
        req_context: Option<&systemprompt_core_system::RequestContext>,
    ) -> Result<(), ProxyError> {
        use systemprompt_core_agent::services::registry::AgentRegistry;
        use systemprompt_core_mcp::McpServerRegistry;

        let (oauth_required, required_scopes) = if service.module_name == "agent" {
            match AgentRegistry::new().await {
                Ok(registry) => match registry.get_agent(service_name).await {
                    Ok(agent_config) => (
                        agent_config.oauth.required,
                        agent_config.oauth.scopes.clone(),
                    ),
                    Err(e) => {
                        return Err(ProxyError::ServiceNotFound {
                            service: format!(
                                "Agent '{}' not found in registry: {}",
                                service_name, e
                            ),
                        });
                    },
                },
                Err(e) => {
                    return Err(ProxyError::ServiceNotRunning {
                        service: service_name.to_string(),
                        status: format!("Failed to load agent registry: {}", e),
                    });
                },
            }
        } else if service.module_name == "mcp" {
            match McpServerRegistry::new().await {
                Ok(registry) => match registry.get_server(service_name).await {
                    Ok(server_config) => (
                        server_config.oauth.required,
                        server_config.oauth.scopes.clone(),
                    ),
                    Err(e) => {
                        return Err(ProxyError::ServiceNotFound {
                            service: format!(
                                "MCP server '{}' not found in registry: {}",
                                service_name, e
                            ),
                        });
                    },
                },
                Err(e) => {
                    return Err(ProxyError::ServiceNotRunning {
                        service: service_name.to_string(),
                        status: format!("Failed to load MCP registry: {}", e),
                    });
                },
            }
        } else {
            (true, vec![])
        };

        if !oauth_required {
            return Ok(());
        }

        let authenticated_user =
            match AuthValidator::validate_service_access(headers, service_name, ctx, req_context)
                .await
            {
                Ok(user) => user,
                Err(status_code) => {
                    match OAuthChallengeBuilder::build_challenge_response(
                        service_name,
                        ctx,
                        status_code,
                    )
                    .await
                    {
                        Ok(_response) => {
                            return Err(ProxyError::AuthenticationRequired {
                                service: service_name.to_string(),
                            });
                        },
                        Err(status) => {
                            return Err(if status == StatusCode::UNAUTHORIZED {
                                ProxyError::AuthenticationRequired {
                                    service: service_name.to_string(),
                                }
                            } else {
                                ProxyError::Forbidden {
                                    service: service_name.to_string(),
                                }
                            });
                        },
                    }
                },
            };

        if !required_scopes.is_empty() {
            let has_required_scope = required_scopes.iter().any(|required_permission| {
                authenticated_user
                    .permissions
                    .iter()
                    .any(|user_permission| {
                        user_permission == required_permission
                            || user_permission.implies(required_permission)
                    })
            });

            if !has_required_scope {
                return Err(ProxyError::Forbidden {
                    service: format!(
                        "Insufficient permissions for {}. Required: {:?}, User has: {:?}",
                        service_name, required_scopes, authenticated_user.permissions
                    ),
                });
            }
        }

        Ok(())
    }

    pub async fn handle_mcp_request(
        &self,
        path_params: Path<(String,)>,
        State(ctx): State<AppContext>,
        request: Request<Body>,
    ) -> Result<Response<Body>, StatusCode> {
        let Path((service_name,)) = path_params;
        self.proxy_request(&service_name, "", request, ctx)
            .await
            .map_err(|e| e.to_status_code())
    }

    pub async fn handle_mcp_request_with_path(
        &self,
        path_params: Path<(String, String)>,
        State(ctx): State<AppContext>,
        request: Request<Body>,
    ) -> Result<Response<Body>, StatusCode> {
        let Path((service_name, path)) = path_params;
        self.proxy_request(&service_name, &path, request, ctx)
            .await
            .map_err(|e| e.to_status_code())
    }

    pub async fn handle_agent_request(
        &self,
        path_params: Path<(String,)>,
        State(ctx): State<AppContext>,
        request: Request<Body>,
    ) -> Result<Response<Body>, StatusCode> {
        let Path((service_name,)) = path_params;
        self.proxy_request(&service_name, "", request, ctx)
            .await
            .map_err(|e| e.to_status_code())
    }

    pub async fn handle_agent_request_with_path(
        &self,
        path_params: Path<(String, String)>,
        State(ctx): State<AppContext>,
        request: Request<Body>,
    ) -> Result<Response<Body>, StatusCode> {
        let Path((service_name, path)) = path_params;
        self.proxy_request(&service_name, &path, request, ctx)
            .await
            .map_err(|e| e.to_status_code())
    }

    async fn attempt_service_restart(
        &self,
        service_name: &str,
        ctx: &AppContext,
        logger: &LogService,
    ) -> Result<(), ProxyError> {
        use std::sync::Arc;
        use systemprompt_core_mcp::services::McpManager;

        let orchestrator = McpManager::new(Arc::new(ctx.clone())).await.map_err(|e| {
            ProxyError::ServiceNotRunning {
                service: service_name.to_string(),
                status: format!("Failed to create orchestrator: {}", e),
            }
        })?;

        match orchestrator
            .start_services(Some(service_name.to_string()))
            .await
        {
            Ok(_) => {},
            Err(e) => {
                logger
                    .error(
                        "api_proxy",
                        &format!("Failed to restart service {}: {}", service_name, e),
                    )
                    .await
                    .ok();
                return Err(ProxyError::ServiceNotRunning {
                    service: service_name.to_string(),
                    status: format!("Restart failed: {}", e),
                });
            },
        }

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        Ok(())
    }

    pub async fn proxy_request(
        &self,
        service_name: &str,
        path: &str,
        request: Request<Body>,
        ctx: AppContext,
    ) -> Result<Response<Body>, ProxyError> {
        use systemprompt_core_logging::LogContext;
        use systemprompt_core_system::RequestContext;

        let request_context = request.extensions().get::<RequestContext>();

        let log_context = match request_context {
            Some(req_ctx) => req_ctx.log_context(),
            None => {
                let system_context = LogContext::system();
                let temp_logger = LogService::new(ctx.db_pool().clone(), system_context.clone());
                temp_logger
                    .warn(
                        "api_proxy",
                        "RequestContext missing from request extensions, using system context",
                    )
                    .await
                    .ok();
                system_context
            },
        };

        let logger = LogService::new(ctx.db_pool().clone(), log_context);

        let service = self.resolve_service(service_name, &logger, &ctx).await?;

        let req_ctx = request.extensions().get::<RequestContext>().cloned();
        self.validate_access(
            request.headers(),
            service_name,
            &service,
            &ctx,
            req_ctx.as_ref(),
        )
        .await?;

        let backend_url = UrlResolver::build_backend_url("http", "127.0.0.1", service.port, path);

        // Debug logging removed for performance

        let method_str = request.method().to_string();
        let mut headers = request.headers().clone();
        let query = request.uri().query();
        let full_url = UrlResolver::append_query_params(backend_url, query);

        // Inject agent context - agent name comes from URL path parameter
        // FAIL FAST: No fallbacks. If context is missing, something is broken upstream.
        let mut req_context = req_ctx.clone().ok_or_else(|| ProxyError::MissingContext {
            message: "Request context required - proxy cannot operate without authentication"
                .to_string(),
        })?;

        // Set agent name from service_name for both agent and MCP services
        // This ensures tasks are always attributed to the service that executes them
        use systemprompt_identifiers::AgentName;
        if service.module_name == "agent" || service.module_name == "mcp" {
            req_context = req_context.with_agent_name(AgentName::new(service_name.to_string()));
        }

        // Inject all context headers (including x-agent-name)
        HeaderInjector::inject_context(&mut headers, &req_context);

        let body = RequestBuilder::extract_body(request.into_body())
            .await
            .map_err(|_| ProxyError::BodyExtractionFailed {
                source: axum::Error::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed to extract request body",
                )),
            })?;

        let reqwest_method =
            RequestBuilder::parse_method(&method_str).map_err(|_| ProxyError::InvalidMethod {
                method: method_str.clone(),
            })?;

        let client = self.client_pool.get_default_client();

        let req_builder =
            RequestBuilder::build_request(&client, reqwest_method, &full_url, &headers, body)
                .map_err(|_| ProxyError::UrlConstructionFailed {
                    service: service_name.to_string(),
                    reason: "Failed to build request".to_string(),
                })?;

        let response = match req_builder.send().await {
            Ok(resp) => resp,
            Err(e) => {
                logger
                    .error(
                        "api_proxy",
                        &format!(
                            "Connection failed to {} at {}: {}",
                            service_name, full_url, e
                        ),
                    )
                    .await
                    .ok();
                return Err(ProxyError::ConnectionFailed {
                    service: service_name.to_string(),
                    url: full_url.clone(),
                    source: e,
                });
            },
        };

        match ResponseHandler::build_response(response).await {
            Ok(resp) => Ok(resp),
            Err(e) => {
                logger
                    .error(
                        "api_proxy",
                        &format!("Failed to build response from {}: {}", service_name, e),
                    )
                    .await
                    .ok();
                Err(ProxyError::InvalidResponse {
                    service: service_name.to_string(),
                    reason: format!("Failed to build response: {}", e),
                })
            },
        }
    }
}
