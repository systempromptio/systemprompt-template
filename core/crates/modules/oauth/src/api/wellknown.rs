use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
use systemprompt_core_system::AppContext;

pub fn wellknown_routes(ctx: &AppContext) -> Router {
    Router::new()
        .route(
            "/.well-known/oauth-authorization-server",
            get(super::rest::discovery::handle_well_known).options(|| async { StatusCode::OK }),
        )
        .route(
            "/.well-known/oauth-authorization-server/",
            get(super::rest::discovery::handle_well_known).options(|| async { StatusCode::OK }),
        )
        .route(
            "/.well-known/openid-configuration",
            get(super::rest::discovery::handle_well_known).options(|| async { StatusCode::OK }),
        )
        .route(
            "/.well-known/openid-configuration/",
            get(super::rest::discovery::handle_well_known).options(|| async { StatusCode::OK }),
        )
        .route(
            "/.well-known/oauth-protected-resource",
            get(super::rest::discovery::handle_oauth_protected_resource)
                .options(|| async { StatusCode::OK }),
        )
        .route(
            "/.well-known/oauth-protected-resource/",
            get(super::rest::discovery::handle_oauth_protected_resource)
                .options(|| async { StatusCode::OK }),
        )
        .with_state(ctx.clone())
}
