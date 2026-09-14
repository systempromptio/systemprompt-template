//! Loads the skills-page entries from the `services/skills/` tree.

use std::sync::Arc;

use crate::skills_page::{SkillEntry, SkillsPageConfig};

use super::{ConfigError, load_app_paths};

pub(crate) fn load_skills_page_config() -> Result<Option<Arc<SkillsPageConfig>>, ConfigError> {
    let paths = match load_app_paths() {
        Ok(p) => p,
        Err(e) => {
            tracing::debug!(error = %e, "AppPaths not available for skills page config");
            return Ok(None);
        },
    };

    let skills_dir = paths.system().services().join("skills");

    let Some(entries) = read_skills_dir(&skills_dir)? else {
        return Ok(None);
    };

    let mut skills = parse_skill_entries(entries);

    if skills.is_empty() {
        tracing::debug!("No skills loaded for skills page");
        return Ok(None);
    }

    skills.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then_with(|| a.name.cmp(&b.name))
    });

    tracing::info!(
        skill_count = skills.len(),
        "Loaded skills page config from services/skills/"
    );

    Ok(Some(Arc::new(SkillsPageConfig { skills })))
}

#[doc(hidden)]
pub fn read_skills_dir(
    skills_dir: &std::path::Path,
) -> Result<Option<std::fs::ReadDir>, ConfigError> {
    match std::fs::read_dir(skills_dir) {
        Ok(entries) => Ok(Some(entries)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::debug!(
                path = %skills_dir.display(),
                "Skills directory does not exist"
            );
            Ok(None)
        },
        Err(e) => Err(ConfigError::Parse {
            config_name: skills_dir.display().to_string(),
            message: format!("Failed to read directory: {e}"),
        }),
    }
}

#[doc(hidden)]
pub fn parse_skill_entries(entries: std::fs::ReadDir) -> Vec<SkillEntry> {
    entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| {
            let config_path = entry.path().join("config.yaml");
            let content = std::fs::read_to_string(&config_path).ok()?;
            match serde_yaml::from_str::<SkillEntry>(&content) {
                Ok(skill) => Some(skill),
                Err(e) => {
                    tracing::warn!(
                        path = %config_path.display(),
                        error = %e,
                        "Skipping skill with unparseable config.yaml"
                    );
                    None
                },
            }
        })
        .filter(|skill: &SkillEntry| skill.enabled)
        .collect()
}
