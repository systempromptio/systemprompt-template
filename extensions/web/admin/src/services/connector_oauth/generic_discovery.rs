//! Trusted OAuth discovery and bounded metadata parsing.
use super::Provider;
use crate::error::{AdminError, AdminResult};
use serde::Deserialize;
use url::Url;
#[derive(Deserialize)]
struct Resource {
    resource: String,
    authorization_servers: Vec<String>,
}
#[derive(Deserialize)]
pub(super) struct AuthorizationMetadata {
    pub(super) issuer: String,
    pub(super) authorization_endpoint: String,
    pub(super) token_endpoint: String,
    pub(super) registration_endpoint: Option<String>,
    #[serde(default)]
    pub(super) code_challenge_methods_supported: Vec<String>,
    #[serde(default)]
    pub(super) token_endpoint_auth_methods_supported: Vec<String>,
}
pub(super) fn trusted_endpoint(provider: &Provider, value: &str) -> AdminResult<Url> {
    let settings = provider
        .settings()
        .ok_or_else(|| AdminError::Unavailable("OAuth setup required".into()))?;
    validate_endpoint(&provider.endpoint(), &settings.authorization_origins, value)
}

pub fn validate_endpoint(resource: &str, origins: &[String], value: &str) -> AdminResult<Url> {
    let resource = Url::parse(resource).map_err(AdminError::internal)?;
    let url = Url::parse(value).map_err(AdminError::internal)?;
    let approved = url.origin() == resource.origin()
        || origins.iter().any(|origin| {
            Url::parse(origin).is_ok_and(|allowed| {
                allowed.origin() == url.origin()
                    && allowed.path() == "/"
                    && allowed.query().is_none()
                    && allowed.fragment().is_none()
                    && allowed.username().is_empty()
                    && allowed.password().is_none()
            })
        });
    if url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
        || !approved
    {
        return Err(AdminError::Unavailable(
            "OAuth endpoint is outside configured trusted origins".into(),
        ));
    }
    Ok(url)
}

async fn read<T: serde::de::DeserializeOwned>(
    http: &reqwest::Client,
    provider: &Provider,
    url: &str,
) -> AdminResult<T> {
    let response = http
        .get(trusted_endpoint(provider, url)?)
        .send()
        .await
        .map_err(|_redacted_error| AdminError::Unavailable("OAuth discovery unavailable".into()))?;
    if !response.status().is_success() {
        return Err(AdminError::Unavailable("OAuth discovery rejected".into()));
    }
    bounded_json(response).await
}

pub(super) async fn bounded_json<T: serde::de::DeserializeOwned>(
    mut response: reqwest::Response,
) -> AdminResult<T> {
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_redacted_error| AdminError::Upstream("OAuth response interrupted".into()))?
    {
        if bytes.len() + chunk.len() > 65_536 {
            return Err(AdminError::Upstream(
                "OAuth metadata exceeds size limit".into(),
            ));
        }
        bytes.extend_from_slice(&chunk);
    }
    serde_json::from_slice(&bytes)
        .map_err(|_redacted_error| AdminError::Upstream("Invalid OAuth metadata".into()))
}

pub(super) async fn metadata(
    http: &reqwest::Client,
    provider: &Provider,
) -> AdminResult<AuthorizationMetadata> {
    let resource_url = trusted_endpoint(provider, &provider.endpoint())?;
    let mut discovery = resource_url.clone();
    discovery.set_path(&format!(
        "/.well-known/oauth-protected-resource{}",
        resource_url.path().trim_end_matches('/')
    ));
    discovery.set_query(None);
    let resource: Resource = if let Ok(resource) = read(http, provider, discovery.as_str()).await {
        resource
    } else {
        discovery.set_path("/.well-known/oauth-protected-resource");
        read(http, provider, discovery.as_str()).await?
    };
    if Url::parse(&resource.resource).ok().as_ref() != Some(&resource_url) {
        return Err(AdminError::Unavailable(
            "OAuth protected resource mismatch".into(),
        ));
    }
    let issuer = resource
        .authorization_servers
        .first()
        .ok_or_else(|| AdminError::Unavailable("OAuth issuer missing".into()))?;
    let issuer_url = trusted_endpoint(provider, issuer)?;
    let mut endpoint = issuer_url.clone();
    endpoint.set_path(&format!(
        "/.well-known/oauth-authorization-server{}",
        issuer_url.path().trim_end_matches('/')
    ));
    endpoint.set_query(None);
    let result: AuthorizationMetadata =
        if let Ok(value) = read(http, provider, endpoint.as_str()).await {
            value
        } else {
            endpoint.set_path(&format!(
                "{}/.well-known/openid-configuration",
                issuer_url.path().trim_end_matches('/')
            ));
            read(http, provider, endpoint.as_str()).await?
        };
    if result.issuer != *issuer
        || !result
            .code_challenge_methods_supported
            .iter()
            .any(|m| m == "S256")
    {
        return Err(AdminError::Unavailable(
            "OAuth issuer mismatch or PKCE S256 unavailable".into(),
        ));
    }
    trusted_endpoint(provider, &result.authorization_endpoint)?;
    trusted_endpoint(provider, &result.token_endpoint)?;
    Ok(result)
}
