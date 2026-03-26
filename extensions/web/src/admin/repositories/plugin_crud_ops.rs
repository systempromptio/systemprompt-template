use std::fmt::Write;
use std::path::Path;

use super::super::types::{
    CreateHookRequest, CreatePluginRequest, PluginDetail, UpdatePluginRequest,
};

pub fn get_plugin_detail(
    services_path: &Path,
    plugin_id: &str,
) -> Result<Option<PluginDetail>, anyhow::Error> {
    use super::super::types::PlatformPluginConfigFile;
    use super::plugin_resolvers::resolve_all_plugin_skill_ids;

    let config_path = services_path
        .join("plugins")
        .join(plugin_id)
        .join("config.yaml");
    if !config_path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&config_path)?;
    let plugin_file: PlatformPluginConfigFile = serde_yaml::from_str(&content)?;
    let p = plugin_file.plugin;
    let skills_path = services_path.join("skills");
    let agents_path = services_path.join("agents");
    let skills = resolve_all_plugin_skill_ids(&p.base, &skills_path, &agents_path);
    Ok(Some(PluginDetail {
        id: p.base.id,
        name: p.base.name,
        description: p.base.description,
        version: p.base.version,
        enabled: p.base.enabled,
        category: p.base.category,
        keywords: p.base.keywords,
        author_name: p.base.author.name,
        roles: p.roles,
        skills,
        agents: p.base.agents.include,
        mcp_servers: p.base.mcp_servers,
    }))
}

pub fn create_plugin(
    services_path: &Path,
    req: &CreatePluginRequest,
) -> Result<PluginDetail, anyhow::Error> {
    use anyhow::Context;
    let plugin_dir = services_path.join("plugins").join(&req.id);
    if plugin_dir.exists() {
        anyhow::bail!("Plugin '{}' already exists", req.id);
    }
    std::fs::create_dir_all(&plugin_dir)?;
    let hooks_yaml = build_hooks_yaml(&req.hooks);
    let skills_yaml: String = req.skills.iter().fold(String::new(), |mut s, skill| {
        writeln!(s, "      - {skill}").expect("write to String cannot fail");
        s
    });
    let agents_yaml: String = req.agents.iter().fold(String::new(), |mut s, a| {
        writeln!(s, "      - {a}").expect("write to String cannot fail");
        s
    });
    let mcp_yaml: String = req.mcp_servers.iter().fold(String::new(), |mut s, m| {
        writeln!(s, "    - {m}").expect("write to String cannot fail");
        s
    });
    let roles_yaml: String = req.roles.iter().fold(String::new(), |mut s, r| {
        writeln!(s, "    - {r}").expect("write to String cannot fail");
        s
    });
    let keywords_yaml: String = req.keywords.iter().fold(String::new(), |mut s, k| {
        writeln!(s, "    - {k}").expect("write to String cannot fail");
        s
    });
    let yaml_content = format!(
        "plugin:\n  id: {}\n  name: \"{}\"\n  description: \"{}\"\n  version: \"{}\"\n  enabled: {}\n\n  skills:\n    source: explicit\n    include:\n{}\n  agents:\n    source: explicit\n    include:\n{}\n  mcp_servers:\n{}\n{}\n  roles:\n{}\n  scripts: []\n\n  keywords:\n{}  category: {}\n\n  author:\n    name: \"{}\"\n",
        req.id,
        req.name,
        req.description.replace('"', "\\\""),
        req.version,
        req.enabled,
        if skills_yaml.is_empty() {
            "      []\n".to_string()
        } else {
            skills_yaml
        },
        if agents_yaml.is_empty() {
            "      []\n".to_string()
        } else {
            agents_yaml
        },
        if mcp_yaml.is_empty() {
            "    []\n".to_string()
        } else {
            mcp_yaml
        },
        hooks_yaml,
        if roles_yaml.is_empty() {
            "    []\n".to_string()
        } else {
            roles_yaml
        },
        if keywords_yaml.is_empty() {
            "    []\n".to_string()
        } else {
            keywords_yaml
        },
        if req.category.is_empty() {
            "general"
        } else {
            &req.category
        },
        req.author_name,
    );
    std::fs::write(plugin_dir.join("config.yaml"), &yaml_content)
        .with_context(|| "Failed to write plugin config")?;
    std::fs::create_dir_all(plugin_dir.join("scripts"))?;
    Ok(PluginDetail {
        id: req.id.clone(),
        name: req.name.clone(),
        description: req.description.clone(),
        version: req.version.clone(),
        enabled: req.enabled,
        category: if req.category.is_empty() {
            "general".to_string()
        } else {
            req.category.clone()
        },
        keywords: req.keywords.clone(),
        author_name: req.author_name.clone(),
        roles: req.roles.clone(),
        skills: req.skills.clone(),
        agents: req.agents.clone(),
        mcp_servers: req.mcp_servers.clone(),
    })
}

