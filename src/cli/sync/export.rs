use anyhow::Result;
use std::fs;
use std::path::Path;
use systemprompt_core_agent::models::Skill;
use systemprompt_core_blog::models::Content;

pub fn escape_yaml(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

pub fn generate_content_markdown(content: &Content) -> String {
    let public_str = "true";
    let image_str = content.image.as_deref().unwrap_or("");
    let updated_at_str = content
        .updated_at
        .map(|d| d.format("%Y-%m-%d").to_string())
        .unwrap_or_default();

    format!(
        r#"---
title: "{}"
description: "{}"
author: "{}"
slug: "{}"
keywords: "{}"
image: "{}"
kind: "{}"
public: {}
tags: []
published_at: "{}"
updated_at: "{}"
---

{}"#,
        escape_yaml(&content.title),
        escape_yaml(&content.description),
        escape_yaml(&content.author),
        &content.slug,
        escape_yaml(&content.keywords),
        image_str,
        &content.kind,
        public_str,
        content.published_at.format("%Y-%m-%d"),
        updated_at_str,
        &content.body
    )
}

pub fn generate_skill_markdown(skill: &Skill) -> String {
    let tags_str = skill.tags.join(", ");
    let category = skill
        .category_id
        .as_ref()
        .map(|c| c.as_str())
        .unwrap_or("skills");

    format!(
        r#"---
title: "{}"
slug: "{}"
description: "{}"
author: "systemprompt"
published_at: "{}"
type: "skill"
category: "{}"
keywords: "{}"
---

{}"#,
        escape_yaml(&skill.name),
        skill.skill_id.as_str().replace('_', "-"),
        escape_yaml(&skill.description),
        skill.created_at.format("%Y-%m-%d"),
        category,
        tags_str,
        &skill.instructions
    )
}

pub fn generate_skill_config(skill: &Skill) -> String {
    let tags_yaml = if skill.tags.is_empty() {
        "[]".to_string()
    } else {
        skill
            .tags
            .iter()
            .map(|t| format!("  - {}", t))
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"id: {}
name: "{}"
description: "{}"
enabled: {}
version: "1.0.0"
file: "index.md"
assigned_agents:
  - content
tags:
{}"#,
        skill.skill_id.as_str(),
        escape_yaml(&skill.name),
        escape_yaml(&skill.description),
        skill.enabled,
        tags_yaml
    )
}

pub fn export_content_to_file(content: &Content, base_path: &Path, source_type: &str) -> Result<()> {
    let markdown = generate_content_markdown(content);

    let content_dir = if source_type == "blog" {
        let dir = base_path.join(&content.slug);
        fs::create_dir_all(&dir)?;
        dir.join("index.md")
    } else {
        fs::create_dir_all(base_path)?;
        base_path.join(format!("{}.md", content.slug))
    };

    fs::write(&content_dir, markdown)?;
    Ok(())
}

pub fn export_skill_to_disk(skill: &Skill, base_path: &Path) -> Result<()> {
    let skill_dir_name = skill.skill_id.as_str().replace('_', "-");
    let skill_dir = base_path.join(&skill_dir_name);
    fs::create_dir_all(&skill_dir)?;

    let index_content = generate_skill_markdown(skill);
    fs::write(skill_dir.join("index.md"), index_content)?;

    let config_content = generate_skill_config(skill);
    fs::write(skill_dir.join("config.yml"), config_content)?;

    Ok(())
}

