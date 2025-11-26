pub mod agent_config;
pub mod settings;

pub use agent_config::*;
pub use settings::*;

use crate::mcp::Deployment;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Partial configuration fragment for include files (agents-only or mcp-only)
/// Include files don't have settings - those are only in the root config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartialServicesConfig {
    #[serde(default)]
    pub agents: HashMap<String, AgentConfig>,
    #[serde(default)]
    pub mcp_servers: HashMap<String, Deployment>,
}

/// Complete merged services configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesConfig {
    #[serde(default)]
    pub agents: HashMap<String, AgentConfig>,
    #[serde(default)]
    pub mcp_servers: HashMap<String, Deployment>,
    #[serde(default)]
    pub settings: Settings,
}

impl ServicesConfig {
    pub fn validate(&self) -> anyhow::Result<()> {
        self.validate_port_conflicts()?;
        self.validate_port_ranges()?;
        self.validate_mcp_port_ranges()?;
        self.validate_single_default_agent()?;

        for (name, agent) in &self.agents {
            agent.validate(name)?;
        }

        Ok(())
    }

    fn validate_port_conflicts(&self) -> anyhow::Result<()> {
        let mut seen_ports = HashMap::new();

        for (name, agent) in &self.agents {
            if let Some(existing) = seen_ports.insert(agent.port, ("agent", name.as_str())) {
                anyhow::bail!(
                    "Port conflict: {} used by both {} '{}' and agent '{}'",
                    agent.port,
                    existing.0,
                    existing.1,
                    name
                );
            }
        }

        for (name, mcp) in &self.mcp_servers {
            if let Some(existing) = seen_ports.insert(mcp.port, ("mcp_server", name.as_str())) {
                anyhow::bail!(
                    "Port conflict: {} used by both {} '{}' and mcp_server '{}'",
                    mcp.port,
                    existing.0,
                    existing.1,
                    name
                );
            }
        }

        Ok(())
    }

    fn validate_port_ranges(&self) -> anyhow::Result<()> {
        let (min, max) = self.settings.agent_port_range;

        for (name, agent) in &self.agents {
            if agent.port < min || agent.port > max {
                anyhow::bail!(
                    "Agent '{}' port {} is outside allowed range {}-{}",
                    name,
                    agent.port,
                    min,
                    max
                );
            }
        }

        Ok(())
    }

    fn validate_mcp_port_ranges(&self) -> anyhow::Result<()> {
        let (min, max) = self.settings.mcp_port_range;

        for (name, mcp) in &self.mcp_servers {
            if mcp.port < min || mcp.port > max {
                anyhow::bail!(
                    "MCP server '{}' port {} is outside allowed range {}-{}",
                    name,
                    mcp.port,
                    min,
                    max
                );
            }
        }

        Ok(())
    }

    fn validate_single_default_agent(&self) -> anyhow::Result<()> {
        let default_agents: Vec<&str> = self
            .agents
            .iter()
            .filter_map(|(name, agent)| {
                if agent.default {
                    Some(name.as_str())
                } else {
                    None
                }
            })
            .collect();

        match default_agents.len() {
            0 | 1 => Ok(()),
            _ => anyhow::bail!(
                "Multiple agents marked as default: {}. Only one agent can have 'default: true'",
                default_agents.join(", ")
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_single_default_agent_with_none() {
        let mut config = ServicesConfig {
            agents: HashMap::new(),
            mcp_servers: HashMap::new(),
            settings: Settings::default(),
        };

        let agent1 = create_test_agent("agent1", 9000, false);
        let agent2 = create_test_agent("agent2", 9001, false);
        config.agents.insert("agent1".to_string(), agent1);
        config.agents.insert("agent2".to_string(), agent2);

        assert!(config.validate_single_default_agent().is_ok());
    }

    #[test]
    fn test_validate_single_default_agent_with_one() {
        let mut config = ServicesConfig {
            agents: HashMap::new(),
            mcp_servers: HashMap::new(),
            settings: Settings::default(),
        };

        let agent1 = create_test_agent("agent1", 9000, true);
        let agent2 = create_test_agent("agent2", 9001, false);
        config.agents.insert("agent1".to_string(), agent1);
        config.agents.insert("agent2".to_string(), agent2);

        assert!(config.validate_single_default_agent().is_ok());
    }

    #[test]
    fn test_validate_single_default_agent_with_multiple() {
        let mut config = ServicesConfig {
            agents: HashMap::new(),
            mcp_servers: HashMap::new(),
            settings: Settings::default(),
        };

        let agent1 = create_test_agent("agent1", 9000, true);
        let agent2 = create_test_agent("agent2", 9001, true);
        config.agents.insert("agent1".to_string(), agent1);
        config.agents.insert("agent2".to_string(), agent2);

        let result = config.validate_single_default_agent();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Multiple agents marked as default"));
    }

    fn create_test_agent(name: &str, port: u16, default: bool) -> AgentConfig {
        AgentConfig {
            name: name.to_string(),
            port,
            endpoint: format!("http://localhost:{}", port),
            enabled: true,
            is_primary: false,
            default,
            card: AgentCardConfig {
                protocol_version: "0.3.0".to_string(),
                name: Some(name.to_string()),
                display_name: name.to_string(),
                description: "Test agent".to_string(),
                version: "1.0.0".to_string(),
                preferred_transport: "JSONRPC".to_string(),
                icon_url: None,
                documentation_url: None,
                provider: None,
                capabilities: CapabilitiesConfig::default(),
                default_input_modes: vec!["text/plain".to_string()],
                default_output_modes: vec!["text/plain".to_string()],
                security_schemes: None,
                security: None,
                skills: vec![],
                supports_authenticated_extended_card: false,
            },
            metadata: AgentMetadataConfig::default(),
            oauth: OAuthConfig::default(),
        }
    }
}
