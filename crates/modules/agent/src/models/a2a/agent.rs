//! A2A Agent domain types
//!
//! Agent card, capabilities, and skill definitions.

use super::auth::SecurityScheme;
use super::transport::TransportProtocol;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Agent Interface as specified in A2A spec section 5.5.5
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentInterface {
    pub url: String,
    pub transport: TransportProtocol,
}

/// Agent Provider as specified in A2A spec section 5.5.1
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentProvider {
    pub organization: String,
    pub url: String,
}

/// Agent Capabilities as specified in A2A spec section 5.5.2
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub streaming: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub push_notifications: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_transition_history: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<AgentExtension>>,
}

impl Default for AgentCapabilities {
    fn default() -> Self {
        Self {
            streaming: Some(true),          // Default to true for web chat support
            push_notifications: Some(true), // Enabled: A2A spec-compliant push notifications
            state_transition_history: Some(true),
            extensions: None,
        }
    }
}

impl AgentCapabilities {
    /// Normalize capabilities to ensure A2A spec-compliant defaults
    ///
    /// Applies sensible defaults per A2A specification section 5.5.2:
    /// - streaming: true (most agents support streaming)
    /// - push_notifications: false (opt-in feature)
    /// - state_transition_history: true (useful for debugging)
    pub const fn normalize(mut self) -> Self {
        if self.streaming.is_none() {
            self.streaming = Some(true);
        }
        if self.push_notifications.is_none() {
            self.push_notifications = Some(false);
        }
        if self.state_transition_history.is_none() {
            self.state_transition_history = Some(true);
        }
        self
    }
}

/// Agent Extension as specified in A2A spec section 5.5.2.1
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentExtension {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

impl AgentExtension {
    /// Creates an MCP tools extension declaration
    pub fn mcp_tools_extension() -> Self {
        Self {
            uri: "systemprompt:mcp-tools".to_string(),
            description: Some("MCP tool execution capabilities".to_string()),
            required: Some(false),
            params: Some(serde_json::json!({
                "supported_protocols": ["mcp-1.0"]
            })),
        }
    }

    /// Creates an MCP tools extension with server endpoint information
    pub fn mcp_tools_extension_with_servers(servers: Vec<serde_json::Value>) -> Self {
        Self {
            uri: "systemprompt:mcp-tools".to_string(),
            description: Some("MCP tool execution capabilities with server endpoints".to_string()),
            required: Some(false),
            params: Some(serde_json::json!({
                "supported_protocols": ["mcp-1.0"],
                "servers": servers
            })),
        }
    }

    /// Creates an OpenCode integration extension
    pub fn opencode_integration_extension() -> Self {
        Self {
            uri: "systemprompt:opencode-integration".to_string(),
            description: Some("OpenCode AI reasoning integration".to_string()),
            required: Some(false),
            params: Some(serde_json::json!({
                "reasoning_model": "claude-3-5-sonnet",
                "execution_mode": "structured_planning"
            })),
        }
    }

    /// Creates an artifact rendering extension declaration
    pub fn artifact_rendering_extension() -> Self {
        Self {
            uri: "https://systemprompt.io/extensions/artifact-rendering/v1".to_string(),
            description: Some(
                "MCP tool results rendered as typed artifacts with UI hints".to_string(),
            ),
            required: Some(false),
            params: Some(serde_json::json!({
                "supported_types": ["table", "form", "chart", "tree", "code", "json", "markdown"],
                "version": "1.0.0"
            })),
        }
    }

    /// Agent identity extension with platform name
    pub fn agent_identity(agent_name: String) -> Self {
        Self {
            uri: "systemprompt:agent-identity".to_string(),
            description: Some("SystemPrompt platform agent name".to_string()),
            required: Some(true),
            params: Some(serde_json::json!({
                "name": agent_name
            })),
        }
    }

    /// System instructions extension with agent prompt
    pub fn system_instructions(system_prompt: Option<String>) -> Option<Self> {
        system_prompt.map(|prompt| Self {
            uri: "systemprompt:system-instructions".to_string(),
            description: Some("Agent system prompt and behavioral guidelines".to_string()),
            required: Some(true),
            params: Some(serde_json::json!({
                "systemPrompt": prompt,
                "format": "text/plain"
            })),
        })
    }

    /// Service status extension with runtime information
    pub fn service_status(
        status: String,
        port: Option<u16>,
        pid: Option<u32>,
        default: bool,
    ) -> Self {
        let mut params = serde_json::json!({
            "status": status,
            "default": default
        });

        if let Some(p) = port {
            params["port"] = serde_json::json!(p);
        }
        if let Some(p) = pid {
            params["pid"] = serde_json::json!(p);
        }

        Self {
            uri: "systemprompt:service-status".to_string(),
            description: Some("Runtime service status from orchestrator".to_string()),
            required: Some(true),
            params: Some(params),
        }
    }
}

/// Agent Skill as specified in A2A spec section 5.5.4
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AgentSkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_modes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_modes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<HashMap<String, Vec<String>>>>,
}

impl AgentSkill {
    /// Create skill from MCP server metadata
    ///
    /// CRITICAL: This is the ONLY way to create skills in SystemPrompt.
    /// Skills are NEVER hardcoded - always derived from MCP registry.
    pub const fn from_mcp_server(
        server_name: String,
        display_name: String,
        description: String,
        tags: Vec<String>,
    ) -> Self {
        Self {
            id: server_name,
            name: display_name,
            description,
            tags,
            examples: None,
            input_modes: None,
            output_modes: None,
            security: None,
        }
    }

