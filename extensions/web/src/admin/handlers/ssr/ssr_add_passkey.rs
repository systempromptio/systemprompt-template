use crate::admin::templates::AdminTemplateEngine;
use crate::utils::html_escape;
use axum::{
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    Extension,
};
pub async fn add_passkey_page(
    Extension(engine): Extension<AdminTemplateEngine>,
) -> Response {
    match engine.render("add-passkey", &super::branding_context(&engine)) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!(error = ?e, "Add-passkey page render failed");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!(
                    "<h1>Error</h1><p>{}</p>",
                    html_escape(&e.to_string())
                )),
            )
                .into_response()
        }
    }
}
