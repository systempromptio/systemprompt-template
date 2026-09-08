//! Pinned provider resources and server-only application configuration.

use crate::error::{AdminError, AdminResult};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    Atlassian,
    Github,
    Salesforce,
}

impl Provider {
    pub const ALL: [Self; 3] = [Self::Atlassian, Self::Github, Self::Salesforce];
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Atlassian => "atlassian",
            Self::Github => "github",
            Self::Salesforce => "salesforce",
        }
    }
    pub fn endpoint(self) -> String {
        match self {
            Self::Atlassian => "https://mcp.atlassian.com/v2/mcp".into(),
            Self::Github => "https://api.githubcopilot.com/mcp/".into(),
            Self::Salesforce => {
                "https://api.salesforce.com/platform/mcp/v1/platform/sobject-all".into()
            },
        }
    }
    pub fn callback(self) -> AdminResult<String> {
        let cfg = systemprompt::models::Config::get().map_err(AdminError::internal)?;
        Ok(format!(
            "{}/api/public/connectors/{}/callback",
            cfg.api_external_url.trim_end_matches('/'),
            self.slug()
        ))
    }
    pub fn configured(self) -> bool {
        systemprompt::loader::ServicesBootstrap::get().is_ok_and(|services| {
            services
                .mcp_servers
                .get(self.slug())
                .is_some_and(|server| server.enabled)
        })
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

pub(super) fn salesforce_domain() -> AdminResult<String> {
    let value = secret("salesforce_mcp_domain")?;
    let url = reqwest::Url::parse(&value).map_err(AdminError::internal)?;
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
