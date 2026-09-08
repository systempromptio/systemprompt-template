//! Bounded OAuth exchanges; provider bodies and credentials never enter logs.

use super::{Grant, Provider, config};
use crate::error::{AdminError, AdminResult};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::Utc;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::time::Duration;
use systemprompt::identifiers::ClientId;

pub(super) fn client() -> AdminResult<reqwest::Client> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(AdminError::internal)
}

#[derive(Deserialize)]
struct Registration {
    client_id: ClientId,
    #[serde(default)]
    client_secret: String,
}

#[derive(Deserialize)]
pub(super) struct Metadata {
    pub(super) issuer: String,
    #[serde(rename = "authorization_endpoint")]
    authorization: String,
    #[serde(rename = "token_endpoint")]
    token: String,
    #[serde(rename = "registration_endpoint")]
    registration: Option<String>,
}

pub(super) fn atlassian_endpoint(value: &str) -> AdminResult<()> {
    let url = reqwest::Url::parse(value).map_err(AdminError::internal)?;
    if url.scheme() != "https"
        || !matches!(
            url.host_str(),
            Some("mcp.atlassian.com" | "auth.atlassian.com")
        )
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(AdminError::Unavailable(
            "Unsupported Atlassian authorization endpoint".into(),
        ));
    }
    Ok(())
}

async fn registration(
    provider: Provider,
    callback: &str,
) -> AdminResult<(Registration, String, String)> {
    match provider {
        Provider::Github => Ok((
            Registration {
                client_id: ClientId::new(config::secret("github_mcp_client_id")?),
                client_secret: config::secret("github_mcp_client_secret")?,
            },
            "https://github.com/login/oauth/authorize".into(),
            "https://github.com/login/oauth/access_token".into(),
        )),
        Provider::Salesforce => {
            let base = config::salesforce_domain()?;
            Ok((
                Registration {
                    client_id: ClientId::new(config::secret("salesforce_mcp_client_id")?),
                    client_secret: config::secret("salesforce_mcp_client_secret")?,
                },
                format!("{base}/services/oauth2/authorize"),
                format!("{base}/services/oauth2/token"),
            ))
        },
        Provider::Atlassian => {
            let metadata = super::discovery::atlassian_metadata().await?;
            atlassian_endpoint(&metadata.authorization)?;
            atlassian_endpoint(&metadata.token)?;
            let endpoint = metadata.registration.ok_or_else(|| {
                AdminError::Unavailable("Atlassian client registration unavailable".into())
            })?;
            atlassian_endpoint(&endpoint)?;
            let response = client()?
                .post(endpoint)
                .json(&serde_json::json!({
                    "client_name":"Systemprompt", "redirect_uris":[callback],
                    "grant_types":["authorization_code","refresh_token"], "response_types":["code"],
                    "token_endpoint_auth_method":"none",
                    "scope": super::discovery::ATLASSIAN_SCOPES
                }))
                .send()
                .await
                .map_err(|_redacted_error| {
                    AdminError::Upstream("Atlassian registration unavailable".into())
                })?;
            if !response.status().is_success() {
                return Err(AdminError::Upstream(
                    "Atlassian client registration rejected".into(),
                ));
            }
            let registration = response.json().await.map_err(|_redacted_error| {
                AdminError::Upstream("Invalid Atlassian registration".into())
            })?;
            Ok((registration, metadata.authorization, metadata.token))
        },
    }
}

