//! Account identity for generic connectors, read from the issuer's OIDC
//! `userinfo` endpoint when the descriptor asks for it.

use super::Grant;
use crate::error::{AdminError, AdminResult};
// JSON: OIDC userinfo is an open claim set; only `sub`, `email` and `name`
// are read, and the body was already verified as JSON by `response::body`.
use serde_json::Value;
use systemprompt::models::mcp::deployment::ConnectorIdentity;

pub(super) async fn generic_identity(http: &reqwest::Client, grant: &mut Grant) -> AdminResult<()> {
    let wants_userinfo = grant
        .provider
        .settings()
        .is_some_and(|settings| settings.identity == Some(ConnectorIdentity::Userinfo));
    if !wants_userinfo {
        return Ok(());
    }
    let metadata = super::generic_discovery::metadata(http, &grant.provider).await?;
    let endpoint = metadata.userinfo_endpoint.as_deref().ok_or_else(|| {
        AdminError::Unavailable("OAuth issuer publishes no userinfo endpoint".into())
    })?;
    let url = super::generic_discovery::trusted_endpoint(&grant.provider, endpoint)?;
    let response = http
        .get(url)
        .bearer_auth(&grant.access_token)
        .send()
        .await
        .map_err(|_redacted_error| AdminError::Upstream("Provider identity unavailable".into()))?;
    let info = super::response::body(response).await?;
    info.get("sub")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AdminError::Upstream("Provider returned no verified account ID".into()))?
        .clone_into(&mut grant.account_id);
    info.get("email")
        .or_else(|| info.get("name"))
        .and_then(Value::as_str)
        .unwrap_or(&grant.account_id)
        .clone_into(&mut grant.account_name);
    Ok(())
}
