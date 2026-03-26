use super::export::{ExportTotals, PluginBundle, PluginBundleCounts, PluginFile};

pub(super) fn validate_bundle(files: &[PluginFile], expected_skills: usize) {
    let mut warnings = Vec::new();

    if !files.iter().any(|f| f.path == ".claude-plugin/plugin.json") {
        warnings.push("Missing .claude-plugin/plugin.json".to_string());
    }

    let mut skill_dirs = std::collections::HashSet::new();
    let mut skills_with_md = std::collections::HashSet::new();
    for f in files.iter().filter(|f| f.path.starts_with("skills/")) {
        if let Some(dir) = f
            .path
            .strip_prefix("skills/")
            .and_then(|p| p.split('/').next())
        {
            skill_dirs.insert(dir.to_string());
            if f.path.ends_with("/SKILL.md") {
                skills_with_md.insert(dir.to_string());
            }
        }
    }

    if skill_dirs.len() != expected_skills {
        warnings.push(format!(
            "Expected {expected_skills} skills, found {}",
            skill_dirs.len()
        ));
    }

    for dir in &skill_dirs {
        if !skills_with_md.contains(dir) {
            warnings.push(format!("Skill '{dir}' missing SKILL.md"));
        }
    }

    for f in files.iter().filter(|f| f.path == "hooks/hooks.json") {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&f.content) {
            if val.get("hooks").is_none() {
                warnings.push("hooks.json missing required {\"hooks\": ...} wrapper".to_string());
            }
        }
    }

    for w in &warnings {
        tracing::warn!(warning = %w, "Export validation");
    }
}

pub(super) fn build_manifest(
    plugin: &systemprompt::models::PluginConfig,
    version_override: Option<&str>,
) -> serde_json::Value {
    let mut manifest = serde_json::Map::new();
    manifest.insert(
        "name".to_string(),
        serde_json::Value::String(plugin.id.clone()),
    );
    manifest.insert(
        "description".to_string(),
        serde_json::Value::String(plugin.description.clone()),
    );
    manifest.insert(
        "version".to_string(),
        serde_json::Value::String(version_override.unwrap_or(&plugin.version).to_string()),
    );
    let mut author_obj = serde_json::Map::new();
    author_obj.insert(
        "name".to_string(),
        serde_json::Value::String(plugin.author.name.clone()),
    );
    author_obj.insert(
        "email".to_string(),
        serde_json::Value::String(plugin.author.email.clone()),
    );
    manifest.insert("author".to_string(), serde_json::Value::Object(author_obj));
    serde_json::Value::Object(manifest)
}

pub(super) fn compute_content_version(base_version: &str, files: &[PluginFile]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    for file in files {
        file.path.hash(&mut hasher);
        file.content.hash(&mut hasher);
    }
    let hash = hasher.finish();

    format!("{base_version}+{hash:08x}")
}

pub(super) fn compute_bundle_counts(files: &[PluginFile]) -> PluginBundleCounts {
    let skills = {
        let mut dirs = std::collections::HashSet::new();
        for f in files.iter().filter(|f| f.path.starts_with("skills/")) {
            if let Some(dir) = f
                .path
                .strip_prefix("skills/")
                .and_then(|p| p.split('/').next())
            {
                dirs.insert(dir);
            }
        }
        dirs.len()
    };
    let agents = files
        .iter()
        .filter(|f| f.path.starts_with("agents/"))
        .count();
    let hooks = files
        .iter()
        .filter(|f| f.path == "hooks/hooks.json")
        .count();
    let mcp_servers = usize::from(files.iter().any(|f| f.path == ".mcp.json"));
    let scripts = files
        .iter()
        .filter(|f| f.path.starts_with("scripts/"))
        .count();
    PluginBundleCounts {
        skills,
        agents,
        hooks,
        mcp_servers,
        scripts,
        total_files: files.len(),
    }
}

pub(super) fn compute_export_totals(bundles: &[PluginBundle]) -> ExportTotals {
    ExportTotals {
        plugins: bundles.len(),
        files: bundles.iter().map(|b| b.files.len()).sum(),
        skills: bundles.iter().map(|b| b.counts.skills).sum(),
        agents: bundles.iter().map(|b| b.counts.agents).sum(),
    }
}
