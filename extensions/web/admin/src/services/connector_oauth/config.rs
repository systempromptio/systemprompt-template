//! Pinned provider resources and server-only application configuration.

use crate::error::{AdminError, AdminResult};
use serde::{Deserialize, Serialize};

/// A configured MCP connector.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(try_from = "String", into = "String")]
pub enum Provider {
    Atlassian,
    Github,
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
            Self::Generic(id) => id,
        }
    }
    pub fn endpoint(&self) -> String {
        match self {
            Self::Atlassian => return "https://mcp.atlassian.com/v2/mcp".into(),
            Self::Github => return "https://api.githubcopilot.com/mcp/".into(),
            Self::Generic(_) => {},
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
        systemprompt::loader::ServicesBootstrap::get().is_ok_and(|services| {
            services
                .mcp_servers
                .get(self.slug())
                .is_some_and(|server| server.enabled)
        })
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
