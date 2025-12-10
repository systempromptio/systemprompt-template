use crate::models::paper::PaperMetadata;
use crate::models::{Content, ContentMetadata, IngestionReport};
use crate::repository::ContentRepository;
use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};
use std::path::Path;
use systemprompt_core_database::DbPool;

#[derive(Debug)]
pub struct GenericIngestionService {
    content_repo: ContentRepository,
}

impl GenericIngestionService {
    pub fn new(db: DbPool) -> Self {
        Self {
            content_repo: ContentRepository::new(db),
        }
    }

    pub async fn ingest_directory(
        &self,
        path: &Path,
        source_id: String,
        category_id: String,
        allowed_content_types: &[&str],
        override_existing: bool,
    ) -> Result<IngestionReport> {
        let mut report = IngestionReport::new();

        let (markdown_files, validation_errors) =
            Self::scan_markdown_files(path, allowed_content_types)?;
        report.files_found = markdown_files.len() + validation_errors.len();
        report.errors.extend(validation_errors);

        for file_path in markdown_files {
            match self
                .ingest_file(
                    &file_path,
                    source_id.clone(),
                    category_id.clone(),
                    allowed_content_types,
                    override_existing,
                )
                .await
            {
                Ok(()) => {
                    report.files_processed += 1;
                },
                Err(e) => {
                    report
                        .errors
                        .push(format!("{}: {}", file_path.display(), e));
                },
            }
        }

        Ok(report)
    }

    async fn ingest_file(
        &self,
        path: &Path,
        source_id: String,
        category_id: String,
        allowed_content_types: &[&str],
        override_existing: bool,
    ) -> Result<()> {
        let markdown_text = std::fs::read_to_string(path)?;
        let (metadata, content_text) =
            Self::parse_frontmatter(&markdown_text, allowed_content_types)?;

        let resolved_category_id = metadata.category.clone().unwrap_or(category_id);

        let final_content_text = if metadata.kind == "paper" {
            Self::load_paper_chapters(&markdown_text)?
        } else {
            content_text
        };

        let new_content = Self::create_content_from_metadata(
            path,
            &metadata,
            &final_content_text,
            source_id,
            resolved_category_id,
        )?;

        let existing_content = self
            .content_repo
            .get_by_source_and_slug(&new_content.source_id, &new_content.slug)
            .await?;

        match existing_content {
            None => {
                let version_hash = Self::compute_version_hash(&new_content);
                self.content_repo
                    .create(
                        &new_content.slug,
                        &new_content.title,
                        &new_content.description,
                        &new_content.body,
                        &new_content.author,
                        new_content.published_at,
                        &new_content.keywords,
                        &new_content.kind,
                        new_content.image.as_deref(),
                        new_content.category_id.as_deref(),
                        &new_content.source_id,
                        &version_hash,
                        &new_content.links,
                    )
                    .await?;
            },
            Some(existing) => {
                if override_existing {
                    let version_hash = Self::compute_version_hash(&new_content);

                    self.content_repo
                        .update(
                            &existing.id,
                            &new_content.title,
                            &new_content.description,
                            &new_content.body,
                            &new_content.keywords,
                            new_content.image.as_deref(),
                            &version_hash,
                        )
                        .await?;
                }
            },
        }

        Ok(())
    }

    fn parse_frontmatter(
        markdown: &str,
        allowed_content_types: &[&str],
    ) -> Result<(ContentMetadata, String)> {
        let parts: Vec<&str> = markdown.splitn(3, "---").collect();

        if parts.len() < 3 {
            return Err(anyhow!("Invalid frontmatter format"));
        }

        let metadata: ContentMetadata = serde_yaml::from_str(parts[1])?;
        metadata.validate_with_allowed_types(allowed_content_types)?;

        let content = parts[2].trim().to_string();

        Ok((metadata, content))
    }

    fn scan_markdown_files(
        dir: &Path,
        allowed_content_types: &[&str],
    ) -> Result<(Vec<std::path::PathBuf>, Vec<String>)> {
        use walkdir::WalkDir;

        let mut files = Vec::new();
        let mut errors = Vec::new();

        for entry in WalkDir::new(dir)
            .min_depth(1)
            .max_depth(1)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let Some(ext) = entry.path().extension() else {
                continue;
            };

            if ext != "md" {
                continue;
            }

            match Self::validate_markdown_file(entry.path(), allowed_content_types) {
                Ok(()) => files.push(entry.path().to_path_buf()),
                Err(e) => errors.push(format!("{}: {}", entry.path().display(), e)),
            }
        }

