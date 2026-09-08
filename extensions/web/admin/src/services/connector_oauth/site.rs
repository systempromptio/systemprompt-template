//! Match a browser-selected Atlassian site to the grant's authorized cloud IDs.

use super::Grant;
use super::transport::client;
use crate::error::{AdminError, AdminResult};
use serde::Deserialize;
// JSON: the MCP site catalog has legacy and v2 shapes normalized by payload.rs.
use serde_json::Value;

pub fn tenant_metadata_url(site: &str) -> AdminResult<url::Url> {
    let mut url = url::Url::parse(site).map_err(|_redacted_error| {
        AdminError::BadRequest("Enter an HTTPS Atlassian site address".into())
    })?;
    let tenant = url
        .host_str()
        .and_then(|host| host.strip_suffix(".atlassian.net"));
    if url.scheme() != "https"
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
        || !tenant.is_some_and(|t| {
            !t.is_empty() && t.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
        })
    {
        return Err(AdminError::BadRequest(
            "Use your site origin, such as https://example.atlassian.net".into(),
        ));
    }
    url.set_path("/_edge/tenant_info");
    Ok(url)
}

#[derive(Deserialize)]
struct Tenant {
    #[serde(rename = "cloudId")]
    cloud_id: String,
}

// JSON: only a cloud ID present in the authenticated MCP resource list may be
// selected.
pub(super) async fn select(grant: &mut Grant, sites: &[Value]) -> AdminResult<()> {
    let selected = grant.resource_id.clone();
    let cloud_id = if selected.starts_with("https://") {
        let endpoint = tenant_metadata_url(&selected)?;
        // Why: tenant metadata is public; never send the user's grant to a site
        // hostname.
        client()?
            .get(endpoint)
            .send()
            .await
            .map_err(|_redacted_error| {
                AdminError::Unavailable("Atlassian site lookup unavailable".into())
            })?
            .error_for_status()
            .map_err(|_redacted_error| {
                AdminError::BadRequest("Atlassian site address could not be resolved".into())
            })?
            .json::<Tenant>()
            .await
            .map_err(|_redacted_error| {
                AdminError::Unavailable("Atlassian site lookup returned no cloud ID".into())
            })?
            .cloud_id
    } else {
        selected.clone()
    };
    let site = if cloud_id.is_empty() && sites.len() == 1 {
        sites.first()
    } else {
        sites
            .iter()
            .find(|s| s.get("id").and_then(Value::as_str) == Some(cloud_id.as_str()))
    }
    .ok_or_else(|| {
        AdminError::Forbidden(
            "The selected Atlassian site is not in this account's authorized resources".into(),
        )
    })?;
    grant.resource_id = site
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| AdminError::Unavailable("Atlassian resource has no cloud ID".into()))?
        .into();
    grant.resource_name = if selected.starts_with("https://") {
        selected.trim_end_matches('/').into()
    } else {
        site.get("url")
            .and_then(Value::as_str)
            .unwrap_or_else(|| {
                if grant.resource_name.is_empty() {
                    &grant.resource_id
                } else {
                    &grant.resource_name
                }
            })
            .to_owned()
    };
    Ok(())
}
