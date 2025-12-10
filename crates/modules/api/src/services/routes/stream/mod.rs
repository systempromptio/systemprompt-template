use axum::routing::get;
use axum::Router;
use systemprompt_core_system::AppContext;

pub mod contexts;

pub fn stream_router(ctx: &AppContext) -> Router {
    Router::new()
        .route("/contexts", get(contexts::stream_context_state))
        .with_state(ctx.clone())
}
