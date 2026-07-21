use std::collections::HashMap;
use std::path::Path;

use crate::repositories::plugins_grp::plugins::list_skill_catalog;
use crate::types::{AgentDetail, AgentSkillInfo};
use systemprompt::identifiers::{AgentId, McpServerId};
use systemprompt_web_shared::error::MarketplaceError;

pub fn list_agents(services_path: &Path) -> Result<Vec<AgentDetail>, MarketplaceError> {
    let agents_dir = services_path.join("agents");
    let mut agents = Vec::new();
    if !agents_dir.exists() {
        return Ok(agents);
    }
    // Skill metadata is sourced once from the skill catalog and looked up by id;
    // agents only declare flat `metadata.skills: [id]` and never duplicate
    // name/description.
    let skill_catalog: HashMap<String, AgentSkillInfo> = list_skill_catalog(services_path)
        .unwrap_or_default()
        .into_iter()
        .map(|entry| {
            (
                entry.id.as_str().to_owned(),
                AgentSkillInfo {
                    id: entry.id,
                    name: entry.name,
                    description: entry.description,
                },
            )
        })
        .collect();
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
                if let Some(key_str) = key.as_str() {
                    let agent_id = AgentId::from(key_str);
                    agents.push(parse_agent_detail(&agent_id, val, &skill_catalog));
                }
            }
        }
    }
    agents.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    Ok(agents)
}

fn parse_agent_detail(
    agent_id: &AgentId,
    val: &serde_yaml::Value,
    skill_catalog: &HashMap<String, AgentSkillInfo>,
) -> AgentDetail {
    AgentDetail {
        id: agent_id.clone(),
        name: val
            .get("card")
            .and_then(|c| c.get("displayName"))
            .or_else(|| val.get("card").and_then(|c| c.get("name")))
            .and_then(|n| n.as_str())
            .unwrap_or(agent_id.as_str())
            .to_owned(),
        description: val
            .get("card")
            .and_then(|c| c.get("description"))
            .and_then(|d| d.as_str())
            .unwrap_or("")
            .to_owned(),
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
            .to_owned(),
        port: val
            .get("port")
            .and_then(serde_yaml::Value::as_u64)
            .map(|p| u16::try_from(p).unwrap_or(u16::MAX)),
        endpoint: val
            .get("endpoint")
            .and_then(|e| e.as_str())
            .map(str::to_owned),
        mcp_servers: val
            .get("mcp_servers")
            .and_then(|v| v.as_sequence())
            .map_or_else(Vec::new, |seq| {
                seq.iter()
                    .filter_map(|v| v.as_str().and_then(|s| McpServerId::try_new(s).ok()))
                    .collect()
            }),
        skills: val
            .get("metadata")
            .and_then(|m| m.get("skills"))
            .and_then(|s| s.as_sequence())
            .map_or_else(Vec::new, |seq| {
                seq.iter()
                    .filter_map(|v| v.as_str())
                    .filter_map(|id| {
                        skill_catalog.get(id).map(|info| AgentSkillInfo {
                            id: info.id.clone(),
                            name: info.name.clone(),
                            description: info.description.clone(),
                        })
                    })
                    .collect()
            }),
    }
}

pub fn find_agent(
    services_path: &Path,
    agent_id: &AgentId,
) -> Result<Option<AgentDetail>, MarketplaceError> {
    let agents = list_agents(services_path)?;
    Ok(agents
        .into_iter()
        .find(|a| a.id.as_str() == agent_id.as_str()))
}
