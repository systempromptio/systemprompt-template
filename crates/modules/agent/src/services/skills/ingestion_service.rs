use crate::models::Skill;
use crate::repository::SkillRepository;
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::path::Path;
use std::sync::Arc;
use systemprompt_core_blog::IngestionReport;
use systemprompt_core_database::DatabaseProvider;
use systemprompt_identifiers::{SkillId, SourceId};

#[derive(Debug)]
pub struct SkillIngestionService {
    skill_repo: SkillRepository,
}

impl SkillIngestionService {
    pub fn new(db: Arc<dyn DatabaseProvider>) -> Self {
        Self {
            skill_repo: SkillRepository::new(db),
        }
    }

    pub async fn ingest_directory(
        &self,
        path: &Path,
        source_id: SourceId,
    ) -> Result<IngestionReport> {
        let mut report = IngestionReport::new();

        let skill_dirs = self.scan_skill_directories(path)?;
        report.files_found = skill_dirs.len();

        for skill_dir in skill_dirs {
            match self.ingest_skill(&skill_dir, source_id.clone()).await {
                Ok(_) => {
                    report.files_processed += 1;
                },
                Err(e) => {
                    report
                        .errors
                        .push(format!("{}: {}", skill_dir.display(), e));
                },
            }
        }

        Ok(report)
    }

    async fn ingest_skill(&self, skill_dir: &Path, source_id: SourceId) -> Result<()> {
        let index_file = skill_dir.join("index.md");

        if !index_file.exists() {
            return Err(anyhow!("No index.md found in skill directory"));
        }

        let markdown_text = std::fs::read_to_string(&index_file)?;
        let (metadata, instructions) = Self::parse_skill_markdown(&markdown_text)?;

        let skill_id = metadata
            .get("slug")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Skill must have 'slug' in frontmatter"))?
            .replace('-', "_")
            .to_string();

        let name = metadata
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Skill must have 'title' in frontmatter"))?
            .to_string();

        let description = metadata
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let file_path = index_file.to_string_lossy().to_string();
        let enabled = metadata
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let allowed_tools = metadata
            .get("allowed_tools")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let tags = metadata
            .get("keywords")
            .and_then(|v| v.as_sequence())
            .map(|seq| {
                seq.iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let skill = Skill {
            skill_id: SkillId::new(skill_id),
            file_path: file_path.clone(),
            name,
            description,
            instructions,
            enabled,
            allowed_tools,
            tags,
            category_id: None,
            source_id,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        if let Some(existing_skill) = self.skill_repo.get_by_file_path(&file_path).await? {
            self.skill_repo.update(&existing_skill.skill_id, &skill).await?;
        } else {
            self.skill_repo.create(&skill).await?;
        }

        Ok(())
    }

    fn scan_skill_directories(&self, dir: &Path) -> Result<Vec<std::path::PathBuf>> {
        use walkdir::WalkDir;

        let mut skill_dirs = Vec::new();
        let mut seen = HashSet::new();

        for entry in WalkDir::new(dir)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_dir() && entry.file_name() != "." {
                let index_file = entry.path().join("index.md");
                if index_file.exists() {
                    let path = entry.path().to_path_buf();
                    if !seen.contains(&path) {
                        skill_dirs.push(path.clone());
                        seen.insert(path);
                    }
                }
            }
        }

        Ok(skill_dirs)
    }

    fn parse_skill_markdown(markdown: &str) -> Result<(serde_yaml::Mapping, String)> {
        let parts: Vec<&str> = markdown.splitn(3, "---").collect();

        if parts.len() < 3 {
            return Err(anyhow!("Invalid frontmatter format"));
        }

        let metadata = serde_yaml::from_str::<serde_yaml::Value>(parts[1])?
            .as_mapping()
            .ok_or_else(|| anyhow!("Invalid YAML in frontmatter"))?
            .clone();

        let instructions = parts[2].trim().to_string();

        Ok((metadata, instructions))
    }
}
