use std::fmt::Write;
use std::path::Path;

use super::super::types::{CreatePluginRequest, PluginDetail, UpdatePluginRequest};
use systemprompt_web_shared::error::MarketplaceError;

pub fn find_plugin_detail(
    services_path: &Path,
    plugin_id: &str,
) -> Result<Option<PluginDetail>, MarketplaceError> {
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
) -> Result<PluginDetail, MarketplaceError> {
    let plugin_dir = services_path.join("plugins").join(&req.id);
    if plugin_dir.exists() {
        return Err(MarketplaceError::Internal(format!(
            "Plugin '{}' already exists",
            req.id
        )));
    }
    std::fs::create_dir_all(&plugin_dir)?;

    let yaml_content = build_plugin_yaml(req);
    std::fs::write(plugin_dir.join("config.yaml"), &yaml_content)
        .map_err(|e| MarketplaceError::Internal(format!("Failed to write plugin config: {e}")))?;
    std::fs::create_dir_all(plugin_dir.join("scripts"))?;

    let category = if req.category.is_empty() {
        "general".to_string()
    } else {
        req.category.clone()
    };
    Ok(PluginDetail {
        id: req.id.clone(),
        name: req.name.clone(),
        description: req.description.clone(),
        version: req.version.clone(),
        enabled: req.enabled,
        category,
        keywords: req.keywords.clone(),
        author_name: req.author_name.clone(),
        roles: req.roles.clone(),
        skills: req.skills.clone(),
        agents: req.agents.clone(),
        mcp_servers: req.mcp_servers.clone(),
    })
}

fn build_plugin_yaml(req: &CreatePluginRequest) -> String {
    fn yaml_list(items: &[String], indent: &str) -> String {
        if items.is_empty() {
            return format!("{indent}[]\n");
        }
        let mut out = String::new();
        for item in items {
            let _ = writeln!(out, "{indent}- {item}");
        }
        out
    }

    format!(
        "plugin:\n  id: {}\n  name: \"{}\"\n  description: \"{}\"\n  version: \"{}\"\n  enabled: {}\n\n  skills:\n    source: explicit\n    include:\n{}\n  agents:\n    source: explicit\n    include:\n{}\n  mcp_servers:\n{}\n{}\n  roles:\n{}\n  scripts: []\n\n  keywords:\n{}  category: {}\n\n  author:\n    name: \"{}\"\n",
        req.id,
        req.name,
        req.description.replace('"', "\\\""),
        req.version,
        req.enabled,
        yaml_list(&req.skills, "      "),
        yaml_list(&req.agents, "      "),
        yaml_list(&req.mcp_servers, "    "),
        "  hooks: {}\n",
        yaml_list(&req.roles, "    "),
        yaml_list(&req.keywords, "    "),
        if req.category.is_empty() { "general" } else { &req.category },
        req.author_name,
    )
}

pub fn update_plugin(
    services_path: &Path,
    plugin_id: &str,
    req: &UpdatePluginRequest,
) -> Result<Option<PluginDetail>, MarketplaceError> {
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
        apply_plugin_updates(plugin, req);
    }
    std::fs::write(&config_path, serde_yaml::to_string(&doc)?).map_err(|e| {
        MarketplaceError::Internal(format!("Failed to write: {}: {e}", config_path.display()))
    })?;
    find_plugin_detail(services_path, plugin_id)
}

fn apply_plugin_updates(plugin: &mut serde_yaml::Value, req: &UpdatePluginRequest) {
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

pub fn delete_plugin(services_path: &Path, plugin_id: &str) -> Result<bool, MarketplaceError> {
    let plugin_dir = services_path.join("plugins").join(plugin_id);
    if !plugin_dir.exists() {
        return Ok(false);
    }
    std::fs::remove_dir_all(&plugin_dir)?;
    Ok(true)
}
