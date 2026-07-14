use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::handlers::ssr::ssr_helpers::render_typed_page;
use crate::services::bridge_profile;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

pub(crate) async fn profile_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let data = bridge_profile::build_bridge_profile_data(pool, &user_ctx).await;
    render_typed_page(&engine, "profile", &data, &user_ctx, &mkt_ctx)
}
