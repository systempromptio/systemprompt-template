//! Outbound Salesforce token acquisition via the RFC 7523 JWT-bearer grant.
//!
//! No token is banked: build and sign a short-lived JWT assertion with the
//! Connected App's private key, POST it to Salesforce's
//! `/services/oauth2/token` under
//! `grant_type=urn:ietf:params:oauth:grant-type:jwt-bearer`, and return the
//! fresh bearer. Every accessor call mints a new one, which is why this path
//! needs no refresh-token rotation machinery.
//!
//! Operational prerequisite: the Connected App must have the matching digital
//! certificate uploaded with "Use digital signatures" enabled, and the user
//! must be admin-pre-authorized. The private key is provisioned as
//! `SALESFORCE_PRIVATE_KEY` (PEM).

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

// Why: `username` is the Salesforce Username to act as (e.g.
// `ed.aa…@agentforce.com`), NOT the login email — the two differ and Salesforce
// matches `sub` on the Username.
pub(crate) async fn get_token(
    cfg: &SalesforceConfig,
    username: &str,
) -> Result<FreshToken, SalesforceError> {
    let private_key_pem = salesforce_private_key().ok_or(SalesforceError::MissingPrivateKey)?;

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let audience = cfg.jwt_bearer_audience().to_owned();

    let assertion = Assertion {
        iss: cfg.consumer_key.clone(),
        sub: username.to_owned(),
        aud: audience.clone(),
        exp: now + ASSERTION_TTL_SECS,
    };

    let key = EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
        .map_err(SalesforceError::PrivateKey)?;
    let signed = encode(&Header::new(Algorithm::RS256), &assertion, &key)
        .map_err(SalesforceError::Signing)?;

    let resp = post_token_request(&cfg.token_url(), assertion_form_body(&signed)).await?;

    // Why: the grant returns the instance the token is scoped to; fall back to
    // the org base if Salesforce omits it.
    let instance_url = resp
        .instance_url
        .filter(|u| !u.is_empty())
        .unwrap_or(audience);

    Ok(FreshToken {
        access_token: resp.access_token,
        instance_url,
    })
}

// Why: reqwest is built with `default-features = false`, so `.form()` is
// unavailable — encode the body by hand.
fn assertion_form_body(signed: &str) -> String {
    format!(
        "grant_type={}&assertion={}",
        urlencoding::encode("urn:ietf:params:oauth:grant-type:jwt-bearer"),
        urlencoding::encode(signed),
    )
}
