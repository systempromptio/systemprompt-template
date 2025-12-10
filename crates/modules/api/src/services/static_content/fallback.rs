use axum::extract::State;
use axum::http::{Method, StatusCode, Uri};
use axum::response::{IntoResponse, Json};
use serde_json::json;

use super::vite::StaticContentState;

pub async fn smart_fallback_handler(
    State(state): State<StaticContentState>,
    uri: Uri,
    method: Method,
    req_ctx: Option<axum::Extension<systemprompt_core_system::RequestContext>>,
) -> impl IntoResponse {
    let path = uri.path();

    if is_api_path(path) {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Not Found",
                "message": format!("No route matches {method} {path}"),
                "path": path,
                "suggestions": get_api_suggestions(path)
            })),
        )
            .into_response();
    }

    super::serve_vite_app(State(state), uri, req_ctx)
        .await
        .into_response()
}

fn is_api_path(path: &str) -> bool {
    path.starts_with("/api/")
        || path.starts_with("/.well-known/")
        || path.starts_with("/server/")
        || path.starts_with("/mcp/")
        || path.starts_with("/agent/")
        || path.starts_with("/health")
        || path.starts_with("/openapi")
        || path.starts_with("/docs/")
        || path.starts_with("/swagger/")
        || path.starts_with("/v1/")
        || path.starts_with("/auth/")
        || path.starts_with("/oauth/")
}

fn get_api_suggestions(path: &str) -> Vec<&'static str> {
    if path.starts_with("/api/") {
        vec![
            "/api/v1 - API discovery endpoint",
            "/api/v1/openapi - OpenAPI specification",
            "/api/v1/health - Health check",
            "/api/v1/core - Core services discovery",
            "/api/v1/agents - Agent services discovery",
            "/api/v1/mcp - MCP services discovery",
        ]
    } else if path.starts_with("/.well-known/") {
        vec![
            "/.well-known/oauth-authorization-server - OAuth metadata",
            "/.well-known/agent-card.json - Agent card",
        ]
    } else if path.contains("health") {
        vec!["/api/v1/health - Health check endpoint"]
    } else if path.contains("openapi") || path.contains("swagger") {
        vec!["/api/v1/openapi - OpenAPI specification"]
    } else {
        vec![
            "/api/v1 - Start here for API discovery",
            "/ - Frontend application",
        ]
    }
}