pub fn build_hooks_yaml(hooks: &[CreateHookRequest]) -> String {
    if hooks.is_empty() {
        return "  hooks: {}\n".to_string();
    }
    let mut events: std::collections::HashMap<&str, Vec<&CreateHookRequest>> =
        std::collections::HashMap::new();
    for hook in hooks {
        events.entry(&hook.event).or_default().push(hook);
    }
    let mut yaml = "  hooks:\n".to_string();
    for (event, entries) in &events {
        writeln!(yaml, "    {event}:").expect("write to String cannot fail");
        for entry in entries {
            write!(
                yaml,
                "      - matcher: \"{}\"\n        hooks:\n          - type: command\n            command: \"{}\"\n            async: {}\n",
                entry.matcher, entry.command, entry.is_async
            )
            .expect("write to String cannot fail");
        }
    }
    yaml
}

pub fn update_plugin(
    services_path: &Path,
    plugin_id: &str,
    req: &UpdatePluginRequest,
) -> Result<Option<PluginDetail>, anyhow::Error> {
    use anyhow::Context;
    let config_path = services_path
        .join("plugins")
        .join(plugin_id)
        .join("config.yaml");
    if !config_path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&config_path)?;
    let mut doc: serde_yaml::Value = serde_yaml::from_str(&content)?;
    if let Some(plugin) = doc.get_mut("plugin") {
        if let Some(ref v) = req.name {
            plugin["name"] = serde_yaml::Value::String(v.clone());
        }
        if let Some(ref v) = req.description {
            plugin["description"] = serde_yaml::Value::String(v.clone());
        }
        if let Some(ref v) = req.version {
            plugin["version"] = serde_yaml::Value::String(v.clone());
        }
        if let Some(v) = req.enabled {
            plugin["enabled"] = serde_yaml::Value::Bool(v);
        }
        if let Some(ref v) = req.category {
            plugin["category"] = serde_yaml::Value::String(v.clone());
        }
        if let Some(ref v) = req.keywords {
            plugin["keywords"] = serde_yaml::Value::Sequence(
                v.iter()
                    .map(|k| serde_yaml::Value::String(k.clone()))
                    .collect(),
            );
        }
        if let Some(ref v) = req.author_name {
            if plugin.get("author").is_none() {
                plugin["author"] = serde_yaml::Value::Mapping(serde_yaml::Mapping::new());
            }
            if let Some(a) = plugin.get_mut("author") {
                a["name"] = serde_yaml::Value::String(v.clone());
            }
        }
        if let Some(ref v) = req.roles {
            plugin["roles"] = serde_yaml::Value::Sequence(
                v.iter()
                    .map(|r| serde_yaml::Value::String(r.clone()))
                    .collect(),
            );
        }
        if let Some(ref v) = req.skills {
            if let Some(s) = plugin.get_mut("skills") {
                s["source"] = serde_yaml::Value::String("explicit".into());
                s["include"] = serde_yaml::Value::Sequence(
                    v.iter()
                        .map(|s| serde_yaml::Value::String(s.clone()))
                        .collect(),
                );
            }
        }
        if let Some(ref v) = req.agents {
            if let Some(a) = plugin.get_mut("agents") {
                a["source"] = serde_yaml::Value::String("explicit".into());
                a["include"] = serde_yaml::Value::Sequence(
                    v.iter()
                        .map(|a| serde_yaml::Value::String(a.clone()))
                        .collect(),
                );
            }
        }
        if let Some(ref v) = req.mcp_servers {
            plugin["mcp_servers"] = serde_yaml::Value::Sequence(
                v.iter()
                    .map(|m| serde_yaml::Value::String(m.clone()))
                    .collect(),
            );
        }
    }
    std::fs::write(&config_path, serde_yaml::to_string(&doc)?)
        .with_context(|| format!("Failed to write: {}", config_path.display()))?;
    get_plugin_detail(services_path, plugin_id)
}

pub fn delete_plugin(services_path: &Path, plugin_id: &str) -> Result<bool, anyhow::Error> {
    let plugin_dir = services_path.join("plugins").join(plugin_id);
    if !plugin_dir.exists() {
        return Ok(false);
    }
    std::fs::remove_dir_all(&plugin_dir)?;
    Ok(true)
}
