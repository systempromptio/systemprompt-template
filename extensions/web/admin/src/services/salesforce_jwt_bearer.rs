//! Outbound Salesforce token acquisition via the RFC 7523 JWT-bearer grant.
//!
//! Replaces per-user token *banking* with an on-demand exchange: build and sign
//! a short-lived JWT assertion with the Connected App's private key, POST it to
//! Salesforce's `/services/oauth2/token` under
//! `grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer`, and return the
//! fresh `{access_token, instance_url}` bearer the Salesforce Hosted-MCP host
//! needs. No tokens are stored: every call mints a fresh one.
//!
//! Operational prerequisite: the Connected App must have the matching digital
//! certificate uploaded with "Use digital signatures" enabled, and the user
//! must be admin-pre-authorized. The private key is provisioned as
//! `SALESFORCE_PRIVATE_KEY` (PEM) — see [`salesforce_private_key`].
//!
//! [`salesforce_private_key`]: crate::handlers::salesforce_auth::salesforce_private_key

use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{Algorithm, EncodingKey, Header, encode};
use serde::Serialize;

use crate::handlers::salesforce_auth::{
    SalesforceConfig, SalesforceError, post_token_request, salesforce_private_key,
};

pub(crate) struct FreshToken {
    pub access_token: String,
    pub instance_url: String,
}

// Why: Salesforce rejects an assertion whose `exp` is more than 5 minutes out.
const ASSERTION_TTL_SECS: u64 = 180;

#[derive(Debug, Serialize)]
struct Assertion {
    iss: String,
    sub: String,
    aud: String,
    exp: u64,
}

// Why: Mint a fresh Salesforce access token for `username` via the JWT-bearer
// grant.
//
// `username` is the Salesforce Username to act as — the userinfo
// `preferred_username` captured at SSO login (e.g. `ed.aa…@agentforce.com`),
// NOT the login email; the two differ and Salesforce matches `sub` on the
// Username. The External Client App must have the user admin-pre-authorized.
//
// # Errors
// - [`SalesforceError::MissingPrivateKey`] if `SALESFORCE_PRIVATE_KEY` is
// unset.
// - [`SalesforceError::Internal`] if the key is not valid PEM or signing
// fails.
// - [`SalesforceError::TokenEndpoint`] / [`SalesforceError::Http`] on a failed
// POST.
pub(crate) async fn get_token(
    cfg: &SalesforceConfig,
    username: &str,
) -> Result<FreshToken, SalesforceError> {
    let private_key_pem = salesforce_private_key().ok_or(SalesforceError::MissingPrivateKey)?;
    get_token_with_key(
        &cfg.consumer_key,
        username,
        cfg.jwt_bearer_audience(),
        &cfg.token_url(),
        &private_key_pem,
    )
    .await
}

// Why: The same grant against an arbitrary org rather than this deployment's
// configured one. Org provisioning targets orgs that have no `SalesforceConfig`
// and whose key is supplied per-invocation, so the credentials are parameters
// here instead of being read from the ambient config and secret store.
//
// # Errors
// - [`SalesforceError::PrivateKey`] if the key is not valid PEM,
// [`SalesforceError::Signing`] if signing fails.
// - [`SalesforceError::TokenEndpoint`] / [`SalesforceError::Http`] on a failed
// POST.
pub(crate) async fn get_token_with_key(
    consumer_key: &str,
    username: &str,
    audience: &str,
    token_url: &str,
    private_key_pem: &str,
) -> Result<FreshToken, SalesforceError> {
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

    let assertion = Assertion {
        iss: consumer_key.to_owned(),
        sub: username.to_owned(),
        aud: audience.to_owned(),
        exp: now + ASSERTION_TTL_SECS,
    };

    let key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
        .map_err(SalesforceError::PrivateKey)?;
    let signed = encode(&Header::new(Algorithm::RS256), &assertion, &key)
        .map_err(SalesforceError::Signing)?;

    let resp = post_token_request(token_url, assertion_form_body(&signed)).await?;

    // Why: The JWT-bearer grant returns the instance the token is scoped to; fall
    // back to the org base if Salesforce omits it.
    let instance_url = resp
        .instance_url
        .filter(|u| !u.is_empty())
        .unwrap_or_else(|| audience.to_owned());

    Ok(FreshToken {
        access_token: resp.access_token,
        instance_url,
    })
}

fn assertion_form_body(signed: &str) -> String {
    format!(
        "grant_type={}&assertion={}",
        urlencoding::encode("urn:ietf:params:oauth:grant-type:jwt-bearer"),
        urlencoding::encode(signed),
    )
}
