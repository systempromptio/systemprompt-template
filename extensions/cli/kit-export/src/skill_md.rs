//! The `SKILL.md` a kit carries for one skill: the instance's file with a
//! frontmatter that also says what `config.yaml` said.
//!
//! On the instance a skill's name, tags and categories live in
//! `services/skills/<id>/config.yaml` and the `SKILL.md` frontmatter carries
//! only `name` and `description`. A kit has no `config.yaml` — core's import
//! reads `title`, `tags`, `category` and `display_category` from the
//! frontmatter — so the export merges the two, keeps the body verbatim, and
//! the round trip loses nothing.

use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use systemprompt::manifest::services::split_frontmatter;

#[derive(Debug, Default, Deserialize)]
struct SkillConfigFields {
    name: String,
    description: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    display_category: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct ExistingFront {
    #[serde(default)]
    hosts: Vec<String>,
}

#[derive(Debug, Serialize)]
struct KitFront<'a> {
    name: &'a str,
    title: &'a str,
    description: &'a str,
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    tags: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    display_category: Option<&'a str>,
    #[serde(skip_serializing_if = "<[String]>::is_empty")]
    hosts: &'a [String],
}

pub(crate) fn render(skill_dir: &Path, kebab: &str) -> Result<String> {
    let config_path = skill_dir.join("config.yaml");
    let config: SkillConfigFields = serde_yaml::from_str(
        &std::fs::read_to_string(&config_path)
            .with_context(|| config_path.display().to_string())?,
    )
    .with_context(|| config_path.display().to_string())?;
    let md_path = skill_dir.join("SKILL.md");
    let raw = std::fs::read_to_string(&md_path).with_context(|| md_path.display().to_string())?;
    // Why: a frontmatter that does not parse as the fields carried over (hosts)
    // simply carries none; the body is still exported verbatim.
    let (existing, body) = split_frontmatter(&raw).map_or_else(
        || (ExistingFront::default(), raw.as_str()),
        |f| {
            (
                // Why: discard-ok: the body, not the old frontmatter, is kept
                serde_yaml::from_str::<ExistingFront>(f.yaml).unwrap_or_default(),
                f.body,
            )
        },
    );
    let front = serde_yaml::to_string(&KitFront {
        name: kebab,
        title: &config.name,
        description: &config.description,
        tags: &config.tags,
        category: config.category.as_deref(),
        display_category: config.display_category.as_deref(),
        hosts: &existing.hosts,
    })?;
    Ok(format!("---\n{front}---\n{body}"))
}
