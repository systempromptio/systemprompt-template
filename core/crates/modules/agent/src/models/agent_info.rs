//! Agent information models for unified agent representation

use crate::models::a2a::{AgentCard, AgentSkill};
use serde::{Deserialize, Serialize};
use systemprompt_core_system::Config;

/// Unified agent information structure - thin wrapper around AgentCard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    /// Agent identifier (derived from extensions or URL)
    pub agent_id: String,
    /// Full agent card with capabilities and configuration
    pub card: AgentCard,
    /// Whether the agent is enabled/active (runtime state)
    pub enabled: bool,
    /// Dynamically loaded skills from MCP servers
    pub skills: Option<Vec<AgentSkill>>,
    /// MCP server names assigned to this agent
    pub mcp_servers: Option<Vec<String>>,
}

impl AgentInfo {
    /// Create AgentInfo from repository data
    pub fn from_repository_data(agent_id: String, card: AgentCard, enabled: bool) -> Self {
        Self {
            agent_id,
            card,
            enabled,
            skills: None,
            mcp_servers: None,
        }
    }

    /// Create AgentInfo from AgentCard (should be preferred method)
    pub fn from_card(agent_id: String, card: AgentCard, enabled: bool) -> Self {
        Self {
            agent_id,
            card,
            enabled,
            skills: None,
            mcp_servers: None,
        }
    }

    /// Get the agent ID
    pub fn id(&self) -> &str {
        &self.agent_id
    }

    /// Get agent name from card
    pub fn name(&self) -> &str {
        &self.card.name
    }

    /// Get agent endpoint from card
    pub fn endpoint(&self) -> &str {
        &self.card.url
    }

    /// Get full agent endpoint URL for display
    pub fn full_endpoint(&self) -> String {
        let endpoint = &self.card.url;
        if endpoint.starts_with('/') {
            let config = Config::global();
            format!("{}{}", config.api_external_url, endpoint)
        } else {
            endpoint.to_string()
        }
    }

    /// Get agent version from card
    pub fn version(&self) -> &str {
        &self.card.version
    }

    /// Set skills for this agent
    pub fn with_skills(mut self, skills: Vec<AgentSkill>) -> Self {
        self.skills = Some(skills);
        self
    }

    /// Set MCP servers for this agent
    pub fn with_mcp_servers(mut self, servers: Vec<String>) -> Self {
        self.mcp_servers = Some(servers);
        self
    }

    /// Get skills count
    pub fn skills_count(&self) -> usize {
        self.skills.as_ref().map(|s| s.len()).unwrap_or(0)
    }

    /// Get MCP servers count
    pub fn mcp_count(&self) -> usize {
        self.mcp_servers.as_ref().map(|s| s.len()).unwrap_or(0)
    }
}
