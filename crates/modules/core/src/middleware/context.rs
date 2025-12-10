use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
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
            ContextExtractionError::MissingAuthHeader => (
                StatusCode::UNAUTHORIZED,
                "Missing Authorization header".to_string(),
            ),
            ContextExtractionError::InvalidToken(_) => (
                StatusCode::UNAUTHORIZED,
                "Invalid or expired JWT token".to_string(),
            ),
            ContextExtractionError::UserNotFound(_) => (
                StatusCode::UNAUTHORIZED,
                "User no longer exists".to_string(),
            ),
            ContextExtractionError::MissingSessionId => (
                StatusCode::BAD_REQUEST,
                "JWT missing required 'session_id' claim".to_string(),
            ),
            ContextExtractionError::MissingUserId => (
                StatusCode::BAD_REQUEST,
                "JWT missing required 'sub' claim".to_string(),
            ),
            ContextExtractionError::MissingContextId => (
                StatusCode::BAD_REQUEST,
                "Missing required 'x-context-id' header (for MCP routes) or contextId in body \
                 (for A2A routes)"
                    .to_string(),
            ),
            ContextExtractionError::MissingHeader(header) => (
                StatusCode::BAD_REQUEST,
                format!("Missing required header: {header}"),
            ),
            ContextExtractionError::InvalidHeaderValue { header, reason } => (
                StatusCode::BAD_REQUEST,
                format!("Invalid header {header}: {reason}"),
            ),
            ContextExtractionError::InvalidUserId(reason) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid user_id: {reason}"),
            ),
            ContextExtractionError::DatabaseError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            ContextExtractionError::ForbiddenHeader { header, reason } => (
                StatusCode::BAD_REQUEST,
                format!(
                    "Header '{header}' is not allowed: {reason}. Use JWT authentication instead."
                ),
            ),
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

        if request.extensions().get::<RequestContext>().is_some()
            && self.auth_level == ContextRequirement::None
        {
            return next.run(request).await;
        }

        let headers = request.headers();

        match requirement {
            ContextRequirement::None => {
                let mut req_ctx = if let Some(ctx) = request.extensions().get::<RequestContext>() {
                    ctx.clone()
                } else {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Middleware configuration error: SessionMiddleware must run before \
                         ContextMiddleware",
                    )
                        .into_response();
                };

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
