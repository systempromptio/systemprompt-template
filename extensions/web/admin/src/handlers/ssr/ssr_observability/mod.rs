//! `/admin/system/observability` — where this instance ships its telemetry.
//!
//! Core streams every signal to the collector the profile's
//! `observability.otlp` block names; this page shows that target as the
//! profile view carries it (endpoint, protocol, signals; never the headers).

use crate::env::AdminEnv;

use axum::extract::Extension;
use axum::response::Response;
use serde::Serialize;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::observability::find_declared_export;
use crate::repositories::observability::view::{ObservabilityView, build_view};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

const DOCS_URL: &str = "/documentation/enterprise-audit-observability";

#[derive(Debug, Serialize)]
struct ObservabilityPageData {
    page: &'static str,
    title: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    docs_url: &'static str,
    sync_url: &'static str,
    #[serde(flatten)]
    view: ObservabilityView,
}

pub(crate) async fn observability_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(env): Extension<AdminEnv>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let target = find_declared_export(&env);
    let page = ObservabilityPageData {
        page: "system-observability",
        title: "Observability",
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Sync", "/admin/sync"),
            BreadcrumbView::current("Observability"),
        ],
        docs_url: DOCS_URL,
        sync_url: "/admin/sync",
        view: build_view(target.as_ref()),
    };
    Ok(super::render_typed_page(
        &engine,
        "system-observability",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}
