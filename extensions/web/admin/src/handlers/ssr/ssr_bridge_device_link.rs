//! SSR page completing the bridge device-link flow.
//!
//! The redirect target is restricted to loopback: the bridge runs on the user's
//! own machine, and any non-loopback redirect would hand the link code to a
//! third party.

use std::sync::Arc;

use axum::extract::{Extension, Form, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt_web_shared::html_escape;

use crate::error::{AdminHtmlError, AdminHtmlResult};
use crate::repositories::bridge;
use crate::templates::AdminTemplateEngine;
use crate::types::UserContext;

use super::ssr_helpers::branding_context;

#[derive(Debug, Deserialize)]
pub(crate) struct DeviceLinkQuery {
    pub redirect: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DeviceLinkApproveForm {
    pub redirect: String,
}

/// `branding` stays an untyped `Value` because the branding config shape is
/// not fixed at compile time. Unconfigured branding must stay a *missing* key,
/// not a null, so the template's `{{#if}}` guard behaves.
#[derive(Debug, Serialize)]
struct DeviceLinkContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    branding: Option<serde_json::Value>,
    user_email: String,
    redirect: String,
    redirect_host: String,
}

pub(crate) async fn device_link_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Query(query): Query<DeviceLinkQuery>,
) -> AdminHtmlResult<Response> {
    let Some(host) = validate_loopback_redirect(&query.redirect) else {
        return Ok(bad_redirect_response(&query.redirect));
    };

    let branding = branding_context(&engine)
        .as_object_mut()
        .and_then(|obj| obj.remove("branding"));

    let data = DeviceLinkContext {
        branding,
        user_email: user_ctx.email.to_string(),
        redirect: query.redirect,
        redirect_host: host,
    };
    let data = serde_json::to_value(&data).map_err(|e| {
        AdminHtmlError::internal(format!(
            "Failed to serialize bridge device-link context: {e}"
        ))
    })?;

    let html = engine.render("bridge-device-link", &data).map_err(|e| {
        AdminHtmlError::internal(format!("Bridge device-link page render failed: {e:?}"))
    })?;
    Ok(Html(html).into_response())
}

pub(crate) async fn device_link_approve(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Form(form): Form<DeviceLinkApproveForm>,
) -> AdminHtmlResult<Response> {
    if validate_loopback_redirect(&form.redirect).is_none() {
        return Ok(bad_redirect_response(&form.redirect));
    }

    let issued = bridge::issue_exchange_code(&pool, &user_ctx.user_id).await?;

    let sep = if form.redirect.contains('?') {
        '&'
    } else {
        '?'
    };
    let location = format!("{}{}code={}", form.redirect, sep, issued.code);
    Ok(Redirect::to(&location).into_response())
}

pub(crate) async fn device_link_deny(Form(form): Form<DeviceLinkApproveForm>) -> Response {
    // lint-ok: http-error — denying the link is this handler's success path
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
    // lint-ok: http-error — names the accepted redirect forms, which the generic
    // page cannot
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
