//! `/admin` — the console landing page.
//!
//! One health-and-money screen: what the instance spent, how much of it
//! failed, which models carried it, what is waiting on a human, and which
//! containers the money went to. Every tile is a link to the page that owns
//! the detail, carrying the window the reader is already looking at, so the
//! overview never becomes a place to read numbers off rather than a place to
//! start from.
//!
//! A non-console viewer is redirected to their profile rather than refused:
//! every page this shell offers them would refuse them anyway, so a 403 here
//! would be a dead end where a redirect is a door.

mod data;
mod kpis_view;
mod panels_view;
mod view;

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::AdminHtmlResult;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use view::OverviewRange;

// Why: `preset` is the same parameter the AI-activity pages carry, so a window
// chosen there survives a hop through the overview. Everything else those pages
// put on the query string is ignored here on purpose — the overview is
// instance-wide, and honouring a container filter halfway would show a scoped
// number under an unscoped heading.
#[derive(Debug, Default, Deserialize)]
pub(crate) struct OverviewQuery {
    pub preset: Option<String>,
}

pub(crate) async fn overview_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<OverviewQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Ok(Redirect::to("/admin/profile").into_response());
    }

    let range = OverviewRange::from_query(query.preset.as_deref());
    let loaded = data::load_overview(&pool, range).await;
    let page = view::overview_page(range, &loaded);

    Ok(super::render_typed_page(
        &engine, "overview", &page, &user_ctx, &mkt_ctx,
    ))
}
