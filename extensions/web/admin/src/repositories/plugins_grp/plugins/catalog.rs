//! Filesystem catalog walks over `services/` — one read-only row per skill,
//! agent, or plugin, plus the raw skill-id listings. Never writes to disk.

use std::path::Path;

use systemprompt::identifiers::{AgentId, McpServerId, SkillId};

use crate::repositories::plugin_resolvers::resolve_all_plugin_skill_ids;
use crate::repositories::plugins_grp::plugin_loader;
use crate::types::{AgentCatalogEntry, SkillCatalogEntry};
use systemprompt_web_shared::error::MarketplaceError;

/// Walk `services/skills/<id>/config.yaml` and return one catalog row per
/// skill. Used by the unified `/admin/catalog` page; never writes to disk.
pub fn list_skill_catalog(
    services_path: &Path,
) -> Result<Vec<SkillCatalogEntry>, MarketplaceError> {
    let skills_path = services_path.join("skills");
    let mut out: Vec<SkillCatalogEntry> = Vec::new();
    if !skills_path.exists() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(&skills_path)? {
        let entry = entry?;
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let cfg = dir.join("config.yaml");
        if !cfg.exists() {
            continue;
        }
        let raw = match std::fs::read_to_string(&cfg) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!(path = %cfg.display(), error = %e, "skipped unreadable skill config");
                continue;
            }
        };
        let val: serde_yaml::Value = match serde_yaml::from_str(&raw) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(path = %cfg.display(), error = %e, "skipped invalid skill yaml");
                continue;
            }
        };
        let id_str = val
            .get("id")
            .and_then(|v| v.as_str())
            .or_else(|| dir.file_name().and_then(|n| n.to_str()))
            .unwrap_or("")
            .to_string();
        if id_str.is_empty() {
            continue;
        }
        let id = SkillId::new(&id_str);
        let name = val
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(&id_str)
            .to_string();
        let description = val
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let enabled = val
            .get("enabled")
            .and_then(serde_yaml::Value::as_bool)
            .unwrap_or(true);
        // Why: `strip_prefix` returns Err when the path isn't under services_path
        // (e.g. an absolute legacy entry); fall back to the full display path.
        let source_path = cfg
            .strip_prefix(services_path)
            .ok()
            .and_then(|p| p.to_str())
            .map_or_else(|| cfg.display().to_string(), |s| format!("services/{s}"));
        out.push(SkillCatalogEntry {
            id,
            name,
            description,
            enabled,
            source_path,
        });
    }
    out.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    Ok(out)
}

/// Walk `services/agents/<id>.yaml` and return one catalog row per agent.
pub fn list_agent_catalog(
    services_path: &Path,
) -> Result<Vec<AgentCatalogEntry>, MarketplaceError> {
    let agents_path = services_path.join("agents");
    let mut out: Vec<AgentCatalogEntry> = Vec::new();
    if !agents_path.exists() {
        return Ok(out);
    }
    for entry in std::fs::read_dir(&agents_path)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        let Ok(raw) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(val): Result<serde_yaml::Value, _> = serde_yaml::from_str(&raw) else {
            continue;
        };
        // Why: see `list_skill_catalog` — same fallback semantics on prefix miss.
        let source_path = path
            .strip_prefix(services_path)
            .ok()
            .and_then(|p| p.to_str())
            .map_or_else(|| path.display().to_string(), |s| format!("services/{s}"));
        let Some(map) = val.get("agents").and_then(|m| m.as_mapping()) else {
            continue;
        };
        for (key, av) in map {
            let Some(id_str) = key.as_str() else { continue };
            let id = AgentId::new(id_str);
            let name = av
                .get("name")
                .and_then(|v| v.as_str())
                .or_else(|| {
                    av.get("card")
                        .and_then(|c| c.get("displayName"))
                        .and_then(|v| v.as_str())
                })
                .unwrap_or(id_str)
                .to_string();
            let description = av
                .get("card")
                .and_then(|c| c.get("description"))
                .and_then(|v| v.as_str())
                .or_else(|| av.get("description").and_then(|v| v.as_str()))
                .unwrap_or("")
                .to_string();
            let enabled = av
                .get("enabled")
                .and_then(serde_yaml::Value::as_bool)
                .unwrap_or(true);
            out.push(AgentCatalogEntry {
                id,
                name,
                description,
                enabled,
                source_path: source_path.clone(),
            });
        }
    }
    out.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    Ok(out)
}

/// Catalog rows for plugins, including the YAML path each was loaded from.
pub fn list_plugin_catalog(
    services_path: &Path,
) -> Result<Vec<crate::types::PluginDetail>, MarketplaceError> {
    use crate::types::PluginDetail;
    let skills_path = services_path.join("skills");
    let agents_path = services_path.join("agents");
    let mut out: Vec<PluginDetail> = Vec::new();
    for (_id, plugin, source_path) in plugin_loader::load_all_plugins_with_paths()? {
        let skills: Vec<SkillId> =
            resolve_all_plugin_skill_ids(&plugin.base, &skills_path, &agents_path)
                .into_iter()
                .map(SkillId::from)
                .collect();
        out.push(PluginDetail {
            id: plugin.base.id.to_string(),
            name: plugin.base.name,
            description: plugin.base.description,
            version: plugin.base.version,
            enabled: plugin.base.enabled,
            category: plugin.base.category,
            keywords: plugin.base.keywords,
            author_name: plugin.base.author.name,
            roles: plugin.roles,
            skills,
            agents: plugin
                .base
                .agents
                .include
                .into_iter()
                .map(AgentId::from)
                .collect(),
            mcp_servers: plugin
                .base
                .mcp_servers
                .include
                .into_iter()
                .filter_map(|s| McpServerId::try_new(s).ok())
                .collect(),
            source_path,
        });
    }
    Ok(out)
}

pub fn list_all_skill_ids(services_path: &Path) -> Result<Vec<String>, MarketplaceError> {
    let skills_path = services_path.join("skills");
    let mut ids = Vec::new();
    if !skills_path.exists() {
        return Ok(ids);
    }
    for entry in std::fs::read_dir(&skills_path)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            ids.push(stem.to_string());
        }
    }
    ids.sort();
    ids.dedup();
    Ok(ids)
}

pub fn list_plugin_skill_ids(
    services_path: &Path,
    plugin_id: &str,
) -> Result<Vec<String>, MarketplaceError> {
    let plugin = plugin_loader::find_plugin(plugin_id)?
        .ok_or_else(|| MarketplaceError::NotFound(format!("Plugin not found: {plugin_id}")))?;
    let skills_path = services_path.join("skills");
    let agents_path = services_path.join("agents");
    Ok(resolve_all_plugin_skill_ids(
        &plugin,
        &skills_path,
        &agents_path,
    ))
}

pub fn update_plugin_skills(
    _services_path: &Path,
    _plugin_id: &str,
    _skill_ids: &[SkillId],
) -> Result<(), MarketplaceError> {
    Err(MarketplaceError::Internal(
        "update_plugin_skills is disabled; edit services/plugins/*.yaml directly".to_string(),
    ))
}
