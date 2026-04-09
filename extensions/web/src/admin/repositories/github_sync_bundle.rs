use std::path::Path;

use anyhow::Result;

use super::export::{PluginBundle, PluginBundleCounts, PluginFile};

pub(crate) fn build_bundle_from_directory(plugin_dir: &Path) -> Result<PluginBundle> {
    let plugin_json_path = plugin_dir.join(".claude-plugin/plugin.json");
    let manifest_content = std::fs::read_to_string(&plugin_json_path).map_err(|e| {
        anyhow::anyhow!(
            "Failed to read plugin.json at {}: {e}",
            plugin_json_path.display()
        )
    })?;
    let manifest: serde_json::Value = serde_json::from_str(&manifest_content)?;

    let plugin_id = manifest
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("plugin.json missing 'name'"))?
        .to_string();

    let description = manifest
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let version = manifest
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("1.0.0")
        .to_string();

    let name = plugin_id.clone();

    let mut files = Vec::new();

    files.push(PluginFile {
        path: ".claude-plugin/plugin.json".to_string(),
        content: manifest_content,
        executable: false,
    });

    let hooks_path = plugin_dir.join("hooks/hooks.json");
    if hooks_path.exists() {
        let content = std::fs::read_to_string(&hooks_path)?;
        files.push(PluginFile {
            path: "hooks/hooks.json".to_string(),
            content,
            executable: false,
        });
    }

    let skills_dir = plugin_dir.join("skills");
    if skills_dir.exists() {
        collect_directory_files(&skills_dir, "skills", &mut files)?;
    }

    let agents_dir = plugin_dir.join("agents");
    if agents_dir.exists() {
        collect_directory_files(&agents_dir, "agents", &mut files)?;
    }

    let mcp_path = plugin_dir.join(".mcp.json");
    if mcp_path.exists() {
        let content = std::fs::read_to_string(&mcp_path)?;
        files.push(PluginFile {
            path: ".mcp.json".to_string(),
            content,
            executable: false,
        });
    }

    let mut skills_count = 0;
    let mut agents_count = 0;
    let mut _hooks_count = 0;
    for f in &files {
        if f.path.starts_with("skills/") && f.path.ends_with("SKILL.md") {
            skills_count += 1;
        } else if f.path.starts_with("agents/")
            && std::path::Path::new(&f.path)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            agents_count += 1;
        } else if f.path == "hooks/hooks.json" {
            _hooks_count += 1;
        }
    }
    let total_files = files.len();

    Ok(PluginBundle {
        id: plugin_id,
        name,
        description,
        version,
        files,
        counts: PluginBundleCounts {
            skills: skills_count,
            agents: agents_count,
            mcp_servers: 0,
            scripts: 0,
            total_files,
        },
    })
}

pub(crate) fn collect_directory_files(
    dir: &Path,
    prefix: &str,
    files: &mut Vec<PluginFile>,
) -> Result<()> {
    for entry in walkdir::WalkDir::new(dir)
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let rel_path = entry
            .path()
            .strip_prefix(dir)
            .map_err(|e| anyhow::anyhow!("Failed to strip prefix: {e}"))?;
        let path = format!("{prefix}/{}", rel_path.display());
        let content = std::fs::read_to_string(entry.path())
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {e}", entry.path().display()))?;
        files.push(PluginFile {
            path,
            content,
            executable: false,
        });
    }
    Ok(())
}

pub(crate) fn import_or_update_plugin(services_path: &Path, bundle: &PluginBundle) -> Result<()> {
    let plugin_dir = services_path.join("plugins").join(&bundle.id);

    if plugin_dir.exists() {
        std::fs::remove_dir_all(&plugin_dir)?;
    }

    super::import_plugin_bundle(services_path, bundle)?;
    Ok(())
}
