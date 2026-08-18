//! The one place SSR page data is handed to the template engine.

use crate::error::AdminHtmlError;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::response::{Html, IntoResponse, Response};
use serde::Serialize;

use super::context::{BrandingShell, PageShell};

pub(crate) const fn branding_context(engine: &AdminTemplateEngine) -> BrandingShell<'_> {
    BrandingShell {
        branding: engine.branding(),
    }
}

pub(crate) fn render_typed_page<T: Serialize>(
    engine: &AdminTemplateEngine,
    template: &str,
    data: &T,
    user_ctx: &UserContext,
    mkt_ctx: &MarketplaceContext,
) -> Response {
    // Why: lint-ok: http-error — renders the page directly; the failure arm is
    // already the typed AdminHtmlError response.
    // JSON: the shell reads the page's own `page` key to pick its help text,
    // which needs the page context as data rather than as a type. This is the
    // only Value conversion on the SSR render path.
    let value = serde_json::to_value(data).unwrap_or_else(|e| {
        tracing::warn!(template, error = %e, "Failed to serialize SSR page data");
        serde_json::Value::Object(serde_json::Map::new())
    });
    let page_id = value.get("page").and_then(serde_json::Value::as_str);
    let shell = PageShell::new(engine.branding(), user_ctx, mkt_ctx, page_id, &value);

    match engine.render(template, &shell) {
        Ok(html) => Html(html).into_response(),
        Err(e) => AdminHtmlError::from(e).into_response(),
    }
}