pub async fn authorize(
    user: &str,
    provider: Provider,
    state: &str,
    verifier: String,
) -> AdminResult<(String, Grant)> {
    if !provider.configured() {
        return Err(AdminError::Unavailable("Connector not configured".into()));
    }
    let callback = provider.callback()?;
    let (registration, authorize, token_endpoint) = registration(provider, &callback).await?;
    let mut url = reqwest::Url::parse(&authorize).map_err(AdminError::internal)?;
    url.query_pairs_mut().extend_pairs([
        ("client_id", registration.client_id.as_str()),
        ("redirect_uri", &callback),
        ("response_type", "code"),
        ("state", state),
        ("code_challenge_method", "S256"),
        (
            "code_challenge",
            &URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes())),
        ),
    ]);
    if provider == Provider::Atlassian {
        url.query_pairs_mut()
            .append_pair("resource", &provider.endpoint())
            .append_pair("scope", super::discovery::ATLASSIAN_SCOPES);
    } else if provider == Provider::Salesforce {
        url.query_pairs_mut()
            .append_pair("scope", "mcp_api refresh_token openid");
    }
    Ok((
        url.to_string(),
        Grant {
            user: user.into(),
            provider,
            client: registration.client_id.to_string(),
            client_secret: registration.client_secret,
            verifier,
            access_token: String::new(),
            refresh_token: None,
            expires_at: 0,
            token_endpoint,
            generation: 0,
            session: None,
            auth_method: "oauth".into(),
            account_id: String::new(),
            account_name: String::new(),
            resource_id: String::new(),
            resource_name: String::new(),
            authorization_scheme: "Bearer".into(),
        },
    ))
}

#[derive(Deserialize)]
struct Tokens {
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    error: Option<String>,
}

async fn request(grant: &mut Grant, fields: &[(&str, &str)]) -> AdminResult<()> {
    let body = {
        let mut form = url::form_urlencoded::Serializer::new(String::new());
        form.extend_pairs(fields)
            .append_pair("client_id", &grant.client);
        if !grant.client_secret.is_empty() {
            form.append_pair("client_secret", &grant.client_secret);
        }
        if grant.provider == Provider::Atlassian {
            form.append_pair("resource", &grant.provider.endpoint());
        }
        form.finish()
    };
    let response = client()?
        .post(&grant.token_endpoint)
        .header("Accept", "application/json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|_redacted_error| {
            AdminError::Upstream("Connector authorization service unavailable".into())
        })?;
    let status = response.status();
    if status.is_server_error() || status.as_u16() == 429 {
        return Err(AdminError::Upstream(
            "Connector authorization service unavailable".into(),
        ));
    }
    let tokens: Tokens = response.json().await.map_err(|_redacted_error| {
        AdminError::Upstream("Invalid connector authorization response".into())
    })?;
    if tokens.error.as_deref() == Some("invalid_grant") || status.as_u16() == 401 {
        return Err(AdminError::Unauthorized(
            "Connector grant rejected; reconnect required".into(),
        ));
    }
    if !status.is_success() || tokens.error.is_some() {
        return Err(AdminError::Upstream(
            "Connector application authorization rejected".into(),
        ));
    }
    grant.access_token = tokens
        .access_token
        .filter(|t| !t.trim().is_empty())
        .ok_or_else(|| AdminError::Upstream("Connector returned no access token".into()))?;
    if let Some(refresh) = tokens.refresh_token {
        grant.refresh_token = Some(refresh);
    }
    // Why: Salesforce may omit expires_in. In that case refresh on the next use
    // instead of guessing how long the upstream session will stay valid.
    let seconds = tokens
        .expires_in
        .filter(|n| *n > 0 && *n <= 31_536_000)
        .or_else(|| (grant.provider == Provider::Salesforce).then_some(0))
        .ok_or_else(|| AdminError::Upstream("Connector returned no valid token expiry".into()))?;
    grant.expires_at = Utc::now().timestamp() + seconds;
    if grant.refresh_token.is_none() {
        return Err(AdminError::Unauthorized(
            "Connector did not grant refresh authorization".into(),
        ));
    }
    Ok(())
}

pub async fn exchange(grant: &mut Grant, code: &str) -> AdminResult<()> {
    let callback = grant.provider.callback()?;
    let verifier = grant.verifier.clone();
    request(
        grant,
        &[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &callback),
            ("code_verifier", &verifier),
        ],
    )
    .await?;
    grant.verifier.clear();
    Ok(())
}

pub(super) async fn refresh(grant: &mut Grant) -> AdminResult<()> {
    let token = grant.refresh_token.clone().ok_or_else(|| {
        AdminError::Unauthorized("Connector refresh authorization missing".into())
    })?;
    request(
        grant,
        &[("grant_type", "refresh_token"), ("refresh_token", &token)],
    )
    .await
}
