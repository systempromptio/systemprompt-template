use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};

use crate::repository::ContextRepository;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::api::ApiError;

pub async fn get_context(
    Extension(req_ctx): Extension<systemprompt_core_system::RequestContext>,
    State(ctx): State<systemprompt_core_system::AppContext>,
    Path(context_id): Path<String>,
) -> impl IntoResponse {
    let db_pool = ctx.db_pool().clone();
    let context_repo = ContextRepository::new(db_pool.clone());
    let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());
    let user_id = &req_ctx.auth.user_id;

    match context_repo
        .get_context(&context_id, user_id.as_str())
        .await
    {
        Ok(context) => {
            logger
                .debug(
                    "context_api",
                    &format!("Retrieved context {context_id} for user {user_id}"),
                )
                .await
                .ok();
            (StatusCode::OK, Json(context)).into_response()
        },
        Err(e) => {
            logger
                .error("context_api", &format!("Failed to get context: {e}"))
                .await
                .ok();
            ApiError::not_found(format!("Context not found: {e}")).into_response()
        },
    }
}
