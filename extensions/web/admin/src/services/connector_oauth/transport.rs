//! Bounded OAuth exchanges; provider bodies and credentials never enter logs.

use super::{Consent, Grant, Provider, config};
use crate::error::{AdminError, AdminResult};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::time::Duration;
use systemprompt::identifiers::ClientId;
type HttpClient = reqwest::Client; // Why: external OAuth boundary. lint-ok: web-transport

pub(super) fn client() -> AdminResult<HttpClient> {
    reqwest::Client::builder() // Why: external OAuth boundary. lint-ok: web-transport
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
        Provider::Generic(_) => Err(AdminError::Unavailable(
            "Use generic OAuth registration".into(),
        )),
        Provider::Github => Ok((
            Registration {
                client_id: ClientId::new(config::secret("github_mcp_client_id")?),
                client_secret: config::secret("github_mcp_client_secret")?,
            },
            "https://github.com/login/oauth/authorize".into(),
            "https://github.com/login/oauth/access_token".into(),
        )),
        Provider::Salesforce(_) => {
            let org = provider.salesforce_org()?;
            let base = org.domain()?;
            Ok((
                Registration {
                    client_id: ClientId::new(org.client_id()?),
                    client_secret: org.client_secret_value()?,
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
                    "client_name":"Systemprompt Systemprompt", "redirect_uris":[callback],
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

pub async fn authorize(consent: Consent<'_>, provider: Provider) -> AdminResult<(String, Grant)> {
    if !provider.configured() {
        return Err(AdminError::Unavailable("Connector not configured".into()));
    }
    if matches!(provider, Provider::Generic(_)) {
        return super::generic::authorize(consent, provider).await;
    }
    let Consent {
        user,
        state,
        verifier,
    } = consent;
    let callback = provider.callback()?;
    let (registration, authorize, token_endpoint) =
        registration(provider.clone(), &callback).await?;
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
    } else if provider.is_salesforce() {
        url.query_pairs_mut()
            .append_pair("scope", "mcp_api refresh_token openid");
    }
    Ok((
        url.to_string(),
        Grant {
            configuration_binding: String::new(),
            authorization_issuer: String::new(),
            token_auth_method: String::new(),
            user: user.into(),
            provider: provider.clone(),
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


pub(super) use super::tokens::refresh;
pub use super::tokens::{exchange, exchange_with_client, refresh_with_client};
