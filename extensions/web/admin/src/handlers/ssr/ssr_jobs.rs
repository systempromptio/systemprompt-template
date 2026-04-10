use std::sync::Arc;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

pub async fn jobs_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (
            axum::http::StatusCode::FORBIDDEN,
            axum::response::Html(super::ACCESS_DENIED_HTML),
        )
            .into_response();
    }

    let jobs = repositories::list_jobs(&pool).await.unwrap_or_else(|e| {
        tracing::error!(error = %e, "Failed to list jobs");
        vec![]
    });
    let data = json!({
        "page": "jobs",
        "title": "Jobs",
        "jobs": jobs,
    });
    super::render_page(&engine, "jobs", &data, &user_ctx, &mkt_ctx)
}
