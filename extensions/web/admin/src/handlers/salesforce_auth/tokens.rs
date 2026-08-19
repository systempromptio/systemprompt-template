//! Salesforce token shapes, the authorization-code exchange (used by the SSO
//! login callback), and the typed accessor endpoint core's external-MCP client
//! calls to obtain a fresh Salesforce bearer. The accessor no longer reads a
//! banked token — it mints one on demand via the RFC 7523 JWT-bearer grant.

use crate::error::{AdminError, AdminResult};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use super::config::SalesforceConfig;
use super::{SalesforceDeps, SalesforceError};
use crate::handlers::users::extract_mcp_accessor_user;
use crate::repositories::users::salesforce_identity;
use crate::services::salesforce_jwt_bearer;

// Why: The Salesforce `/services/oauth2/token` response. Only the fields both
// the authorization-code (login) and JWT-bearer (Hosted-MCP) flows consume are
// modelled; other fields are ignored.
#[derive(Debug, Deserialize)]
pub(crate) struct SalesforceTokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub instance_url: Option<String>,
}

// Why: Exchange an authorization code for the full token set.
pub(super) async fn exchange_code(
    cfg: &SalesforceConfig,
    code: &str,
    client_secret: &str,
    code_verifier: &str,
) -> Result<SalesforceTokenResponse, SalesforceError> {
    // Why: reqwest is built with `default-features = false`, so `.form()` is
    // unavailable — encode the body by hand.
    let body = format!(
        "grant_type=authorization_code&code={}&client_id={}&client_secret={}&redirect_uri={}&code_verifier={}",
        urlencoding::encode(code),
        urlencoding::encode(&cfg.consumer_key),
        urlencoding::encode(client_secret),
        urlencoding::encode(&cfg.redirect_uri),
        urlencoding::encode(code_verifier),
    );
    post_token_request(&cfg.token_url(), body).await
}

// Why: Shared `application/x-www-form-urlencoded` POST against a Salesforce
// token endpoint, used by both the code exchange and the refresh service.
pub(crate) async fn post_token_request(
    token_url: &str,
    body: String,
) -> Result<SalesforceTokenResponse, SalesforceError> {
    let resp = reqwest::Client::new()
        .post(token_url)
        .header(
            reqwest::header::CONTENT_TYPE,
            "application/x-www-form-urlencoded",
        )
        .body(body)
        .send()
        .await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(SalesforceError::TokenEndpoint { status, body });
    }
    Ok(resp.json().await?)
}

#[derive(Debug, Serialize)]
struct TokenResponse {
    access_token: String,
    instance_url: String,
}

// Why: `GET /api/public/salesforce/token` — the typed contract core's
// Salesforce-MCP bearer injection consumes. Authenticates the caller, mints a
// fresh bearer via the RFC 7523 JWT-bearer grant (acting as the caller's
// Salesforce username), and returns `{ access_token, instance_url }`.
pub(crate) async fn salesforce_token_handler(
    Extension(deps): Extension<SalesforceDeps>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    let session = extract_mcp_accessor_user(&headers)?;
    if !deps.config.is_usable() {
        return Err(AdminError::Unavailable(
            "Salesforce not configured".to_owned(),
        ));
    }

    // Why: The JWT-bearer `sub` must be the Salesforce Username (captured at SSO
    // login), not the login email. No identity row means the caller never linked
    // Salesforce — the entity gate should have kept the server out of their
    // manifest, so answer with a clean denial rather than a doomed mint attempt.
    let username = match salesforce_identity::find(&deps.write_pool, &session.user_id).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return Err(AdminError::Forbidden(
                "Salesforce account not linked".to_owned(),
            ));
        },
        Err(e) => {
            tracing::warn!(error = %e, user_id = %session.user_id, "Salesforce username lookup failed; falling back to email");
            session.email.as_str().to_owned()
        },
    };

    // Why: Why not `?`: a mint failure is an *upstream* fault. 502 tells the
    // accessor's caller that Salesforce refused, where a 500 would blame this
    // server for Salesforce being down.
    let fresh = salesforce_jwt_bearer::get_token(&deps.config, &username)
        .await
        // Why: lint-ok: error-adapt — deliberate 502 re-classification, see above
        .map_err(|e| AdminError::Upstream(format!("Salesforce token mint failed: {e}")))?;
    Ok(Json(TokenResponse {
        access_token: fresh.access_token,
        instance_url: fresh.instance_url,
    })
    .into_response())
}
