//! Pinned provider resources and server-only application configuration.

use crate::error::{AdminError, AdminResult};
use serde::{Deserialize, Serialize};
use systemprompt::manifest::services::ServicesConfig;
use systemprompt::models::auth::Permission;
use systemprompt::models::mcp::Deployment;

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
    pub fn endpoint(&self, services: &ServicesConfig) -> String {
        match self {
            Self::Atlassian => return "https://mcp.atlassian.com/v2/mcp".into(),
            Self::Github => return "https://api.githubcopilot.com/mcp/".into(),
            Self::Generic(_) => {},
        }
        services
            .mcp_servers
            .get(self.slug())
            .and_then(|server| server.endpoint.clone())
            .unwrap_or_default()
    }
    pub fn callback(&self, api_external_url: &str) -> String {
        format!(
            "{}/api/public/connectors/{}/callback",
            api_external_url.trim_end_matches('/'),
            self.slug()
        )
    }
    pub fn configured(&self, services: &ServicesConfig) -> bool {
        services
            .mcp_servers
            .get(self.slug())
            .is_some_and(|server| server.enabled)
    }
    pub fn requires_auth(&self, services: &ServicesConfig) -> bool {
        !matches!(self, Self::Generic(_))
            || self.settings(services).is_some()
            || self.is_session_attested(services)
    }
    fn server<'s>(&self, services: &'s ServicesConfig) -> Option<&'s Deployment> {
        services.mcp_servers.get(self.slug())
    }
    pub fn settings(
        &self,
        services: &ServicesConfig,
    ) -> Option<systemprompt::models::mcp::deployment::ConnectorConfig> {
        self.server(services)?.connector.clone()
    }
    // Why: a server that requires the platform's own OAuth and declares no
    // `connector:` block is authenticated by the caller's signed-in session,
    // not by a grant they consent to. Its `oauth.scopes` are the entitlement,
    // so `systemprompt` (scope `admin`) is a connector only an admin holds.
    pub fn session_scopes<'s>(&self, services: &'s ServicesConfig) -> Option<&'s [Permission]> {
        if !matches!(self, Self::Generic(_)) || self.settings(services).is_some() {
            return None;
        }
        let oauth = &self.server(services)?.oauth;
        (oauth.required && !oauth.scopes.is_empty()).then_some(oauth.scopes.as_slice())
    }
    pub fn is_session_attested(&self, services: &ServicesConfig) -> bool {
        self.session_scopes(services).is_some()
    }
    pub fn display_name(&self, services: &ServicesConfig) -> String {
        match self {
            Self::Atlassian => "Atlassian".into(),
            Self::Github => "GitHub".into(),
            Self::Generic(id) => self
                .settings(services)
                .and_then(|settings| settings.display_name)
                .unwrap_or_else(|| id.clone()),
        }
    }
}

// Why: No provider secret is included in a service descriptor or client bundle.
// A secret is read only by a name the web member declared and the profile
// granted it; any other name is absent.
pub(crate) fn secret(
    secrets: &systemprompt::traits::DeclaredSecrets,
    name: &str,
) -> AdminResult<String> {
    secrets
        .secret(name)
        .map(|s| s.expose().to_owned())
        .filter(|s| !s.is_empty() && !s.starts_with("REPLACE_WITH"))
        .ok_or_else(|| AdminError::Unavailable(format!("Connector configuration missing: {name}")))
}
