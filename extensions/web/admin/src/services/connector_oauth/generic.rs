//! OAuth for configured MCP resources with explicitly trusted authorization
//! origins.

pub use super::generic_discovery::validate_endpoint;
use super::generic_discovery::{bounded_json, metadata, trusted_endpoint};
use super::transport::client;
use super::{Grant, Provider, config};
use crate::error::{AdminError, AdminResult};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use systemprompt::identifiers::ClientId;

#[derive(Deserialize)]
struct Registration {
    client_id: ClientId,
    #[serde(default)]
    client_secret: String,
}

pub fn binding(provider: &Provider) -> AdminResult<String> {
    let settings = provider
        .settings()
        .ok_or_else(|| AdminError::Unavailable("OAuth setup required".into()))?;
    let bytes =
        serde_json::to_vec(&(provider.endpoint(), settings)).map_err(AdminError::internal)?;
    Ok(URL_SAFE_NO_PAD.encode(Sha256::digest(bytes)))
}

pub fn validate_grant(grant: &Grant) -> AdminResult<()> {
    if matches!(grant.provider, Provider::Generic(_)) {
        if grant.configuration_binding != binding(&grant.provider)? {
            return Err(AdminError::Unauthorized(
                "Connector configuration changed; reconnect required".into(),
            ));
        }
        trusted_endpoint(&grant.provider, &grant.token_endpoint)?;
    }
    Ok(())
}

/// One consent request: who is consenting and the PKCE state that binds the
/// callback to it.
#[derive(Debug, Clone)]
pub struct Consent<'a> {
    pub user: &'a str,
    pub state: &'a str,
    pub verifier: String,
}

pub async fn authorize(consent: Consent<'_>, provider: Provider) -> AdminResult<(String, Grant)> {
    let callback = provider.callback()?;
    authorize_with_client(consent, provider, &callback, &client()?).await
}

// Why: the callback and transport are injectable so the OAuth boundary can be
// exercised against a local server in tests.
pub async fn authorize_with_client(
    consent: Consent<'_>,
    provider: Provider,
    callback: &str,
    http: &reqwest::Client, // Why: external OAuth boundary. lint-ok: web-transport
) -> AdminResult<(String, Grant)> {
    let Consent {
        user,
        state,
        verifier,
    } = consent;
    let settings = provider
        .settings()
        .ok_or_else(|| AdminError::Unavailable("OAuth setup required".into()))?;
    if settings.adapter != "generic" {
        return Err(AdminError::Unavailable(
            "Unsupported connector adapter".into(),
        ));
    }
    let meta = metadata(http, &provider).await?;
    let scope = settings.scopes.join(" ");
    let (registration, method) = register_client(&provider, &meta, callback, http).await?;
    if registration.client_id.as_str().is_empty() {
        return Err(AdminError::Unavailable(
            "OAuth client registration has no identifier".into(),
        ));
    }
    let mut url = trusted_endpoint(&provider, &meta.authorization_endpoint)?;
    url.query_pairs_mut().extend_pairs([
        ("response_type", "code"),
        ("client_id", registration.client_id.as_str()),
        ("redirect_uri", callback),
        ("state", state),
        ("code_challenge_method", "S256"),
        (
            "code_challenge",
            URL_SAFE_NO_PAD
                .encode(Sha256::digest(verifier.as_bytes()))
                .as_str(),
        ),
        ("resource", provider.endpoint().as_str()),
        ("scope", scope.as_str()),
    ]);
    let grant = Grant {
        user: user.into(),
        configuration_binding: binding(&provider)?,
        authorization_issuer: meta.issuer,
        token_auth_method: method.into(),
        provider,
        client: registration.client_id.to_string(),
        client_secret: registration.client_secret,
        verifier,
        token_endpoint: meta.token_endpoint,
        access_token: String::new(),
        refresh_token: None,
        expires_at: 0,
        generation: 0,
        session: None,
        auth_method: "oauth".into(),
        account_id: String::new(),
        account_name: String::new(),
        resource_id: String::new(),
        resource_name: String::new(),
        authorization_scheme: "Bearer".into(),
    };
    Ok((url.into(), grant))
}
pub async fn validate_refresh(
    grant: &Grant,
    http: &reqwest::Client, // Why: external OAuth boundary. lint-ok: web-transport
) -> AdminResult<()> {
    validate_grant(grant)?;
    let metadata = metadata(http, &grant.provider).await?;
    if (metadata.issuer.as_str(), metadata.token_endpoint.as_str())
        != (
            grant.authorization_issuer.as_str(),
            grant.token_endpoint.as_str(),
        )
    {
        return Err(AdminError::Unauthorized(
            "OAuth issuer changed; reconnect required".into(),
        ));
    }
    Ok(())
}

async fn register_client(
    provider: &Provider,
    meta: &super::generic_discovery::AuthorizationMetadata,
    callback: &str,
    http: &reqwest::Client, // Why: external OAuth boundary. lint-ok: web-transport
) -> AdminResult<(Registration, &'static str)> {
    let settings = provider
        .settings()
        .ok_or_else(|| AdminError::Unavailable("OAuth setup required".into()))?;
    let scope = settings.scopes.join(" ");
    let registration = if let Some(key) = &settings.client_id_secret {
        let secret = settings
            .client_secret
            .as_deref()
            .map(config::secret)
            .transpose()?
            .unwrap_or_default();
        let method = if secret.is_empty() {
            "none"
        } else if meta
            .token_endpoint_auth_methods_supported
            .iter()
            .any(|m| m == "client_secret_post")
        {
            "client_secret_post"
        } else {
            "client_secret_basic"
        };
        (
            Registration {
                client_id: ClientId::new(config::secret(key)?),
                client_secret: secret,
            },
            method,
        )
    } else {
        let endpoint = meta.registration_endpoint.as_deref().ok_or_else(|| {
            AdminError::Unavailable("Configure an OAuth client for this server".into())
        })?;
        let response = http
            .post(trusted_endpoint(provider, endpoint)?)
            .json(&serde_json::json!({
                "client_name": "Systemprompt Digital", "redirect_uris": [callback],
                "grant_types": ["authorization_code", "refresh_token"], "response_types": ["code"],
                "token_endpoint_auth_method": "none", "scope": scope
            }))
            .send()
            .await
            .map_err(|_redacted_error| {
                AdminError::Unavailable("OAuth registration unavailable".into())
            })?;
        if !response.status().is_success() {
            return Err(AdminError::Unavailable(
                "OAuth client registration rejected".into(),
            ));
        }
        (bounded_json::<Registration>(response).await?, "none")
    };
    Ok(registration)
}
