//! SSR page for enrolling a `WebAuthn` passkey.

use crate::error::{AdminHtmlError, AdminHtmlResult};
use crate::templates::AdminTemplateEngine;
use axum::Extension;
use axum::response::{Html, IntoResponse, Response};

pub(crate) async fn add_passkey_page(
    Extension(engine): Extension<AdminTemplateEngine>,
) -> AdminHtmlResult<Response> {
    let html = engine
        .render("add-passkey", &super::branding_context(&engine))
        .map_err(|e| AdminHtmlError::internal(format!("Add-passkey page render failed: {e:?}")))?;
    Ok(Html(html).into_response())
}
