use super::super::ai::ToolModelOverrides;
use super::super::auth::{JwtAudience, Permission};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub name: String,
    pub port: u16,
    pub endpoint: String,
    pub enabled: bool,
    #[serde(default)]
    pub is_primary: bool,
    #[serde(default)]
    pub default: bool,
    pub card: AgentCardConfig,
    pub metadata: AgentMetadataConfig,
    #[serde(default)]
    pub oauth: OAuthConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCardConfig {
    pub protocol_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub display_name: String,
    pub description: String,
    pub version: String,
    #[serde(default = "default_transport")]
    pub preferred_transport: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<AgentProviderInfo>,
    #[serde(default)]
    pub capabilities: CapabilitiesConfig,
    #[serde(default = "default_input_modes")]
    pub default_input_modes: Vec<String>,
    #[serde(default = "default_output_modes")]
    pub default_output_modes: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_schemes: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub skills: Vec<AgentSkillConfig>,
    #[serde(default)]
    pub supports_authenticated_extended_card: bool,
}

/// Agent skill definition for A2A Agent Card.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSkillConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_modes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_modes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<serde_json::Value>>,
}

/// Information about the organization providing this agent.
///
/// This is metadata about the provider, not configuration for calling AI
/// providers. For AI provider configuration, see
/// `crates/modules/ai/src/services/providers/provider_factory.
/// rs::AiProviderConfig`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProviderInfo {
    pub organization: String,
    pub url: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitiesConfig {
    #[serde(default = "default_true")]
    pub streaming: bool,
    #[serde(default)]
    pub push_notifications: bool,
    #[serde(default = "default_true")]
    pub state_transition_history: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct AgentMetadataConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
    #[serde(default)]
    pub mcp_servers: Vec<String>,
    #[serde(default)]
    pub skills: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(default)]
    pub tool_model_overrides: ToolModelOverrides,
}

/// OAuth configuration for A2A agent authentication requirements.
///
/// Defines the permissions and audience required to access this agent.
/// Corresponds to the `security` field in the A2A `AgentCard` specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub scopes: Vec<Permission>,
    #[serde(default = "default_audience")]
    pub audience: JwtAudience,
}

impl AgentConfig {
    pub fn validate(&self, name: &str) -> anyhow::Result<()> {
        if self.name != name {
            anyhow::bail!(
                "Agent config key '{}' does not match name field '{}'",
                name,
                self.name
            );
        }

        if !self
            .name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            anyhow::bail!(
                "Agent name '{}' must be lowercase alphanumeric with hyphens only",
                self.name
            );
        }

        if self.name.len() < 3 || self.name.len() > 50 {
            anyhow::bail!(
                "Agent name '{}' must be between 3 and 50 characters",
                self.name
            );
        }

        if self.port == 0 {
            anyhow::bail!("Agent '{}' has invalid port {}", self.name, self.port);
        }

        Ok(())
    }

    pub fn extract_oauth_scopes_from_card(&mut self) {
        if let Some(security_vec) = &self.card.security {
            for security_obj in security_vec {
                if let Some(oauth2_scopes) = security_obj.get("oauth2").and_then(|v| v.as_array()) {
                    let mut permissions = Vec::new();
                    for scope_val in oauth2_scopes {
                        if let Some(scope_str) = scope_val.as_str() {
                            match scope_str {
                                "admin" => permissions.push(Permission::Admin),
                                "user" => permissions.push(Permission::User),
                                "service" => permissions.push(Permission::Service),
                                "a2a" => permissions.push(Permission::A2a),
                                "mcp" => permissions.push(Permission::Mcp),
                                "anonymous" => permissions.push(Permission::Anonymous),
                                _ => {},
                            }
                        }
                    }
                    if !permissions.is_empty() {
                        self.oauth.scopes = permissions;
                        self.oauth.required = true;
                    }
                }
            }
        }
    }

    pub fn construct_url(&self, base_url: &str) -> String {
        format!(
            "{}/api/v1/agents/{}",
            base_url.trim_end_matches('/'),
            self.name
        )
    }
}

impl Default for CapabilitiesConfig {
    fn default() -> Self {
        Self {
            streaming: true,
            push_notifications: false,
            state_transition_history: true,
        }
    }
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            required: false,
            scopes: Vec::new(),
            audience: JwtAudience::A2a,
        }
    }
}

fn default_transport() -> String {
    "JSONRPC".to_string()
}

fn default_input_modes() -> Vec<String> {
    vec!["text/plain".to_string()]
}

fn default_output_modes() -> Vec<String> {
    vec!["text/plain".to_string()]
}

const fn default_true() -> bool {
    true
}

const fn default_audience() -> JwtAudience {
    JwtAudience::A2a
}
