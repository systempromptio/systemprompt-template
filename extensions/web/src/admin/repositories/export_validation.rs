use super::export::{
    ExportTotals, ManifestAuthor, PluginBundle, PluginBundleCounts, PluginFile, PluginManifest,
};

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

    for w in &warnings {
        tracing::warn!(warning = %w, "Export validation");
    }
}

pub(super) fn build_manifest(
    plugin: &systemprompt::models::PluginConfig,
    version_override: Option<&str>,
) -> PluginManifest {
    PluginManifest {
        name: plugin.id.clone(),
        description: plugin.description.clone(),
        version: version_override.unwrap_or(&plugin.version).to_string(),
        author: Some(ManifestAuthor {
            name: plugin.author.name.clone(),
            email: plugin.author.email.clone(),
        }),
        hooks: Some("./hooks/hooks.json".to_string()),
        keywords: Vec::new(),
    }
}

pub(super) fn compute_content_version(base_version: &str, files: &[PluginFile]) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    for file in files {
        hasher.update(file.path.as_bytes());
        hasher.update(b"\0");
        hasher.update(file.content.as_bytes());
        hasher.update(b"\0");
    }
    let hash = hasher.finalize();
    let short_hash = hex::encode(&hash[..4]);

    format!("{base_version}+{short_hash}")
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
    let mcp_servers = usize::from(files.iter().any(|f| f.path == ".mcp.json"));
    let scripts = files
        .iter()
        .filter(|f| f.path.starts_with("scripts/"))
        .count();
    PluginBundleCounts {
        skills,
        agents,
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
