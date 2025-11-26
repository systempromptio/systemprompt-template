//! Web API models for Agent module
//!
//! REST API models for Agent management. These models handle JSON REST API
//! requests and responses, separate from the core A2A protocol models to provide
//! clean JSON interfaces for web applications.

use crate::models::a2a::{AgentCapabilities, AgentCard, SecurityScheme, TransportProtocol};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use systemprompt_core_system::AppContext;

/// Internal structure for deserializing agent card with optional URL
#[derive(Debug, Clone, Deserialize)]
pub struct AgentCardInput {
    #[serde(default = "default_protocol_version")]
    pub protocol_version: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub url: Option<String>, // Optional for REST API
    pub version: String,
    #[serde(default)]
    pub preferred_transport: Option<TransportProtocol>,
    #[serde(default)]
    pub capabilities: AgentCapabilities,
    #[serde(default)]
    pub default_input_modes: Vec<String>,
    #[serde(default)]
    pub default_output_modes: Vec<String>,
    #[serde(default)]
    pub skills: Vec<crate::models::a2a::AgentSkill>,
    #[serde(default)]
    pub security_schemes: Option<HashMap<String, SecurityScheme>>,
    #[serde(default)]
    pub security: Option<Vec<HashMap<String, Vec<String>>>>,
}

/// Raw structure for deserializing CreateAgentRequest with optional URL
#[derive(Debug, Clone, Deserialize)]
pub struct CreateAgentRequestRaw {
    pub card: AgentCardInput,
    pub is_active: Option<bool>,
    pub system_prompt: Option<String>,
    pub mcp_servers: Option<Vec<String>>,
}

/// JSON request for creating a new agent via REST API
#[derive(Debug, Clone, Serialize)]
pub struct CreateAgentRequest {
    pub card: AgentCard,
    pub is_active: Option<bool>,
    pub system_prompt: Option<String>,
    pub mcp_servers: Option<Vec<String>>,
}

fn default_protocol_version() -> String {
    "0.3.0".to_string()
}

// Custom Deserialize implementation for CreateAgentRequest
impl<'de> Deserialize<'de> for CreateAgentRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = CreateAgentRequestRaw::deserialize(deserializer)?;

        // We can't access context here, so we'll use a placeholder
        // The actual URL generation will happen in the endpoint
        let url = raw
            .card
            .url
            .unwrap_or_else(|| format!("http://placeholder/api/v1/agents/{}", raw.card.name));

        let card = AgentCard {
            protocol_version: raw.card.protocol_version,
            name: raw.card.name,
            description: raw.card.description,
            url,
            version: raw.card.version,
            preferred_transport: raw.card.preferred_transport,
            additional_interfaces: None,
            icon_url: None,
            provider: None,
            documentation_url: None,
            capabilities: raw.card.capabilities.normalize(),
            security_schemes: raw.card.security_schemes,
            security: raw.card.security,
            default_input_modes: if raw.card.default_input_modes.is_empty() {
                vec!["text/plain".to_string()]
            } else {
                raw.card.default_input_modes
            },
            default_output_modes: if raw.card.default_output_modes.is_empty() {
                vec!["text/plain".to_string()]
            } else {
                raw.card.default_output_modes
            },
            skills: raw.card.skills,
            supports_authenticated_extended_card: None,
            signatures: None,
        };

        Ok(CreateAgentRequest {
            card,
            is_active: raw.is_active,
            system_prompt: raw.system_prompt,
            mcp_servers: raw.mcp_servers,
        })
    }
}

// Custom Deserialize implementation for UpdateAgentRequest
impl<'de> Deserialize<'de> for UpdateAgentRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = UpdateAgentRequestRaw::deserialize(deserializer)?;

        // We can't access context here, so we'll use a placeholder
        // The actual URL generation will happen in the endpoint
        let url = raw
            .card
            .url
            .unwrap_or_else(|| format!("http://placeholder/api/v1/agents/{}", raw.card.name));

        let card = AgentCard {
            protocol_version: raw.card.protocol_version,
            name: raw.card.name,
            description: raw.card.description,
            url,
            version: raw.card.version,
            preferred_transport: raw.card.preferred_transport,
            additional_interfaces: None,
            icon_url: None,
            provider: None,
            documentation_url: None,
            capabilities: raw.card.capabilities.normalize(),
            security_schemes: raw.card.security_schemes,
            security: raw.card.security,
            default_input_modes: if raw.card.default_input_modes.is_empty() {
                vec!["text/plain".to_string()]
            } else {
                raw.card.default_input_modes
            },
            default_output_modes: if raw.card.default_output_modes.is_empty() {
                vec!["text/plain".to_string()]
            } else {
                raw.card.default_output_modes
            },
            skills: raw.card.skills,
            supports_authenticated_extended_card: None,
            signatures: None,
        };

        Ok(UpdateAgentRequest {
            card,
            is_active: raw.is_active,
            system_prompt: raw.system_prompt,
            mcp_servers: raw.mcp_servers,
        })
    }
}

