//! SSR page completing the bridge device-link flow.
//!
//! The redirect target is restricted to loopback: the bridge runs on the user's
//! own machine, and any non-loopback redirect would hand the link code to a
//! third party.
//!
//! `redirect` is optional. A CLI with no browser has nothing listening on
//! loopback, so it sends the user here without one and the approve step
//! *displays* the code for the user to copy back into the terminal.

use std::sync::Arc;

use axum::extract::{Extension, Form, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Redirect, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt_web_shared::html_escape;

use crate::error::{AdminHtmlError, AdminHtmlResult};
use crate::repositories::bridge;
use crate::services::bridge_profile::BRIDGE_BINARY;
use crate::templates::AdminTemplateEngine;
use crate::types::UserContext;

use super::ssr_helpers::branding_context;
use systemprompt_web_shared::BrandingConfig;

#[derive(Debug, Deserialize)]
pub(crate) struct DeviceLinkQuery {
    pub redirect: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DeviceLinkApproveForm {
    pub redirect: Option<String>,
}

// Why: unconfigured branding must stay a missing key rather than a null, so
// the template's `{{#if}}` guard behaves. `redirect`/`redirect_host` follow the
// same rule: absent, not empty, when there is no callback to return to.
#[derive(Debug, Serialize)]
struct DeviceLinkContext<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    branding: Option<&'a BrandingConfig>,
    user_email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect_host: Option<String>,
    code_ttl_seconds: i64,
}

#[derive(Debug, Serialize)]
struct DeviceCodeContext<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    branding: Option<&'a BrandingConfig>,
    approved: bool,
    user_email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    code: Option<String>,
    expires_in_seconds: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    login_command: Option<String>,
}

pub(crate) async fn device_link_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Query(query): Query<DeviceLinkQuery>,
) -> AdminHtmlResult<Response> {
    let redirect_host = match query.redirect.as_deref() {
        Some(redirect) => match validate_loopback_redirect(redirect) {
            Some(host) => Some(host),
            None => return Ok(bad_redirect_response(redirect)),
        },
        None => None,
    };

    let branding = branding_context(&engine).branding;

    let data = DeviceLinkContext {
        branding,
        user_email: user_ctx.email.to_string(),
        redirect: query.redirect,
        redirect_host,
        code_ttl_seconds: bridge::EXCHANGE_CODE_TTL_SECONDS,
    };
    let data = serde_json::to_value(&data).map_err(AdminHtmlError::internal)?;

    let html = engine.render("bridge-device-link", &data)?;
    Ok(Html(html).into_response())
}

pub(crate) async fn device_link_approve(
    Extension(user_ctx): Extension<UserContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Form(form): Form<DeviceLinkApproveForm>,
) -> AdminHtmlResult<Response> {
    if let Some(redirect) = form.redirect.as_deref()
        && validate_loopback_redirect(redirect).is_none()
    {
        return Ok(bad_redirect_response(redirect));
    }

    let issued = bridge::issue_exchange_code(&pool, &user_ctx.user_id).await?;

    let Some(redirect) = form.redirect else {
        let expires_in_seconds = (issued.expires_at - chrono::Utc::now())
            .num_seconds()
            .max(0);
        let login_command = format!(
            "{BRIDGE_BINARY} login --code {code}{gateway}",
            code = issued.code,
            gateway = gateway_suffix()
        );
        return render_code_page(
            &engine,
            &DeviceCodeContext {
                branding: branding_context(&engine).branding,
                approved: true,
                user_email: user_ctx.email.to_string(),
                code: Some(issued.code),
                expires_in_seconds,
                login_command: Some(login_command),
            },
        );
    };

    let sep = if redirect.contains('?') { '&' } else { '?' };
    let location = format!("{redirect}{sep}code={}", issued.code);
    Ok(Redirect::to(&location).into_response())
}

pub(crate) async fn device_link_deny(
    Extension(user_ctx): Extension<UserContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Form(form): Form<DeviceLinkApproveForm>,
) -> AdminHtmlResult<Response> {
    let Some(redirect) = form.redirect else {
        return render_code_page(
            &engine,
            &DeviceCodeContext {
                branding: branding_context(&engine).branding,
                approved: false,
                user_email: user_ctx.email.to_string(),
                code: None,
                expires_in_seconds: 0,
                login_command: None,
            },
        );
    };

    if validate_loopback_redirect(&redirect).is_none() {
        return Ok(bad_redirect_response(&redirect));
    }
    let sep = if redirect.contains('?') { '&' } else { '?' };
    let location = format!("{redirect}{sep}error=denied");
    Ok(Redirect::to(&location).into_response())
}

fn render_code_page(
    engine: &AdminTemplateEngine,
    ctx: &DeviceCodeContext<'_>,
) -> AdminHtmlResult<Response> {
    let data = serde_json::to_value(ctx).map_err(AdminHtmlError::internal)?;
    let html = engine.render("bridge-device-code", &data)?;
    Ok(Html(html).into_response())
}

// Why: `--gateway` is only worth printing when the server knows its own
// external URL; a wrong one is worse than the CLI's configured default.
fn gateway_suffix() -> String {
    systemprompt::models::Config::get().map_or_else(
        |_| String::new(),
        |c| format!(" --gateway {}", c.api_external_url.trim_end_matches('/')),
    )
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
    // Why: lint-ok: http-error — names the accepted redirect forms, which the
    // generic page cannot
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
