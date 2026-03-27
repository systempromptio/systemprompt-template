use std::path::Path;



use super::super::types::{AgentDetail, AgentSkillInfo, CreateAgentRequest, UpdateAgentRequest};
use crate::error::MarketplaceError;

const DEFAULT_AGENT_PORT: u16 = 9100;

pub fn list_agents(services_path: &Path) -> Result<Vec<AgentDetail>, MarketplaceError> {
    let agents_dir = services_path.join("agents");
    let mut agents = Vec::new();
    if !agents_dir.exists() {
        return Ok(agents);
    }
    for entry in std::fs::read_dir(&agents_dir)? {
        let entry = entry?;
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("yaml") && ext != Some("yml") {
            continue;
        }
        let content = std::fs::read_to_string(&path)?;
        let config: serde_yaml::Value = match serde_yaml::from_str(&content) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if let Some(agents_map) = config.get("agents").and_then(|a| a.as_mapping()) {
            for (key, val) in agents_map {
                if let Some(agent_id) = key.as_str() {
                    agents.push(parse_agent_detail(agent_id, val));
                }
            }
        }
    }
    agents.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    Ok(agents)
}

fn parse_agent_detail(agent_id: &str, val: &serde_yaml::Value) -> AgentDetail {
    AgentDetail {
        id: agent_id.to_string(),
        name: val
            .get("card")
            .and_then(|c| c.get("displayName"))
            .or_else(|| val.get("card").and_then(|c| c.get("name")))
            .and_then(|n| n.as_str())
            .unwrap_or(agent_id)
            .to_string(),
        description: val
            .get("card")
            .and_then(|c| c.get("description"))
            .and_then(|d| d.as_str())
            .unwrap_or("")
            .to_string(),
        enabled: val
            .get("enabled")
            .and_then(serde_yaml::Value::as_bool)
            .unwrap_or(true),
        is_primary: val
            .get("is_primary")
            .and_then(serde_yaml::Value::as_bool)
            .unwrap_or(false),
        show_in_ui: val
            .get("show_in_ui")
            .and_then(serde_yaml::Value::as_bool)
            .unwrap_or(false),
        system_prompt: val
            .get("metadata")
            .and_then(|m| m.get("systemPrompt"))
            .and_then(|s| s.as_str())
            .unwrap_or("")
            .to_string(),
        port: val
            .get("port")
            .and_then(serde_yaml::Value::as_u64)
            .map(|p| u16::try_from(p).unwrap_or(u16::MAX)),
        endpoint: val
            .get("endpoint")
            .and_then(|e| e.as_str())
            .map(std::string::ToString::to_string),
        mcp_servers: val
            .get("mcp_servers")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().map(ToString::to_string))
                    .collect()
            })
            .unwrap_or_else(Vec::new),
        skills: parse_agent_skills(val),
    }
}

