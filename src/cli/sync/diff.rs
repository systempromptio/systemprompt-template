use super::models::{
    ContentDiffItem, ContentDiffResult, DiffStatus, DiskContent, DiskSkill, SkillDiffItem,
    SkillsDiffResult,
};
use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use systemprompt_core_agent::models::Skill;
use systemprompt_core_agent::repository::SkillRepository;
use systemprompt_core_blog::models::Content;
use systemprompt_core_blog::repository::ContentRepository;
use systemprompt_core_database::{DatabaseProvider, DbPool};
use walkdir::WalkDir;

pub struct ContentDiffCalculator {
    content_repo: ContentRepository,
}

impl ContentDiffCalculator {
    pub fn new(db: DbPool) -> Self {
        Self {
            content_repo: ContentRepository::new(db),
        }
    }

    pub async fn calculate_diff(
        &self,
        source_id: &str,
        disk_path: &Path,
        allowed_types: &[String],
    ) -> Result<ContentDiffResult> {
        let db_content = self.content_repo.list_by_source(source_id).await?;
        let db_map: HashMap<String, Content> =
            db_content.into_iter().map(|c| (c.slug.clone(), c)).collect();

        let disk_items = self.scan_disk_content(disk_path, allowed_types)?;

        let mut result = ContentDiffResult {
            source_id: source_id.to_string(),
            ..Default::default()
        };

        for (slug, disk_item) in &disk_items {
            let disk_hash = compute_content_hash(&disk_item.body, &disk_item.title);

            match db_map.get(slug) {
                None => {
                    result.added.push(ContentDiffItem {
                        slug: slug.clone(),
                        source_id: source_id.to_string(),
                        status: DiffStatus::Added,
                        disk_hash: Some(disk_hash),
                        db_hash: None,
                        disk_updated_at: None,
                        db_updated_at: None,
                        title: Some(disk_item.title.clone()),
                    });
                }
                Some(db_item) => {
                    if db_item.version_hash != disk_hash {
                        result.modified.push(ContentDiffItem {
                            slug: slug.clone(),
                            source_id: source_id.to_string(),
                            status: DiffStatus::Modified,
                            disk_hash: Some(disk_hash),
                            db_hash: Some(db_item.version_hash.clone()),
                            disk_updated_at: None,
                            db_updated_at: db_item.updated_at,
                            title: Some(disk_item.title.clone()),
                        });
                    } else {
                        result.unchanged += 1;
                    }
                }
            }
        }

        for (slug, db_item) in &db_map {
            if !disk_items.contains_key(slug) {
                result.removed.push(ContentDiffItem {
                    slug: slug.clone(),
                    source_id: source_id.to_string(),
                    status: DiffStatus::Removed,
                    disk_hash: None,
                    db_hash: Some(db_item.version_hash.clone()),
                    disk_updated_at: None,
                    db_updated_at: db_item.updated_at,
                    title: Some(db_item.title.clone()),
                });
            }
        }

        Ok(result)
    }

    fn scan_disk_content(
        &self,
        path: &Path,
        allowed_types: &[String],
    ) -> Result<HashMap<String, DiskContent>> {
        let mut items = HashMap::new();

        if !path.exists() {
            return Ok(items);
        }

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        {
            let file_path = entry.path();
            match parse_content_file(file_path, allowed_types) {
                Ok(Some(content)) => {
                    items.insert(content.slug.clone(), content);
                }
                Ok(None) => {}
                Err(e) => {
                    eprintln!("Warning: Failed to parse {}: {}", file_path.display(), e);
                }
            }
        }

        Ok(items)
    }
}

pub struct SkillsDiffCalculator {
    skill_repo: SkillRepository,
}

impl SkillsDiffCalculator {
    pub fn new(db: Arc<dyn DatabaseProvider>) -> Self {
        Self {
            skill_repo: SkillRepository::new(db),
        }
    }

