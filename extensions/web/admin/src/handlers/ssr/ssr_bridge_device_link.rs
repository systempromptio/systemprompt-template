use std::sync::Arc;

use axum::extract::{Extension, Form, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use systemprompt_web_shared::html_escape;

use crate::repositories::bridge_grp;
use crate::templates::AdminTemplateEngine;
use crate::types::UserContext;

use super::ssr_helpers::branding_context;

#[derive(Debug, Deserialize)]
pub struct DeviceLinkQuery {
    pub redirect: String,
}

#[derive(Debug, Deserialize)]
pub struct DeviceLinkApproveForm {
    pub redirect: String,
}

pub async fn device_link_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Query(query): Query<DeviceLinkQuery>,
) -> Response {
    let Some(host) = validate_loopback_redirect(&query.redirect) else {
        return bad_redirect_response(&query.redirect);
    };

    let mut data = branding_context(&engine);
    if let Some(obj) = data.as_object_mut() {
        obj.insert("user_email".to_string(), json!(user_ctx.email.to_string()));
        obj.insert("redirect".to_string(), json!(query.redirect));
        obj.insert("redirect_host".to_string(), json!(host));
    }

    match engine.render("bridge-device-link", &data) {
        Ok(html) => Html(html).into_response(),
        Err(e) => {
            tracing::error!(error = ?e, "Bridge device-link page render failed");
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

pub async fn device_link_approve(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Form(form): Form<DeviceLinkApproveForm>,
) -> Response {
    if validate_loopback_redirect(&form.redirect).is_none() {
        return bad_redirect_response(&form.redirect);
    }

    let issued = match bridge_grp::issue_exchange_code(&pool, &user_ctx.user_id).await {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "Failed to issue bridge exchange code");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html("<h1>Internal Error</h1><p>Failed to issue exchange code.</p>"),
            )
                .into_response();
        }
    };

    let sep = if form.redirect.contains('?') {
        '&'
    } else {
        '?'
    };
    let location = format!("{}{}code={}", form.redirect, sep, issued.code);
    Redirect::to(&location).into_response()
}

pub async fn device_link_deny(Form(form): Form<DeviceLinkApproveForm>) -> Response {
    if validate_loopback_redirect(&form.redirect).is_none() {
        return bad_redirect_response(&form.redirect);
    }
    let sep = if form.redirect.contains('?') {
        '&'
    } else {
        '?'
    };
    let location = format!("{}{}error=denied", form.redirect, sep);
    Redirect::to(&location).into_response()
}

fn validate_loopback_redirect(redirect: &str) -> Option<String> {
    let url = url::Url::parse(redirect).ok()?;
    if url.scheme() != "http" {
        return None;
    }
    let host = url.host_str()?;
    if host != "127.0.0.1" && host != "localhost" {
        return None;
    }
    let port = url.port()?;
    Some(format!("{host}:{port}"))
}

fn bad_redirect_response(redirect: &str) -> Response {
    tracing::warn!(
        redirect,
        "Rejected bridge device-link redirect (non-loopback)"
    );
    (
        StatusCode::BAD_REQUEST,
        Html(format!(
            "<h1>Invalid redirect</h1><p>Only http://127.0.0.1:PORT or http://localhost:PORT redirects are accepted. Got: <code>{}</code></p>",
            html_escape(redirect)
        )),
    )
        .into_response()
}
