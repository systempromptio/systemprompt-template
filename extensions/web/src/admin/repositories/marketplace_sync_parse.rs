use std::path::{Path, PathBuf};

use systemprompt::identifiers::SkillId;

use crate::admin::types::ParsedSkill;

use super::marketplace_sync_archive::MAX_FILE_SIZE;
use crate::error::MarketplaceError;

fn find_skills_dir(extract_dir: &Path) -> Option<PathBuf> {
    let custom_skills = extract_dir.join("plugins").join("custom").join("skills");
    if custom_skills.is_dir() {
        return Some(custom_skills);
    }

    if let Ok(entries) = std::fs::read_dir(extract_dir) {
        let dirs: Vec<_> = entries
            .filter_map(Result::ok)
            .filter(|e| e.path().is_dir())
            .collect();
        if dirs.len() == 1 {
            let inner = dirs[0].path();
            let nested = inner.join("plugins").join("custom").join("skills");
            if nested.is_dir() {
                return Some(nested);
            }
            let direct_skills = inner.join("skills");
            if direct_skills.is_dir() {
                return Some(direct_skills);
            }
        }
    }

    let root_skills = extract_dir.join("skills");
    if root_skills.is_dir() {
        return Some(root_skills);
    }

    None
}

fn parse_skill_md(raw: &str) -> Result<(String, String, String), MarketplaceError> {
    let trimmed = raw.trim();
    if !trimmed.starts_with("---") {
        return Err(MarketplaceError::Internal(
            "SKILL.md missing YAML frontmatter (no opening ---)".to_string(),
        ));
    }

    let after_first = &trimmed[3..];
    let end_idx = after_first.find("\n---").ok_or_else(|| {
        MarketplaceError::Internal("SKILL.md missing closing --- for frontmatter".to_string())
    })?;

    let yaml_block = &after_first[..end_idx].trim();
    let content = after_first[end_idx + 4..].trim().to_string();

    let frontmatter: serde_yaml::Value = serde_yaml::from_str(yaml_block)?;

    let name = frontmatter
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let description = frontmatter
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok((name, description, content))
}

pub fn parse_skills_from_directory(
    extract_dir: &Path,
    base_skills_dir: &Path,
) -> Result<Vec<ParsedSkill>, MarketplaceError> {
    let Some(skills_dir) = find_skills_dir(extract_dir) else {
        return Ok(Vec::new());
    };

    let mut skills = Vec::new();

    let Ok(entries) = std::fs::read_dir(&skills_dir) else {
        return Ok(Vec::new());
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let skill_id = entry.file_name().to_string_lossy().into_owned();
        let skill_md_path = path.join("SKILL.md");

        if !skill_md_path.exists() {
            continue;
        }

        let raw = std::fs::read_to_string(&skill_md_path)?;
        if raw.len() > MAX_FILE_SIZE {
            tracing::warn!(skill_id = %skill_id, "Skipping oversized SKILL.md");
            continue;
        }

        let (name, description, content) = match parse_skill_md(&raw) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!(skill_id = %skill_id, error = %e, "Failed to parse SKILL.md, skipping");
                continue;
            }
        };

        let base_skill_id = determine_base_skill_id(&skill_id, base_skills_dir);

        skills.push(ParsedSkill {
            skill_id: SkillId::new(skill_id),
            name,
            description,
            content,
            tags: Vec::new(),
            base_skill_id: base_skill_id.map(SkillId::new),
        });
    }

    Ok(skills)
}

fn determine_base_skill_id(skill_id: &str, base_skills_dir: &Path) -> Option<String> {
    if base_skills_dir.join(skill_id).is_dir() {
        return Some(skill_id.to_string());
    }
    let snake_id = skill_id.replace('-', "_");
    if base_skills_dir.join(&snake_id).is_dir() {
        return Some(snake_id);
    }
    None
}