    pub async fn calculate_diff(&self, skills_path: &Path) -> Result<SkillsDiffResult> {
        let db_skills = self.skill_repo.list_all().await?;
        let db_map: HashMap<String, Skill> = db_skills
            .into_iter()
            .map(|s| (s.skill_id.as_str().to_string(), s))
            .collect();

        let disk_skills = self.scan_disk_skills(skills_path)?;

        let mut result = SkillsDiffResult::default();

        for (skill_id, disk_skill) in &disk_skills {
            let disk_hash = compute_skill_hash(disk_skill);

            match db_map.get(skill_id) {
                None => {
                    result.added.push(SkillDiffItem {
                        skill_id: skill_id.clone(),
                        file_path: disk_skill.file_path.clone(),
                        status: DiffStatus::Added,
                        disk_hash: Some(disk_hash),
                        db_hash: None,
                        name: Some(disk_skill.name.clone()),
                    });
                }
                Some(db_skill) => {
                    let db_hash = compute_db_skill_hash(db_skill);
                    if db_hash != disk_hash {
                        result.modified.push(SkillDiffItem {
                            skill_id: skill_id.clone(),
                            file_path: disk_skill.file_path.clone(),
                            status: DiffStatus::Modified,
                            disk_hash: Some(disk_hash),
                            db_hash: Some(db_hash),
                            name: Some(disk_skill.name.clone()),
                        });
                    } else {
                        result.unchanged += 1;
                    }
                }
            }
        }

        for (skill_id, db_skill) in &db_map {
            if !disk_skills.contains_key(skill_id) {
                result.removed.push(SkillDiffItem {
                    skill_id: skill_id.clone(),
                    file_path: db_skill.file_path.clone(),
                    status: DiffStatus::Removed,
                    disk_hash: None,
                    db_hash: Some(compute_db_skill_hash(db_skill)),
                    name: Some(db_skill.name.clone()),
                });
            }
        }

        Ok(result)
    }

    fn scan_disk_skills(&self, path: &Path) -> Result<HashMap<String, DiskSkill>> {
        let mut skills = HashMap::new();

        if !path.exists() {
            return Ok(skills);
        }

        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let skill_path = entry.path();

            if !skill_path.is_dir() {
                continue;
            }

            let index_path = skill_path.join("index.md");
            let skill_md_path = skill_path.join("SKILL.md");

            let md_path = if index_path.exists() {
                index_path
            } else if skill_md_path.exists() {
                skill_md_path
            } else {
                continue;
            };

            match parse_skill_file(&md_path, &skill_path) {
                Ok(skill) => {
                    skills.insert(skill.skill_id.clone(), skill);
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse skill at {}: {}",
                        skill_path.display(),
                        e
                    );
                }
            }
        }

        Ok(skills)
    }
}

fn parse_content_file(path: &Path, allowed_types: &[String]) -> Result<Option<DiskContent>> {
    let content = std::fs::read_to_string(path)?;

    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return Err(anyhow!("Invalid frontmatter format"));
    }

    let frontmatter: serde_yaml::Value = serde_yaml::from_str(parts[1])?;
    let body = parts[2].trim().to_string();

    let kind = frontmatter
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("article");

    if !allowed_types.iter().any(|t| t == kind) {
        return Ok(None);
    }

    let slug = frontmatter
        .get("slug")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing slug in frontmatter"))?
        .to_string();

    let title = frontmatter
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing title in frontmatter"))?
        .to_string();

    Ok(Some(DiskContent { slug, title, body }))
}

fn parse_skill_file(md_path: &Path, skill_dir: &Path) -> Result<DiskSkill> {
    let content = std::fs::read_to_string(md_path)?;

    let parts: Vec<&str> = content.splitn(3, "---").collect();
    if parts.len() < 3 {
        return Err(anyhow!("Invalid frontmatter format"));
    }

    let frontmatter: serde_yaml::Value = serde_yaml::from_str(parts[1])?;
    let instructions = parts[2].trim().to_string();

    let dir_name = skill_dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow!("Invalid skill directory name"))?;

    let skill_id = dir_name.replace('-', "_");

    let name = frontmatter
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or(dir_name)
        .to_string();

    let description = frontmatter
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(DiskSkill {
        skill_id,
        name,
        description,
        instructions,
        file_path: md_path.to_string_lossy().to_string(),
    })
}

pub fn compute_content_hash(body: &str, title: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(title.as_bytes());
    hasher.update(body.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn compute_skill_hash(skill: &DiskSkill) -> String {
    let mut hasher = Sha256::new();
    hasher.update(skill.name.as_bytes());
    hasher.update(skill.description.as_bytes());
    hasher.update(skill.instructions.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn compute_db_skill_hash(skill: &Skill) -> String {
    let mut hasher = Sha256::new();
    hasher.update(skill.name.as_bytes());
    hasher.update(skill.description.as_bytes());
    hasher.update(skill.instructions.as_bytes());
    format!("{:x}", hasher.finalize())
}
