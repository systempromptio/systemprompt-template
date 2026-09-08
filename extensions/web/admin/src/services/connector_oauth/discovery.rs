//! OAuth discovery bound to the hosted Atlassian MCP resource.

use super::transport::{Metadata, atlassian_endpoint, client};
use crate::error::{AdminError, AdminResult};
use serde::Deserialize;

// Why: the knowledge bank reads through this grant and the BA skills write
// through it (createJiraIssue, createConfluencePage), so both directions
// are consented once. Widening the list invalidates every banked grant:
// users reconnect on /admin/profile.
pub const ATLASSIAN_SCOPES: &str = "read:me read:account offline_access email read:jira:agent-interface write:jira:agent-interface search:jira:agent-interface read:confluence:agent-interface write:confluence:agent-interface search:confluence:agent-interface";

#[derive(Deserialize)]
struct ResourceMetadata {
    resource: String,
    authorization_servers: Vec<String>,
    scopes_supported: Vec<String>,
}

pub(super) async fn atlassian_metadata() -> AdminResult<Metadata> {
    let resource = client()?
        .get("https://mcp.atlassian.com/.well-known/oauth-protected-resource/v2/mcp")
        .send()
        .await
        .map_err(|_redacted_error| {
            AdminError::Unavailable("Atlassian resource discovery unavailable".into())
        })?
        .error_for_status()
        .map_err(|_redacted_error| {
            AdminError::Unavailable("Atlassian resource discovery rejected".into())
        })?
        .json::<ResourceMetadata>()
        .await
        .map_err(|_redacted_error| {
            AdminError::Unavailable("Invalid Atlassian resource metadata".into())
        })?;
    if resource.resource != "https://mcp.atlassian.com/v2/mcp"
        || !ATLASSIAN_SCOPES
            .split_whitespace()
            .all(|scope| resource.scopes_supported.iter().any(|s| s == scope))
    {
        return Err(AdminError::Unavailable(
            "Atlassian resource does not advertise the required pilot scopes".into(),
        ));
    }
    let issuer = resource.authorization_servers.first().ok_or_else(|| {
        AdminError::Unavailable("Atlassian advertises no authorization server".into())
    })?;
    let issuer = url::Url::parse(issuer).map_err(AdminError::internal)?;
    atlassian_endpoint(issuer.as_str())?;
    let discovery = format!(
        "{}/.well-known/oauth-authorization-server{}",
        issuer.origin().ascii_serialization(),
        issuer.path().trim_end_matches('/')
    );
    let metadata = client()?
        .get(discovery)
        .send()
        .await
        .map_err(|_redacted_error| {
            AdminError::Unavailable("Atlassian authorization discovery unavailable".into())
        })?
        .error_for_status()
        .map_err(|_redacted_error| {
            AdminError::Unavailable("Atlassian authorization discovery rejected".into())
        })?
        .json::<Metadata>()
        .await
        .map_err(|_redacted_error| {
            AdminError::Unavailable("Invalid Atlassian authorization metadata".into())
        })?;
    if metadata.issuer != issuer.as_str() {
        return Err(AdminError::Unavailable(
            "Atlassian authorization issuer mismatch".into(),
        ));
    }
    Ok(metadata)
}
