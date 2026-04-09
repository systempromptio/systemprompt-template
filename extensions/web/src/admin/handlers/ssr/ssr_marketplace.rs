use std::sync::Arc;

use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

pub async fn marketplace_versions_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let user_id = user_ctx.user_id.as_str();
    let limit: i64 = 50;
    let offset: i64 = params
        .get("offset")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
        .max(0);

    let total = crate::admin::activity::queries::count_user_entity_activity(&pool, user_id)
        .await
        .unwrap_or(0);

    let events =
        crate::admin::activity::queries::get_user_entity_activity(&pool, user_id, limit, offset)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Failed to load user entity activity");
                vec![]
            });

    let has_prev = offset > 0;
    let has_next = offset + limit < total;
    let prev_offset = (offset - limit).max(0);
    let next_offset = offset + limit;

    let data = json!({
        "page": "marketplace-versions",
        "title": "My Activity",
        "events": events,
        "total": total,
        "limit": limit,
        "offset": offset,
        "has_prev": has_prev,
        "has_next": has_next,
        "prev_offset": prev_offset,
        "next_offset": next_offset,
    });
    super::render_page(&engine, "marketplace-versions", &data, &user_ctx, &mkt_ctx)
}
