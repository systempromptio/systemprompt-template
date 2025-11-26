use axum::Router;

pub mod rest;
pub mod routes;
pub mod wellknown;

pub fn router(ctx: &systemprompt_core_system::AppContext) -> Router {
    Router::new()
        .merge(routes::router())
        .with_state(ctx.clone())
}

pub fn public_router(ctx: &systemprompt_core_system::AppContext) -> Router {
    Router::new()
        .merge(routes::public_router())
        .with_state(ctx.clone())
}

pub fn authenticated_router(ctx: &systemprompt_core_system::AppContext) -> Router {
    Router::new()
        .merge(routes::authenticated_router())
        .with_state(ctx.clone())
}

systemprompt_core_system::register_module_api!(
    "oauth",
    systemprompt_core_system::ServiceCategory::Core,
    router,
    false
);
