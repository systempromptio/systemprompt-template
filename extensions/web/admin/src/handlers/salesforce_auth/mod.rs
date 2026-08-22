//! "Sign in with Salesforce" — OAuth 2.0 / OIDC authorization-code login, plus
//! the typed accessor that mints a per-user Salesforce bearer for the Hosted
//! MCP server.
//!
//! - [`salesforce_start`] / [`salesforce_callback`] drive the browser login.
//! - [`salesforce_token_handler`] is the typed accessor core's external-MCP
//!   client calls to obtain a fresh `{access_token, instance_url}` bearer. The
//!   bearer is minted on demand via the RFC 7523 JWT-bearer grant
//!   ([`crate::services::salesforce_jwt_bearer`]) — no tokens are banked.
//!
//! Module layout: [`config`] (the loaded YAML), [`start`] (the authorize
//! redirect), [`callback`] (the handler: validate → identity → session),
//! [`identity`] (token exchange, claim gating, federated resolution),
//! [`tokens`] (token shapes, the code exchange, the accessor endpoint).

mod callback;
mod config;
mod identity;
mod start;
mod tokens;
mod unlink;

use std::sync::Arc;

use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use sqlx::PgPool;

use systemprompt::models::Config;
use systemprompt::oauth::SessionCreationService;

pub(crate) use callback::salesforce_callback;
pub use config::SalesforceConfig;
pub(crate) use config::{client_secret, salesforce_certificate, salesforce_private_key};
pub use identity::select_sf_username;
pub(crate) use start::salesforce_start;
pub(crate) use tokens::{post_token_request, salesforce_token_handler};
pub(crate) use unlink::salesforce_unlink;

pub(super) const STATE_COOKIE: &str = "sf_oauth_state";
const DEFAULT_REDIRECT: &str = "/admin";

// Why: Errors from the Salesforce OAuth/token plumbing. Logged once at the HTTP
// boundary; the browser only ever sees an opaque `?sso=<reason>`.
// Why: public because `salesforce_org` returns it across the crate boundary to
// the CLI extension; the SSO handlers still only surface it as `?sso=<reason>`.
#[derive(Debug, thiserror::Error)]
pub enum SalesforceError {
    #[error("Salesforce HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Salesforce token endpoint returned {status}: {body}")]
    TokenEndpoint {
        status: reqwest::StatusCode,
        body: String,
    },
    #[error("Salesforce userinfo endpoint returned {0}")]
    UserInfo(reqwest::StatusCode),
    #[error("SALESFORCE_PRIVATE_KEY is not set")]
    MissingPrivateKey,
    #[error("environment variable {name} is unusable: {source}")]
    Env {
        name: String,
        #[source]
        source: std::env::VarError,
    },
    #[error("system clock before epoch: {0}")]
    Clock(#[from] std::time::SystemTimeError),
    #[error("SALESFORCE_PRIVATE_KEY is not a valid RSA private key: {0}")]
    PrivateKey(#[source] jsonwebtoken::errors::Error),
    #[error("assertion signing failed: {0}")]
    Signing(#[source] jsonwebtoken::errors::Error),
    #[error("unreadable deploy result: {0}")]
    DeployResult(#[source] serde_json::Error),
    #[error("deploy zip {path}: {source}")]
    Zip {
        path: String,
        #[source]
        source: zip::result::ZipError,
    },
    #[error("Salesforce token store error: {0}")]
    Storage(#[from] systemprompt_web_shared::error::MarketplaceError),
    #[error("Salesforce token plumbing: {0}")]
    Internal(String),
}

/// Per-request dependencies for the Salesforce handlers, shared via an axum
/// `Extension`.
#[derive(Clone)]
pub struct SalesforceDeps {
    pub config: Arc<SalesforceConfig>,
    pub write_pool: Arc<PgPool>,
    pub session_service: Arc<SessionCreationService>,
}

impl std::fmt::Debug for SalesforceDeps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SalesforceDeps")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

pub(super) fn secure_flag() -> &'static str {
    let use_https = match Config::get() {
        Ok(c) => c.use_https,
        Err(e) => {
            tracing::error!(
                error = %e,
                "config unavailable while setting session cookie; defaulting to Secure — the cookie will be dropped by browsers on plain http"
            );
            true
        },
    };
    if use_https { "; Secure" } else { "" }
}

// Why: Reject anything that isn't a same-site absolute path, to avoid
// open-redirect.
pub(super) fn sanitize_redirect(raw: Option<String>) -> String {
    match raw {
        Some(r) if r.starts_with('/') && !r.starts_with("//") => r,
        _ => DEFAULT_REDIRECT.to_owned(),
    }
}

// Why: lint-ok: http-error — this *is* the SSO failure channel: a redirect back
// to the login page carrying the reason, not an HTTP error.
pub(super) fn login_error(reason: &str) -> Response {
    Redirect::to(&format!("/admin/login?sso={reason}")).into_response()
}

// Why: 32 random bytes as base64url-no-pad (43 chars) — a valid PKCE verifier
// and a fine CSRF nonce.
pub(super) fn random_url_safe() -> String {
    let bytes: [u8; 32] = rand::rng().random();
    URL_SAFE_NO_PAD.encode(bytes)
}

// Why: What the callback should do with the resolved Salesforce identity: mint
// a session (`Login`) or attach it to the already-signed-in user (`Link`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FlowMode {
    Login,
    Link,
}

// Why: Parse the state cookie into `(state, code_verifier, redirect_to, mode)`.
// The mode segment is append-only: a cookie without it (set by an older binary
// mid-flow) parses as a plain login.
pub(super) fn read_state_cookie(headers: &HeaderMap) -> Option<(String, String, String, FlowMode)> {
    let raw = headers
        .get_all("cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|s| s.split(';'))
        .map(str::trim)
        .find_map(|kv| kv.strip_prefix(&format!("{STATE_COOKIE}=")))?;
    let mut parts = raw.splitn(4, '|');
    let state = parts.next()?.to_owned();
    let verifier = parts.next()?.to_owned();
    let redirect = parts.next()?.to_owned();
    let mode = match parts.next() {
        Some("link") => FlowMode::Link,
        _ => FlowMode::Login,
    };
    Some((state, verifier, sanitize_redirect(Some(redirect)), mode))
}
