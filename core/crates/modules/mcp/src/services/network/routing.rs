use anyhow::Result;
use axum::http::HeaderValue;
use axum::Router;
use systemprompt_core_system::Config;
use tower_http::cors::CorsLayer;

pub async fn create_base_router() -> Result<Router> {
    let router = Router::new().route("/health", axum::routing::get(health_check));

    Ok(router)
}

pub async fn apply_cors_layer(router: Router) -> Result<Router> {
    let config = Config::global();

    let mut cors_layer = CorsLayer::new()
        .allow_headers(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any);

    for origin in &config.cors_allowed_origins {
        if let Ok(header_value) = origin.parse::<HeaderValue>() {
            cors_layer = cors_layer.allow_origin(header_value);
        }
    }

    Ok(router.layer(cors_layer))
}

async fn health_check() -> impl axum::response::IntoResponse {
    "OK"
}

pub async fn create_mcp_router(base_router: Router, mcp_router: Router) -> Result<Router> {
    Ok(base_router.nest("/mcp", mcp_router))
}

pub async fn add_middleware(router: Router) -> Result<Router> {
    // Middleware would be applied here
    Ok(router)
}