fn parse_agent_skills(val: &serde_yaml::Value) -> Vec<AgentSkillInfo> {
    val.get("card")
        .and_then(|c| c.get("skills"))
        .and_then(|s| s.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|skill| {
                    Some(AgentSkillInfo {
                        id: skill.get("id")?.as_str()?.to_string(),
                        name: skill.get("name")?.as_str()?.to_string(),
                        description: skill.get("description")?.as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_else(|| Vec::new())
}

pub fn find_agent(
    services_path: &Path,
    agent_id: &str,
) -> Result<Option<AgentDetail>, MarketplaceError> {
    let agents = list_agents(services_path)?;
    Ok(agents.into_iter().find(|a| a.id.as_str() == agent_id))
}

pub fn create_agent(
    services_path: &Path,
    req: &CreateAgentRequest,
) -> Result<AgentDetail, MarketplaceError> {
    let agents_dir = services_path.join("agents");
    std::fs::create_dir_all(&agents_dir)?;
    let file_path = agents_dir.join(format!("{}.yaml", req.id));
    if file_path.exists() {
        return Err(MarketplaceError::Internal(format!(
            "Agent '{}' already exists",
            req.id
        )));
    }
    let yaml_content = format!(
        "agents:\n  {}:\n    name: {}\n    port: {}\n    endpoint: http://localhost:8080/api/v1/agents/{}\n    enabled: {}\n    dev_only: false\n    is_primary: false\n    default: false\n    card:\n      protocolVersion: 0.3.0\n      name: {}\n      displayName: {}\n      description: {}\n      version: 1.0.0\n      preferredTransport: JSONRPC\n      capabilities:\n        streaming: true\n        pushNotifications: false\n        stateTransitionHistory: false\n      defaultInputModes:\n      - text/plain\n      defaultOutputModes:\n      - text/plain\n      - application/json\n    metadata:\n      systemPrompt: |\n        {}\n      mcpServers: []\n      skills: []\n",
        req.id, req.id, DEFAULT_AGENT_PORT, req.id, req.enabled, req.name, req.name, req.description,
        req.system_prompt.replace('\n', "\n        ")
    );
    std::fs::write(&file_path, &yaml_content).map_err(|e| {
        MarketplaceError::Internal(format!(
            "Failed to write agent file: {}: {e}",
            file_path.display()
        ))
    })?;
    Ok(AgentDetail {
        id: req.id.clone(),
        name: req.name.clone(),
        description: req.description.clone(),
        enabled: req.enabled,
        is_primary: false,
        show_in_ui: false,
        system_prompt: req.system_prompt.clone(),
        port: Some(DEFAULT_AGENT_PORT),
        endpoint: Some(format!("http://localhost:8080/api/v1/agents/{}", req.id)),
        mcp_servers: vec![],
        skills: vec![],
    })
}

pub fn update_agent(
    services_path: &Path,
    agent_id: &str,
    req: &UpdateAgentRequest,
) -> Result<Option<AgentDetail>, MarketplaceError> {
    let agents_dir = services_path.join("agents");
    let Some(file_path) = find_agent_file(&agents_dir, agent_id)? else {
        return Ok(None);
    };
    let content = std::fs::read_to_string(&file_path)?;
    let mut doc: serde_yaml::Value = serde_yaml::from_str(&content)?;
    if let Some(agent_val) = doc.get_mut("agents").and_then(|a| a.get_mut(agent_id)) {
        if let Some(ref name) = req.name {
            if let Some(card) = agent_val.get_mut("card") {
                card["name"] = serde_yaml::Value::String(name.clone());
                card["displayName"] = serde_yaml::Value::String(name.clone());
            }
        }
        if let Some(ref desc) = req.description {
            if let Some(card) = agent_val.get_mut("card") {
                card["description"] = serde_yaml::Value::String(desc.clone());
            }
        }
        if let Some(enabled) = req.enabled {
            agent_val["enabled"] = serde_yaml::Value::Bool(enabled);
        }
        if let Some(ref prompt) = req.system_prompt {
            if let Some(metadata) = agent_val.get_mut("metadata") {
                metadata["systemPrompt"] = serde_yaml::Value::String(prompt.clone());
            }
        }
    }
    let yaml_str = serde_yaml::to_string(&doc)?;
    std::fs::write(&file_path, yaml_str).map_err(|e| {
        MarketplaceError::Internal(format!("Failed to write: {}: {e}", file_path.display()))
    })?;
    find_agent(services_path, agent_id)
}

pub fn delete_agent(services_path: &Path, agent_id: &str) -> Result<bool, MarketplaceError> {
    let agents_dir = services_path.join("agents");
    let Some(file_path) = find_agent_file(&agents_dir, agent_id)? else {
        return Ok(false);
    };
    let content = std::fs::read_to_string(&file_path)?;
    let doc: serde_yaml::Value = serde_yaml::from_str(&content)?;
    let count = doc
        .get("agents")
        .and_then(|a| a.as_mapping())
        .map_or(0, serde_yaml::Mapping::len);
    if count <= 1 {
        std::fs::remove_file(&file_path)?;
    } else {
        let mut doc: serde_yaml::Value = serde_yaml::from_str(&content)?;
        if let Some(agents) = doc.get_mut("agents").and_then(|a| a.as_mapping_mut()) {
            agents.remove(serde_yaml::Value::String(agent_id.to_string()));
        }
        std::fs::write(&file_path, serde_yaml::to_string(&doc)?)?;
    }
    Ok(true)
}

fn find_agent_file(
    agents_dir: &Path,
    agent_id: &str,
) -> Result<Option<std::path::PathBuf>, MarketplaceError> {
    if !agents_dir.exists() {
        return Ok(None);
    }
    for entry in std::fs::read_dir(agents_dir)? {
        let entry = entry?;
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("yaml") && ext != Some("yml") {
            continue;
        }
        let content = std::fs::read_to_string(&path)?;
        let config: serde_yaml::Value = match serde_yaml::from_str(&content) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if config.get("agents").and_then(|a| a.get(agent_id)).is_some() {
            return Ok(Some(path));
        }
    }
    Ok(None)
}
