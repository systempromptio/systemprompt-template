use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;

use crate::middleware::requirements::ContextRequirement;
use systemprompt_identifiers::{AgentName, ContextId};
use systemprompt_models::execution::context::{ContextExtractionError, RequestContext};

#[derive(Debug, Clone)]
pub struct ContextMiddleware<E> {
    extractor: Arc<E>,
    auth_level: ContextRequirement,
}

impl<E> ContextMiddleware<E> {
    pub fn new(extractor: E) -> Self {
        Self {
            extractor: Arc::new(extractor),
            auth_level: ContextRequirement::default(),
        }
    }

    pub fn public(extractor: E) -> Self {
        Self {
            extractor: Arc::new(extractor),
            auth_level: ContextRequirement::None,
        }
    }

    pub fn user_only(extractor: E) -> Self {
        Self {
            extractor: Arc::new(extractor),
            auth_level: ContextRequirement::UserOnly,
        }
    }

    pub fn full(extractor: E) -> Self {
        Self {
            extractor: Arc::new(extractor),
            auth_level: ContextRequirement::UserWithContext,
        }
    }

    pub fn mcp(extractor: E) -> Self {
        Self {
            extractor: Arc::new(extractor),
            auth_level: ContextRequirement::McpWithHeaders,
        }
    }

    fn error_response(error: &ContextExtractionError) -> (StatusCode, String) {
        match error {
            ContextExtractionError::MissingAuthHeader => {
                (StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string())
            }
            ContextExtractionError::InvalidToken(_) => {
                (StatusCode::UNAUTHORIZED, "Invalid or expired JWT token".to_string())
            }
            ContextExtractionError::UserNotFound(_) => {
                (StatusCode::UNAUTHORIZED, "User no longer exists".to_string())
            }
            ContextExtractionError::MissingSessionId => {
                (StatusCode::BAD_REQUEST, "JWT missing required 'session_id' claim".to_string())
            }
            ContextExtractionError::MissingUserId => {
                (StatusCode::BAD_REQUEST, "JWT missing required 'sub' claim".to_string())
            }
            ContextExtractionError::MissingContextId => {
                (StatusCode::BAD_REQUEST, "Missing required 'x-context-id' header (for MCP routes) or contextId in body (for A2A routes)".to_string())
            }
            ContextExtractionError::MissingHeader(header) => {
                (StatusCode::BAD_REQUEST, format!("Missing required header: {header}"))
            }
            ContextExtractionError::InvalidHeaderValue { header, reason } => {
                (StatusCode::BAD_REQUEST, format!("Invalid header {header}: {reason}"))
            }
            ContextExtractionError::InvalidUserId(reason) => {
                (StatusCode::BAD_REQUEST, format!("Invalid user_id: {reason}"))
            }
            ContextExtractionError::DatabaseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
            ContextExtractionError::ForbiddenHeader { header, reason } => {
                (StatusCode::BAD_REQUEST, format!("Header '{header}' is not allowed: {reason}. Use JWT authentication instead."))
            }
        }
    }
}

use crate::middleware::extractors::ContextExtractor;

impl<E: ContextExtractor> ContextMiddleware<E> {
    pub async fn handle(&self, mut request: Request, next: Next) -> Response {
        let requirement = request
            .extensions()
            .get::<ContextRequirement>()
            .copied()
            .unwrap_or(self.auth_level);

        // Check if we already have a context that meets the current auth requirement
        if request.extensions().get::<RequestContext>().is_some() {
            // Only reuse if this middleware's requirement is None (public)
            // More restrictive middleware (UserOnly, etc.) must re-validate
            if self.auth_level == ContextRequirement::None {
                return next.run(request).await;
            }
        }

        let headers = request.headers();

        match requirement {
            ContextRequirement::None => {
                // Get existing RequestContext (created by SessionMiddleware)
                let mut req_ctx = if let Some(ctx) = request.extensions().get::<RequestContext>() { ctx.clone() } else {
                    tracing::error!("SessionMiddleware must run before ContextMiddleware");
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Middleware configuration error",
                    )
                        .into_response();
                };

                // Extract conversation context fields (optional)
                if let Some(context_id) = headers.get("x-context-id") {
                    if let Ok(id) = context_id.to_str() {
                        req_ctx.execution.context_id = ContextId::new(id.to_string());
                    }
                }

                if let Some(agent_name) = headers.get("x-agent-name") {
                    if let Ok(name) = agent_name.to_str() {
                        req_ctx.execution.agent_name = AgentName::new(name.to_string());
                    }
                }

                request.extensions_mut().insert(req_ctx);
                next.run(request).await
            },
            ContextRequirement::UserOnly => match self.extractor.extract_user_only(headers).await {
                Ok(context) => {
                    request.extensions_mut().insert(context);
                    next.run(request).await
                },
                Err(e) => {
                    let (status, message) = Self::error_response(&e);
                    (status, message).into_response()
                },
            },
            ContextRequirement::UserWithContext => {
                match self.extractor.extract_from_request(request).await {
                    Ok((context, reconstructed_request)) => {
                        let mut req = reconstructed_request;
                        req.extensions_mut().insert(context);
                        next.run(req).await
                    },
                    Err(e) => {
                        let (status, message) = Self::error_response(&e);
                        (status, message).into_response()
                    },
                }
            },
            ContextRequirement::McpWithHeaders => {
                match self.extractor.extract_from_headers(headers).await {
                    Ok(context) => {
                        request.extensions_mut().insert(context);
                        next.run(request).await
                    },
                    Err(e) => {
                        let (status, message) = Self::error_response(&e);
                        (status, message).into_response()
                    },
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use axum::body::Body;
    use axum::http::Method;
    use systemprompt_identifiers::{AgentName, ContextId, SessionId, TraceId};
    use systemprompt_models::execution::context::{ContextExtractionError, RequestContext};

    struct MockExtractor {
        should_fail: bool,
    }

    #[async_trait]
    impl ContextExtractor for MockExtractor {
        async fn extract_from_headers(
            &self,
            _headers: &axum::http::HeaderMap,
        ) -> Result<RequestContext, ContextExtractionError> {
            if self.should_fail {
                Err(ContextExtractionError::MissingHeader(
                    "test-header".to_string(),
                ))
            } else {
                Ok(RequestContext::new(
                    SessionId::new("test_session".to_string()),
                    TraceId::new("test_trace".to_string()),
                    ContextId::new("test_context".to_string()),
                    AgentName::new("test-agent".to_string()),
                ))
            }
        }
    }

    async fn test_handler(request: Request) -> Response {
        let context = request.extensions().get::<RequestContext>().cloned();
        match context {
            Some(ctx) => (
                StatusCode::OK,
                format!("session: {}", ctx.request.session_id.as_str()),
            )
                .into_response(),
            None => (StatusCode::INTERNAL_SERVER_ERROR, "No context").into_response(),
        }
    }
}
