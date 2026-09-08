//! `GET /api/public/salesforce/token` — the typed accessor core's external-MCP
//! client calls to obtain a fresh per-user Salesforce bearer.

use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use super::{SalesforceDeps, SalesforceError};
use crate::error::{AdminError, AdminResult};
use crate::handlers::users::extract_mcp_accessor_user;
use crate::repositories::users::salesforce_identity;
use crate::services::salesforce_jwt_bearer;

// Why: only the fields the JWT-bearer flow consumes are modelled; Salesforce
// returns several more.
#[derive(Debug, Deserialize)]
pub(crate) struct SalesforceTokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub instance_url: Option<String>,
}

// Why: shared `application/x-www-form-urlencoded` POST against the Salesforce
// token endpoint.
pub(crate) async fn post_token_request(
    token_url: &str,
    body: String,
) -> Result<SalesforceTokenResponse, SalesforceError> {
    // Why: lint-ok: web-transport — exchanges an authorization code with
    // Salesforce.
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
    // Why: core deserializes only `access_token` and ignores the rest. Kept
    // because the Hosted MCP host is addressed per-instance.
    instance_url: String,
}

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

    // Why: no identity row means the caller was never linked. Core turns a 404
    // into "connect the provider account first", which is the actionable
    // message; a mint attempt with the wrong `sub` would fail opaquely instead.
    let Some(username) =
        salesforce_identity::find_username(&deps.write_pool, &session.user_id).await?
    else {
        return Err(AdminError::NotFound(
            "Salesforce account not linked".to_owned(),
        ));
    };

    // Why: not `?` — a mint failure is an upstream fault. 502 says Salesforce
    // refused, where a 500 would blame this server for Salesforce being down.
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
