use std::path::Path;

use super::super::types::{CreateHookRequest, CreatePluginRequest, PluginDetail};

pub fn import_plugin_bundle(
    services_path: &Path,
    bundle: &super::export::PluginBundle,
) -> Result<PluginDetail, anyhow::Error> {
    use anyhow::Context;

    let plugin_id = &bundle.id;
    let plugin_dir = services_path.join("plugins").join(plugin_id);
    if plugin_dir.exists() {
        anyhow::bail!("Plugin '{plugin_id}' already exists");
    }

    let manifest = bundle
        .files
        .iter()
        .find(|f| f.path == ".claude-plugin/plugin.json")
        .map(|f| serde_json::from_str::<serde_json::Value>(&f.content))
        .transpose()
        .context("Failed to parse plugin.json manifest")?;

    let metadata = extract_import_metadata(manifest.as_ref(), bundle);
    let skill_ids = import_skill_files(&bundle.files, &services_path.join("skills"))?;
    let agent_ids = extract_agent_ids(&bundle.files);
    let hooks = parse_import_hooks(&bundle.files, plugin_id);
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
        hooks,
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
    manifest: Option<&serde_json::Value>,
    bundle: &super::export::PluginBundle,
) -> ImportMetadata {
    let name = manifest
        .and_then(|m| m.get("name").and_then(|v| v.as_str()))
        .unwrap_or(&bundle.name)
        .to_string();
    let description = manifest
        .and_then(|m| m.get("description").and_then(|v| v.as_str()))
        .unwrap_or(&bundle.description)
        .to_string();
    let version = manifest
        .and_then(|m| m.get("version").and_then(|v| v.as_str()))
        .unwrap_or(&bundle.version)
        .to_string();
    let author_name = manifest
        .and_then(|m| {
            m.get("author")
                .and_then(|a| a.get("name"))
                .and_then(|v| v.as_str())
        })
        .unwrap_or("")
        .to_string();
    let keywords: Vec<String> = manifest
        .and_then(|m| m.get("keywords"))
        .and_then(|v| v.as_array())
        .map_or_else(Vec::new, |arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });

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
) -> Result<Vec<String>, anyhow::Error> {
    use anyhow::Context;

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
            std::fs::create_dir_all(&skill_dir)
                .with_context(|| format!("Failed to create skill dir: {}", skill_dir.display()))?;
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

fn parse_import_hooks(
    files: &[super::export::PluginFile],
    plugin_id: &str,
) -> Vec<CreateHookRequest> {
    let Some(hooks_file) = files.iter().find(|f| f.path == "hooks/hooks.json") else {
        return Vec::new();
    };
    let Ok(hooks_json) = serde_json::from_str::<serde_json::Value>(&hooks_file.content) else {
        return Vec::new();
    };
    let Some(obj) = hooks_json.as_object() else {
        return Vec::new();
    };

    let mut hooks = Vec::new();
    for (event, matchers) in obj {
        let Some(arr) = matchers.as_array() else {
            continue;
        };
        for entry in arr {
            let matcher = entry
                .get("matcher")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("*")
                .to_string();
            let Some(hook_list) = entry.get("hooks").and_then(serde_json::Value::as_array) else {
                continue;
            };
            for hook in hook_list {
                let mut command = hook
                    .get("command")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("")
                    .to_string();
                if let Some(stripped) = command.strip_prefix("${CLAUDE_PLUGIN_ROOT}/scripts/") {
                    command = format!("services/plugins/{plugin_id}/scripts/{stripped}");
                }
                let is_async = hook
                    .get("async")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);
                if !command.is_empty() {
                    let name = hook
                        .get("name")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    let description = hook
                        .get("description")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("")
                        .to_string();
                    hooks.push(CreateHookRequest {
                        plugin_id: plugin_id.to_string(),
                        event: event.clone(),
                        matcher: matcher.clone(),
                        command,
                        is_async,
                        name,
                        description,
                    });
                }
            }
        }
    }
    hooks
}

fn parse_import_mcp_servers(files: &[super::export::PluginFile]) -> Vec<String> {
    let Some(mcp_file) = files.iter().find(|f| f.path == ".mcp.json") else {
        return Vec::new();
    };
    let Ok(mcp_json) = serde_json::from_str::<serde_json::Value>(&mcp_file.content) else {
        return Vec::new();
    };
    mcp_json
        .get("mcpServers")
        .and_then(serde_json::Value::as_object)
        .map_or_else(Vec::new, |servers| servers.keys().cloned().collect())
}

fn write_import_scripts(
    files: &[super::export::PluginFile],
    plugin_dir: &Path,
) -> Result<(), anyhow::Error> {
    for file in files.iter().filter(|f| f.path.starts_with("scripts/")) {
        let filename = file
            .path
            .strip_prefix("scripts/")
            .expect("path starts with scripts/ due to filter");
        let script_path = plugin_dir.join("scripts").join(filename);
        if let Some(parent) = script_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&script_path, &file.content)?;
        #[cfg(unix)]
        if file.executable {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o755);
            let _ = std::fs::set_permissions(&script_path, perms);
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
