//! Pinned provider resources and server-only application configuration.

use crate::error::{AdminError, AdminResult};
use crate::services::salesforce_orgs::{
    SalesforceOrg, SalesforceOrgRegistry, is_salesforce_server_id,
};
use serde::{Deserialize, Serialize};
use systemprompt::identifiers::McpServerId;

/// A configured MCP connector.
///
/// `Salesforce` carries its server id because each Salesforce org is its own
/// server (`salesforce`, `salesforce-<slug>`), and every org-specific value —
/// domain, app credentials, endpoint — is read from the org registry under
/// that id.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub enum Provider {
    Atlassian,
    Github,
    Salesforce(McpServerId),
    Generic(String),
}

impl TryFrom<String> for Provider {
    type Error = String;
    fn try_from(id: String) -> Result<Self, Self::Error> {
        if id.is_empty()
            || id.len() > 128
            || !id
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_'))
        {
            return Err("Invalid connector identifier".into());
        }
        Ok(match id.as_str() {
            "atlassian" => Self::Atlassian,
            "github" => Self::Github,
            _ if is_salesforce_server_id(&id) => Self::Salesforce(McpServerId::new(id)),
            _ => Self::Generic(id),
        })
    }
}
impl From<Provider> for String {
    fn from(provider: Provider) -> Self {
        provider.slug().to_owned()
    }
}
impl Provider {
    pub fn slug(&self) -> &str {
        match self {
            Self::Atlassian => "atlassian",
            Self::Github => "github",
            Self::Salesforce(id) => id.as_str(),
            Self::Generic(id) => id,
        }
    }
    pub const fn is_salesforce(&self) -> bool {
        matches!(self, Self::Salesforce(_))
    }
    pub fn salesforce_org(&self) -> AdminResult<&'static SalesforceOrg> {
        match self {
            Self::Salesforce(id) => SalesforceOrgRegistry::get().find(id).ok_or_else(|| {
                AdminError::Unavailable("Salesforce org is not in the org registry".into())
            }),
            _ => Err(AdminError::BadRequest("Not a Salesforce connector".into())),
        }
    }
    pub fn endpoint(&self) -> String {
        match self {
            Self::Atlassian => return "https://mcp.atlassian.com/v2/mcp".into(),
            Self::Github => return "https://api.githubcopilot.com/mcp/".into(),
            Self::Salesforce(_) | Self::Generic(_) => {},
        }
        systemprompt::loader::ServicesBootstrap::get()
            .ok()
            .and_then(|s| s.mcp_servers.get(self.slug()))
            .and_then(|server| server.endpoint.clone())
            .unwrap_or_default()
    }
    pub fn callback(&self) -> AdminResult<String> {
        let cfg = systemprompt::models::Config::get().map_err(AdminError::internal)?;
        Ok(format!(
            "{}/api/public/connectors/{}/callback",
            cfg.api_external_url.trim_end_matches('/'),
            self.slug()
        ))
    }
    pub fn configured(&self) -> bool {
        let enabled = systemprompt::loader::ServicesBootstrap::get().is_ok_and(|services| {
            services
                .mcp_servers
                .get(self.slug())
                .is_some_and(|server| server.enabled)
        });
        enabled && (!self.is_salesforce() || self.salesforce_org().is_ok())
    }
    pub fn provisioned(&self) -> bool {
        match self {
            Self::Salesforce(_) => self.salesforce_org().is_ok_and(SalesforceOrg::provisioned),
            _ => true,
        }
    }
    pub fn requires_auth(&self) -> bool {
        !matches!(self, Self::Generic(_)) || self.settings().is_some()
    }
    pub fn settings(&self) -> Option<systemprompt::models::mcp::deployment::ConnectorConfig> {
        systemprompt::loader::ServicesBootstrap::get()
            .ok()?
            .mcp_servers
            .get(self.slug())?
            .connector
            .clone()
    }
    pub fn display_name(&self) -> String {
        match self {
            Self::Atlassian => "Atlassian".into(),
            Self::Github => "GitHub".into(),
            Self::Salesforce(id) => self
                .salesforce_org()
                .map_or_else(|_| id.as_str().to_owned(), |org| org.label.clone()),
            Self::Generic(id) => id.clone(),
        }
    }
}

// Why: No provider secret is included in a service descriptor or client bundle.
pub(crate) fn secret(name: &str) -> AdminResult<String> {
    std::env::var(name.to_uppercase())
        .ok()
        .or_else(|| {
            systemprompt::config::SecretsBootstrap::get()
                .ok()?
                .get(name)
                .cloned()
        })
        .filter(|s| !s.is_empty() && !s.starts_with("REPLACE_WITH"))
        .ok_or_else(|| AdminError::Unavailable(format!("Connector configuration missing: {name}")))
}

pub(crate) fn validate_my_domain(value: &str) -> AdminResult<String> {
    let url = reqwest::Url::parse(value).map_err(AdminError::internal)?;
    let host = url.host_str().unwrap_or_default();
    if url.scheme() != "https"
        || !host.ends_with(".my.salesforce.com")
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err(AdminError::BadRequest(
            "Salesforce MCP domain must be an HTTPS My Domain origin".into(),
        ));
    }
    Ok(value.trim_end_matches('/').to_owned())
}
