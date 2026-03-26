use std::path::Path;

use super::super::types::{CreatePluginRequest, PluginDetail};
use super::export::{McpConfigFile, PluginManifest};
use crate::error::MarketplaceError;

pub fn import_plugin_bundle(
    services_path: &Path,
    bundle: &super::export::PluginBundle,
) -> Result<PluginDetail, MarketplaceError> {
    let plugin_id = &bundle.id;
    let plugin_dir = services_path.join("plugins").join(plugin_id);
    if plugin_dir.exists() {
        return Err(MarketplaceError::Internal(format!(
            "Plugin '{plugin_id}' already exists"
        )));
    }

    let manifest = bundle
        .files
        .iter()
        .find(|f| f.path == ".claude-plugin/plugin.json")
        .map(|f| serde_json::from_str::<PluginManifest>(&f.content))
        .transpose()
        .map_err(|e| {
            MarketplaceError::Internal(format!("Failed to parse plugin.json manifest: {e}"))
        })?;

    let metadata = extract_import_metadata(manifest.as_ref(), bundle);
    let skill_ids = import_skill_files(&bundle.files, &services_path.join("skills"))?;
    let agent_ids = extract_agent_ids(&bundle.files);
    let mcp_servers = parse_import_mcp_servers(&bundle.files);

    std::fs::create_dir_all(plugin_dir.join("scripts"))?;
    write_import_scripts(&bundle.files, &plugin_dir)?;

    let req = CreatePluginRequest {
        id: plugin_id.clone(),
        name: metadata.name,
        description: metadata.description,
        version: metadata.version,
        enabled: true,
        category: String::new(),
        keywords: metadata.keywords,
        author_name: metadata.author_name,
        roles: Vec::new(),
        skills: skill_ids,
        agents: agent_ids,
        mcp_servers,
        hooks: vec![],
    };

    super::plugin_crud_ops::create_plugin(services_path, &req)
}

struct ImportMetadata {
    name: String,
    description: String,
    version: String,
    author_name: String,
    keywords: Vec<String>,
}

fn extract_import_metadata(
    manifest: Option<&PluginManifest>,
    bundle: &super::export::PluginBundle,
) -> ImportMetadata {
    let name = manifest.map_or(&bundle.name, |m| &m.name).to_string();
    let description = manifest
        .map_or(&bundle.description, |m| &m.description)
        .to_string();
    let version = manifest.map_or(&bundle.version, |m| &m.version).to_string();
    let author_name = manifest
        .and_then(|m| m.author.as_ref())
        .map_or("", |a| a.name.as_str())
        .to_string();
    let keywords = manifest.map(|m| m.keywords.clone()).unwrap_or_default();

    ImportMetadata {
        name,
        description,
        version,
        author_name,
        keywords,
    }
}

fn import_skill_files(
    files: &[super::export::PluginFile],
    skills_path: &Path,
) -> Result<Vec<String>, MarketplaceError> {
    let mut skill_ids = Vec::new();
    let mut processed_skills = std::collections::HashSet::new();

    for file in files.iter().filter(|f| f.path.starts_with("skills/")) {
        let Some(rest) = file.path.strip_prefix("skills/") else {
            continue;
        };
        let parts: Vec<&str> = rest.splitn(2, '/').collect();
        if parts.len() < 2 {
            continue;
        }
        let kebab_name = parts[0];
        let skill_id = kebab_name.replace('-', "_");
        processed_skills.insert(skill_id.clone());

        let skill_dir = skills_path.join(&skill_id);

        if skill_dir.exists() {
            if !skill_ids.contains(&skill_id) {
                skill_ids.push(skill_id);
            }
            continue;
        }

        if parts[1] == "SKILL.md" {
            let (fm_name, fm_description, body) = parse_skill_frontmatter(&file.content);
            std::fs::create_dir_all(&skill_dir).map_err(|e| {
                MarketplaceError::Internal(format!(
                    "Failed to create skill dir: {}: {e}",
                    skill_dir.display()
                ))
            })?;
            let config_yaml = format!(
                "name: \"{}\"\ndescription: \"{}\"\nenabled: true\n",
                fm_name.unwrap_or_else(|| kebab_name.to_string()),
                fm_description
                    .unwrap_or_else(String::new)
                    .replace('"', "\\\""),
            );
            std::fs::write(skill_dir.join("config.yaml"), &config_yaml)?;
            std::fs::write(skill_dir.join("index.md"), &body)?;
            if !skill_ids.contains(&skill_id) {
                skill_ids.push(skill_id);
            }
        } else {
            let aux_path = skill_dir.join(parts[1]);
            if let Some(parent) = aux_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&aux_path, &file.content)?;
        }
    }
    Ok(skill_ids)
}

fn extract_agent_ids(files: &[super::export::PluginFile]) -> Vec<String> {
    files
        .iter()
        .filter(|f| f.path.starts_with("agents/"))
        .filter_map(|f| {
            f.path
                .strip_prefix("agents/")
                .and_then(|name| name.strip_suffix(".md"))
                .map(String::from)
        })
        .collect()
}

fn parse_import_mcp_servers(files: &[super::export::PluginFile]) -> Vec<String> {
    let Some(mcp_file) = files.iter().find(|f| f.path == ".mcp.json") else {
        return Vec::new();
    };
    let Ok(mcp_config) = serde_json::from_str::<McpConfigFile>(&mcp_file.content) else {
        return Vec::new();
    };
    mcp_config.mcp_servers.into_keys().collect()
}

fn write_import_scripts(
    files: &[super::export::PluginFile],
    plugin_dir: &Path,
) -> Result<(), MarketplaceError> {
    for file in files.iter().filter(|f| f.path.starts_with("scripts/")) {
        let Some(filename) = file.path.strip_prefix("scripts/") else {
            continue;
        };
        let script_path = plugin_dir.join("scripts").join(filename);
        if let Some(parent) = script_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&script_path, &file.content)?;
        #[cfg(unix)]
        if file.executable {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o755);
            if let Err(e) = std::fs::set_permissions(&script_path, perms) {
                tracing::warn!(error = %e, path = %script_path.display(), "Failed to set file permissions");
            }
        }
    }
    Ok(())
}

fn parse_skill_frontmatter(content: &str) -> (Option<String>, Option<String>, String) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (None, None, content.to_string());
    }
    let parts: Vec<&str> = trimmed.splitn(3, "---").collect();
    if parts.len() < 3 {
        return (None, None, content.to_string());
    }
    let frontmatter = parts[1].trim();
    let body = parts[2].trim().to_string();

    let mut name = None;
    let mut description = None;
    for line in frontmatter.lines() {
        if let Some(val) = line.strip_prefix("name:") {
            name = Some(val.trim().trim_matches('"').to_string());
        } else if let Some(val) = line.strip_prefix("description:") {
            description = Some(val.trim().trim_matches('"').to_string());
        }
    }
    (name, description, body)
}
