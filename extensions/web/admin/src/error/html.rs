//! The HTML face of [`AdminError`], for the server-rendered admin pages.

use axum::response::{Html, IntoResponse, Response};
use systemprompt_web_shared::html_escape;
use thiserror::Error;

use super::AdminError;

/// The HTML face of [`AdminError`], for the server-rendered admin pages.
///
/// A browser navigating to a page needs a page, not a JSON body — but the
/// status and the client-visible text come from the same classification either
/// way, so an SSR handler cannot accidentally disagree with an API handler
/// about what a given failure means. Unlike the hand-rolled error pages this
/// replaces, it renders the error's public message, so an internal cause
/// is logged rather than interpolated into the page.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct AdminHtmlError(pub AdminError);

impl IntoResponse for AdminHtmlError {
    fn into_response(self) -> Response {
        let status = self.0.status();
        self.0.log(status);
        let body = Html(format!(
            r#"<!DOCTYPE html>
<html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{reason}</title>
<link rel="stylesheet" href="/css/core/fonts.css">
<link rel="stylesheet" href="/css/admin-bundle.css">
</head><body style="display:grid;place-items:center;min-height:100vh;margin:0;background:var(--sp-bg-canvas)">
<main style="max-width:28rem;padding:2rem 2.5rem;background:var(--sp-bg-surface);border:1px solid var(--sp-border-subtle);border-radius:0 0.375rem 1.125rem 0;text-align:center">
<p style="font-size:2.5rem;margin:0" aria-hidden="true">{status_code}</p>
<h1 style="font-size:1.25rem;margin:0.5rem 0">{reason}</h1>
<p style="color:var(--sp-text-secondary)">{message}</p>
<p><a href="/admin/profile" style="color:var(--sp-accent-text)">&larr; Back to the dashboard</a></p>
</main></body></html>"#,
            status_code = status.as_u16(),
            reason = status.canonical_reason().unwrap_or("Error"),
            message = html_escape(&self.0.public_message())
        ));
        (status, body).into_response()
    }
}

impl AdminHtmlError {
    #[must_use]
    pub fn internal<E>(err: E) -> Self
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self(AdminError::Internal(err.into()))
    }
}

// Why: `?` in an SSR handler goes through whatever `AdminError` already knows
// how to absorb, so the two faces stay in step by construction.
impl<E: Into<AdminError>> From<E> for AdminHtmlError {
    fn from(value: E) -> Self {
        Self(value.into())
    }
}

/// The SSR counterpart to [`AdminResult`](super::AdminResult).
pub type AdminHtmlResult<T> = Result<T, AdminHtmlError>;
