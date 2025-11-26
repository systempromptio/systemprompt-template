use axum::Router;

pub mod routes;

pub use routes::*;

pub fn registry_router(ctx: &systemprompt_core_system::AppContext) -> Router {
    registry::router(ctx)
}