    pub fn mcp_server_name(&self) -> &str {
        &self.id
    }
}

/// Agent Card Signature as specified in A2A spec section 5.5.6
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentCardSignature {
    pub protected: String,
    pub signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<serde_json::Value>,
}

/// The main AgentCard as specified in A2A spec section 5.5
/// Fields are ordered to match the A2A protocol specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct AgentCard {
    #[serde(default = "default_protocol_version")]
    pub protocol_version: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preferred_transport: Option<TransportProtocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_interfaces: Option<Vec<AgentInterface>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<AgentProvider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_url: Option<String>,
    pub capabilities: AgentCapabilities,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security_schemes: Option<HashMap<String, SecurityScheme>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<Vec<HashMap<String, Vec<String>>>>,
    pub default_input_modes: Vec<String>,
    pub default_output_modes: Vec<String>,
    #[serde(default)]
    pub skills: Vec<AgentSkill>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supports_authenticated_extended_card: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signatures: Option<Vec<AgentCardSignature>>,
}

fn default_protocol_version() -> String {
    "0.3.0".to_string()
}

// Builder pattern for easy agent card creation
impl AgentCard {
    /// Creates a new AgentCard builder
    pub fn builder(
        name: String,
        description: String,
        url: String,
        version: String,
    ) -> AgentCardBuilder {
        AgentCardBuilder::new(name, description, url, version)
    }

    /// Check if card has MCP tools extension
    pub fn has_mcp_extension(&self) -> bool {
        self.capabilities
            .extensions
            .as_ref()
            .map(|exts| exts.iter().any(|ext| ext.uri == "systemprompt:mcp-tools"))
            .unwrap_or(false)
    }

    pub fn ensure_mcp_extension(&mut self) {
        if self.has_mcp_extension() {
            return;
        }

        self.capabilities
            .extensions
            .get_or_insert_with(Vec::new)
            .push(AgentExtension::mcp_tools_extension());
    }
}

#[derive(Debug)]
pub struct AgentCardBuilder {
    agent_card: AgentCard,
}

impl AgentCardBuilder {
    pub fn new(name: String, description: String, url: String, version: String) -> Self {
        Self {
            agent_card: AgentCard {
                protocol_version: "0.3.0".to_string(),
                name,
                description,
                url,
                version,
                preferred_transport: Some(TransportProtocol::JsonRpc),
                additional_interfaces: None,
                icon_url: None,
                provider: None,
                documentation_url: None,
                capabilities: AgentCapabilities::default(),
                security_schemes: None,
                security: None,
                default_input_modes: vec!["text/plain".to_string()],
                default_output_modes: vec!["text/plain".to_string()],
                skills: Vec::new(),
                supports_authenticated_extended_card: Some(false),
                signatures: None,
            },
        }
    }

    pub fn with_mcp_skills(
        mut self,
        mcp_servers: Vec<(String, String, String, Vec<String>)>,
    ) -> Self {
        for (server_name, display_name, description, tags) in mcp_servers {
            let skill = AgentSkill::from_mcp_server(server_name, display_name, description, tags);
            self.agent_card.skills.push(skill);
        }

        let mcp_extension = AgentExtension::mcp_tools_extension();
        let opencode_extension = AgentExtension::opencode_integration_extension();
        let artifact_rendering = AgentExtension::artifact_rendering_extension();

        self.agent_card.capabilities.extensions =
            Some(vec![mcp_extension, opencode_extension, artifact_rendering]);

        self
    }

    pub const fn with_streaming(mut self) -> Self {
        self.agent_card.capabilities.streaming = Some(true);
        self
    }

    pub const fn with_push_notifications(mut self) -> Self {
        self.agent_card.capabilities.push_notifications = Some(true);
        self
    }

    pub fn with_provider(mut self, organization: String, url: String) -> Self {
        self.agent_card.provider = Some(AgentProvider { organization, url });
        self
    }

    pub fn with_oauth2_security(
        mut self,
        authorization_url: String,
        token_url: String,
        scopes: HashMap<String, String>,
    ) -> Self {
        use super::auth::{OAuth2Flow, OAuth2Flows};

        let oauth2_flows = OAuth2Flows {
            authorization_code: Some(OAuth2Flow {
                authorization_url: Some(authorization_url),
                token_url: Some(token_url),
                refresh_url: None,
                scopes,
            }),
            implicit: None,
            password: None,
            client_credentials: None,
        };

        let oauth2_scheme = SecurityScheme::OAuth2 {
            flows: oauth2_flows,
            description: Some("OAuth 2.0 authorization code flow for secure access".to_string()),
        };

        self.agent_card
            .security_schemes
            .get_or_insert_with(HashMap::new)
            .insert("oauth2".to_string(), oauth2_scheme);

        let mut authentication_requirement = HashMap::new();
        authentication_requirement.insert(
            "oauth2".to_string(),
            vec!["admin".to_string(), "user".to_string()],
        );

        self.agent_card
            .security
            .get_or_insert_with(Vec::new)
            .push(authentication_requirement);

        self
    }

    pub fn build(self) -> AgentCard {
        self.agent_card
    }
}