/// Raw structure for deserializing UpdateAgentRequest with optional URL
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAgentRequestRaw {
    pub card: AgentCardInput,
    pub is_active: Option<bool>,
    pub system_prompt: Option<String>,
    pub mcp_servers: Option<Vec<String>>,
}

/// JSON request for updating an existing agent via REST API
#[derive(Debug, Clone, Serialize)]
pub struct UpdateAgentRequest {
    pub card: AgentCard,
    pub is_active: Option<bool>,
    pub system_prompt: Option<String>,
    pub mcp_servers: Option<Vec<String>>,
}

impl UpdateAgentRequest {
    /// Create from raw input with context-aware URL generation
    pub fn from_raw(raw: UpdateAgentRequestRaw, ctx: &AppContext) -> Self {
        // Generate URL using the actual proxy endpoint if not provided
        let url = raw.card.url.unwrap_or_else(|| {
            let host = &ctx.config().api_server_url;
            format!("{}/api/v1/agents/{}", host, raw.card.name)
        });

        // Build proper AgentCard with required URL
        let card = AgentCard {
            protocol_version: raw.card.protocol_version,
            name: raw.card.name,
            description: raw.card.description,
            url,
            version: raw.card.version,
            preferred_transport: raw.card.preferred_transport,
            additional_interfaces: None,
            icon_url: None,
            provider: None,
            documentation_url: None,
            capabilities: raw.card.capabilities.normalize(),
            security_schemes: raw.card.security_schemes,
            security: raw.card.security,
            default_input_modes: if raw.card.default_input_modes.is_empty() {
                vec!["text/plain".to_string()]
            } else {
                raw.card.default_input_modes
            },
            default_output_modes: if raw.card.default_output_modes.is_empty() {
                vec!["text/plain".to_string()]
            } else {
                raw.card.default_output_modes
            },
            skills: raw.card.skills,
            supports_authenticated_extended_card: None,
            signatures: None,
        };

        UpdateAgentRequest {
            card,
            is_active: raw.is_active,
            system_prompt: raw.system_prompt,
            mcp_servers: raw.mcp_servers,
        }
    }
}

impl CreateAgentRequest {
    /// Create from raw input with context-aware URL generation
    pub fn from_raw(raw: CreateAgentRequestRaw, ctx: &AppContext) -> Self {
        // Generate URL using the actual proxy endpoint if not provided
        let url = raw.card.url.unwrap_or_else(|| {
            let host = &ctx.config().api_server_url;
            format!("{}/api/v1/agents/{}", host, raw.card.name)
        });

        // Build proper AgentCard with required URL
        let card = AgentCard {
            protocol_version: raw.card.protocol_version,
            name: raw.card.name,
            description: raw.card.description,
            url,
            version: raw.card.version,
            preferred_transport: raw.card.preferred_transport,
            additional_interfaces: None,
            icon_url: None,
            provider: None,
            documentation_url: None,
            capabilities: raw.card.capabilities.normalize(),
            security_schemes: raw.card.security_schemes,
            security: raw.card.security,
            default_input_modes: if raw.card.default_input_modes.is_empty() {
                vec!["text/plain".to_string()]
            } else {
                raw.card.default_input_modes
            },
            default_output_modes: if raw.card.default_output_modes.is_empty() {
                vec!["text/plain".to_string()]
            } else {
                raw.card.default_output_modes
            },
            skills: raw.card.skills,
            supports_authenticated_extended_card: None,
            signatures: None,
        };

        CreateAgentRequest {
            card,
            is_active: raw.is_active,
            system_prompt: raw.system_prompt,
            mcp_servers: raw.mcp_servers,
        }
    }

    /// Validate the request data
    pub async fn validate(&self) -> Result<(), String> {
        if self.card.name.trim().is_empty() {
            return Err("Name is required".to_string());
        }

        // URL validation (always present due to auto-generation)
        if !self.card.url.starts_with("http://") && !self.card.url.starts_with("https://") {
            return Err("URL must be a valid HTTP or HTTPS URL".to_string());
        }

        // Version validation if provided
        if !is_valid_version(&self.card.version) {
            return Err("Version must be in semantic version format (e.g., 1.0.0)".to_string());
        }

        // MCP server validation
        if let Some(ref mcp_servers) = self.mcp_servers {
            if !mcp_servers.is_empty() {
                let available_servers = list_available_mcp_servers().await?;
                let mut invalid_servers = Vec::new();

                for server in mcp_servers {
                    if !available_servers.contains(server) {
                        invalid_servers.push(server.clone());
                    }
                }

                if !invalid_servers.is_empty() {
                    return Err(format!(
                        "Invalid MCP server(s): {}. Available servers: {}",
                        invalid_servers.join(", "),
                        if available_servers.is_empty() {
                            "(none)".to_string()
                        } else {
                            available_servers.join(", ")
                        }
                    ));
                }
            }
        }

        Ok(())
    }