        Ok((files, errors))
    }

    fn validate_markdown_file(path: &Path, allowed_content_types: &[&str]) -> Result<()> {
        let markdown_text = std::fs::read_to_string(path)?;
        let (metadata, _) = Self::parse_frontmatter(&markdown_text, allowed_content_types)?;

        if metadata.kind == "paper" {
            Self::validate_paper_frontmatter(&markdown_text)?;
        }

        Ok(())
    }

    fn validate_paper_frontmatter(markdown: &str) -> Result<()> {
        let parts: Vec<&str> = markdown.splitn(3, "---").collect();
        if parts.len() < 3 {
            return Err(anyhow!("Invalid frontmatter format for paper"));
        }

        let paper_meta: PaperMetadata = serde_yaml::from_str(parts[1])
            .map_err(|e| anyhow!("Failed to parse paper metadata: {}", e))?;

        paper_meta.validate()?;
        paper_meta.validate_section_ids_unique()?;

        Ok(())
    }

    fn load_paper_chapters(markdown: &str) -> Result<String> {
        let parts: Vec<&str> = markdown.splitn(3, "---").collect();
        if parts.len() < 3 {
            return Err(anyhow!("Invalid frontmatter format for paper"));
        }

        let frontmatter = parts[1];
        let paper_meta: PaperMetadata = serde_yaml::from_str(frontmatter)
            .map_err(|e| anyhow!("Failed to parse paper metadata: {}", e))?;

        let Some(chapters_path) = &paper_meta.chapters_path else {
            return Ok(markdown.to_string());
        };

        let chapters_dir = Path::new(chapters_path);
        let mut chapter_content = String::new();

        for section in &paper_meta.sections {
            if let Some(file) = &section.file {
                let file_path = chapters_dir.join(file);
                let content = std::fs::read_to_string(&file_path).map_err(|e| {
                    anyhow!(
                        "Failed to read chapter file '{}': {}",
                        file_path.display(),
                        e
                    )
                })?;
                if !chapter_content.is_empty() {
                    chapter_content.push_str("\n\n");
                }
                chapter_content.push_str(&format!(
                    "<!-- SECTION_START: {} -->\n{}\n<!-- SECTION_END: {} -->",
                    section.id,
                    content.trim(),
                    section.id
                ));
            }
        }

        if chapter_content.is_empty() {
            Ok(markdown.to_string())
        } else {
            Ok(format!("---\n{frontmatter}\n---\n\n{chapter_content}"))
        }
    }

    fn create_content_from_metadata(
        _file_path: &Path,
        metadata: &ContentMetadata,
        content_text: &str,
        source_id: String,
        category_id: String,
    ) -> Result<Content> {
        let id = uuid::Uuid::new_v4().to_string();
        let slug = metadata.slug.clone();

        let published_at = chrono::NaiveDate::parse_from_str(&metadata.published_at, "%Y-%m-%d")
            .map_err(|e| {
                anyhow!(
                    "Invalid published_at date '{}': {}",
                    metadata.published_at,
                    e
                )
            })?
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| anyhow!("Failed to create datetime"))?
            .and_local_timezone(chrono::Utc)
            .single()
            .ok_or_else(|| anyhow!("Ambiguous timezone conversion"))?;

        let links_vec: Vec<crate::models::ContentLinkMetadata> = metadata
            .links
            .iter()
            .map(|link| crate::models::ContentLinkMetadata {
                title: link.title.clone(),
                url: link.url.clone(),
            })
            .collect();

        Ok(Content {
            id,
            slug,
            title: metadata.title.clone(),
            description: metadata.description.clone(),
            body: content_text.to_string(),
            author: metadata.author.clone(),
            published_at,
            keywords: metadata.keywords.clone(),
            kind: metadata.kind.clone(),
            image: metadata.image.clone(),
            category_id: Some(category_id),
            source_id,
            version_hash: String::new(),
            links: serde_json::to_value(&links_vec).unwrap_or_default(),
            updated_at: Some(chrono::Utc::now()),
        })
    }

    fn compute_version_hash(content: &Content) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.title.as_bytes());
        hasher.update(content.body.as_bytes());
        hasher.update(content.description.as_bytes());
        hasher.update(content.author.as_bytes());
        hasher.update(content.published_at.to_string().as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
