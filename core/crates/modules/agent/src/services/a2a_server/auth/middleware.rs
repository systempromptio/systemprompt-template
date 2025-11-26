use axum::{
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use super::types::AgentOAuthState;
use crate::services::a2a_server::handlers::AgentHandlerState;

pub async fn agent_oauth_middleware(
    State(state): State<AgentOAuthState>,
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let headers = request.headers();

    let context = state
        .auth_service
        .validate_request(headers, state.auth_mode())
        .await
        .map_err(|e| {
            tracing::error!("Authentication failed: {}", e);
            StatusCode::UNAUTHORIZED
        })?;

    request.extensions_mut().insert(context);

    Ok(next.run(request).await)
}

pub fn get_user_context(
    request: &Request<axum::body::Body>,
) -> Option<&super::types::AgentAuthenticatedUser> {
    request
        .extensions()
        .get::<super::types::AgentAuthenticatedUser>()
}

pub async fn agent_oauth_middleware_wrapper(
    State(handler_state): State<Arc<AgentHandlerState>>,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    agent_oauth_middleware(
        State(handler_state.oauth_state.as_ref().clone()),
        request,
        next,
    )
    .await
}