    /// Get the version from the card
    pub fn get_version(&self) -> String {
        self.card.version.clone()
    }

    /// Check if the agent should be active
    pub fn is_active(&self) -> bool {
        self.is_active.unwrap_or(true)
    }

    /// Extract port from endpoint URL
    pub fn extract_port(&self) -> u16 {
        extract_port_from_url(&self.card.url).unwrap_or(80)
    }

    /// Get capabilities from the card
    pub fn get_capabilities(&self) -> &AgentCapabilities {
        &self.card.capabilities
    }

    /// Get transport protocols from the card
    pub fn get_transport_protocols(&self) -> Option<&TransportProtocol> {
        self.card.preferred_transport.as_ref()
    }
}

impl UpdateAgentRequest {
    /// Validate the request data
    pub async fn validate(&self) -> Result<(), String> {
        if self.card.name.trim().is_empty() {
            return Err("Name is required".to_string());
        }

        if self.card.url.trim().is_empty() {
            return Err("Endpoint is required".to_string());
        }

        // Basic URL validation
        if !self.card.url.starts_with("http://") && !self.card.url.starts_with("https://") {
            return Err("Endpoint must be a valid HTTP or HTTPS URL".to_string());
        }

        // Version validation
        if !is_valid_version(&self.card.version) {
            return Err("Version must be in semantic version format (e.g., 1.0.0)".to_string());
        }

        // MCP server validation
        if let Some(ref mcp_servers) = self.mcp_servers {
            if !mcp_servers.is_empty() {
                let available_servers = list_available_mcp_servers().await?;
                let mut invalid_servers = Vec::new();

                for server in mcp_servers {
                    if !available_servers.contains(server) {
                        invalid_servers.push(server.clone());
                    }
                }

                if !invalid_servers.is_empty() {
                    return Err(format!(
                        "Invalid MCP server(s): {}. Available servers: {}",
                        invalid_servers.join(", "),
                        if available_servers.is_empty() {
                            "(none)".to_string()
                        } else {
                            available_servers.join(", ")
                        }
                    ));
                }
            }
        }

        Ok(())
    }

    /// Check if the agent should be active
    pub fn is_active(&self) -> bool {
        self.is_active.unwrap_or(true)
    }

    /// Extract port from endpoint URL
    pub fn extract_port(&self) -> u16 {
        extract_port_from_url(&self.card.url).unwrap_or(80)
    }
}

/// Query parameters for listing agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAgentsQuery {
    /// Page number (1-based)
    pub page: Option<i32>,
    /// Number of items per page
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
    /// Search term for agent names or descriptions
    pub search: Option<String>,
    /// Filter by agent status
    pub status: Option<String>,
    /// Filter by capability
    pub capability: Option<String>,
}

impl Default for ListAgentsQuery {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(20),
            offset: Some(0),
            search: None,
            status: None,
            capability: None,
        }
    }
}

/// Basic semantic version validation
fn is_valid_version(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return false;
    }

    parts.iter().all(|part| part.parse::<u32>().is_ok())
}

/// Extract port from URL
fn extract_port_from_url(url: &str) -> Option<u16> {
    // Simple port extraction without external dependencies
    if let Some(url_after_protocol) = url
        .strip_prefix("http://")
        .or_else(|| url.strip_prefix("https://"))
    {
        if let Some(host_part) = url_after_protocol.split('/').next() {
            if let Some(port_str) = host_part.split(':').nth(1) {
                return port_str.parse().ok();
            }
        }
        // Default ports
        if url.starts_with("https://") {
            Some(443)
        } else {
            Some(80)
        }
    } else {
        None
    }
}

/// Agent counts for API responses
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct AgentCounts {
    pub total: usize,
    pub active: usize,
    pub enabled: usize,
}

/// Agent discovery entry with both UUID and slug
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDiscoveryEntry {
    pub uuid: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub url: String,
    pub status: String,
    pub endpoint: String,
}

/// Discovery response listing available agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDiscoveryResponse {
    pub agents: Vec<AgentDiscoveryEntry>,
    pub total: usize,
}

/// List available MCP server names from services table
pub async fn list_available_mcp_servers() -> Result<Vec<String>, String> {
    // CRITICAL: Validate against MCP registry, not services table
    // Services table contains runtime state which may include non-registry servers
    use systemprompt_core_mcp::services::registry::manager::RegistryService;

    RegistryService::list_servers()
        .await
        .map_err(|e| format!("Failed to load MCP registry: {}", e))
}
