use std::collections::HashSet;
use std::path::Path;

use super::super::types::{AgentInfo, RequiredSecret, SkillInfo};

pub(crate) fn resolve_all_plugin_skill_ids(
    plugin: &systemprompt::models::PluginConfig,
    skills_path: &Path,
    agents_path: &Path,
) -> Vec<String> {
    let mut skill_ids: Vec<String> =
        if plugin.skills.source == systemprompt::models::ComponentSource::Explicit {
            plugin
                .skills
                .include
                .iter()
                .filter(|id| skills_path.join(id).exists())
                .cloned()
                .collect()
        } else {
            let mut ids = Vec::new();
            if let Ok(entries) = std::fs::read_dir(skills_path) {
                for entry in entries.flatten() {
                    if !entry.path().is_dir() {
                        continue;
                    }
                    let skill_id = entry.file_name().to_string_lossy().to_string();
                    if plugin.skills.exclude.contains(&skill_id) {
                        continue;
                    }
                    ids.push(skill_id);
                }
            }
            ids.sort();
            ids
        };

    let existing: HashSet<String> = skill_ids.iter().cloned().collect();
    for agent_skill in collect_agent_skills(&plugin.agents.include, agents_path) {
        if !existing.contains(&agent_skill) && skills_path.join(&agent_skill).exists() {
            skill_ids.push(agent_skill);
        }
    }

    skill_ids
}

pub(crate) fn resolve_plugin_skills(
    plugin: &systemprompt::models::PluginConfig,
    skills_path: &Path,
    agents_path: &Path,
) -> Vec<SkillInfo> {
    resolve_all_plugin_skill_ids(plugin, skills_path, agents_path)
        .into_iter()
        .map(|skill_id| {
            let skill_dir = skills_path.join(&skill_id);
            let (name, description, required_secrets) = read_skill_config(&skill_dir, &skill_id);
            let kebab_name = skill_id.replace('_', "-");
            let command = format!("/{}:{}", plugin.id, kebab_name);
            SkillInfo {
                id: skill_id.clone(),
                name,
                description,
                command,
                source: "system".to_string(),
                enabled: true,
                required_secrets,
            }
        })
        .collect()
}

fn read_skill_config(skill_dir: &Path, skill_id: &str) -> (String, String, Vec<RequiredSecret>) {
    let config_path = skill_dir.join("config.yaml");
    if !config_path.exists() {
        return (skill_id.to_string(), String::new(), Vec::new());
    }
    let cfg_text = std::fs::read_to_string(&config_path).unwrap_or_else(|e| {
        tracing::warn!(
            path = %config_path.display(),
            error = %e,
            "Failed to read skill config"
        );
        String::new()
    });
    let cfg: serde_yaml::Value = serde_yaml::from_str(&cfg_text).unwrap_or_else(|e| {
        tracing::warn!(
            path = %config_path.display(),
            error = %e,
            "Failed to parse skill config"
        );
        serde_yaml::Value::Null
    });
    let name = cfg
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(skill_id)
        .to_string();
    let desc = cfg
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let required_secrets: Vec<RequiredSecret> = cfg
        .get("required_secrets")
        .and_then(|v| serde_yaml::from_value(v.clone()).ok())
        .unwrap_or_else(Vec::new);
    (name, desc, required_secrets)
}

pub(crate) fn read_skill_required_secrets(
    skills_path: &Path,
    skill_id: &str,
) -> Vec<RequiredSecret> {
    let skill_dir = skills_path.join(skill_id);
    let (_, _, required_secrets) = read_skill_config(&skill_dir, skill_id);
    required_secrets
}

pub(super) fn collect_agent_skills(agent_ids: &[String], agents_path: &Path) -> Vec<String> {
    let mut skills = Vec::new();
    if !agents_path.exists() {
        return skills;
    }
    let Ok(entries) = std::fs::read_dir(agents_path) else {
        return skills;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("yaml") && ext != Some("yml") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(config) = serde_yaml::from_str::<serde_yaml::Value>(&content) else {
            continue;
        };
        let Some(agents) = config.get("agents") else {
            continue;
        };
        for agent_id in agent_ids {
            if let Some(agent_skills) = agents
                .get(agent_id)
                .and_then(|a| a.get("metadata"))
                .and_then(|m| m.get("skills"))
                .and_then(|s| s.as_sequence())
            {
                for skill in agent_skills {
                    if let Some(id) = skill.as_str() {
                        skills.push(id.to_string());
                    }
                }
            }
        }
    }
    skills
}

pub(crate) fn resolve_plugin_agents(
    plugin: &systemprompt::models::PluginConfig,
    agents_path: &Path,
) -> Vec<AgentInfo> {
    let agent_ids: Vec<String> = if plugin.agents.source
        == systemprompt::models::ComponentSource::Explicit
    {
        plugin.agents.include.clone()
    } else {
        let mut ids = Vec::new();
        if let Ok(entries) = std::fs::read_dir(agents_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                let ext = path.extension().and_then(|e| e.to_str());
                if ext != Some("yaml") && ext != Some("yml") {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(config) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                        if let Some(agents) = config.get("agents").and_then(|a| a.as_mapping()) {
                            for (key, _) in agents {
                                if let Some(name) = key.as_str() {
                                    if !plugin.agents.exclude.contains(&name.to_string()) {
                                        ids.push(name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        ids.sort();
        ids
    };

    agent_ids
        .into_iter()
        .map(|agent_id| {
            let description = agent_description(&agent_id, agents_path)
                .unwrap_or_else(|| format!("{agent_id} agent"));
            let enabled = agent_enabled(&agent_id, agents_path);
            AgentInfo {
                id: agent_id.clone(),
                name: agent_id,
                description,
                enabled,
            }
        })
        .collect()
}

fn agent_enabled(agent_id: &str, agents_dir: &Path) -> bool {
    if !agents_dir.exists() {
        return true;
    }
    let Ok(entries) = std::fs::read_dir(agents_dir) else {
        return true;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("yaml") && ext != Some("yml") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(config) = serde_yaml::from_str::<serde_yaml::Value>(&content) else {
            continue;
        };
        if let Some(agent) = config.get("agents").and_then(|a| a.get(agent_id)) {
            return agent
                .get("enabled")
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(true);
        }
    }
    true
}

fn agent_description(agent_id: &str, agents_dir: &Path) -> Option<String> {
    if !agents_dir.exists() {
        return None;
    }
    let Ok(entries) = std::fs::read_dir(agents_dir) else {
        tracing::warn!(path = %agents_dir.display(), "Failed to read agents directory");
        return None;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("yaml") && ext != Some("yml") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(config) = serde_yaml::from_str::<serde_yaml::Value>(&content) else {
            continue;
        };
        if let Some(desc) = config
            .get("agents")
            .and_then(|a| a.get(agent_id))
            .and_then(|a| a.get("card"))
            .and_then(|c| c.get("description"))
            .and_then(|d| d.as_str())
        {
            return Some(desc.to_string());
        }
    }
    None
}
