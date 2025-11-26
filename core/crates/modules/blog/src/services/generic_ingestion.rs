use crate::models::{Content, ContentMetadata, IngestionReport};
use crate::repository::{ContentRepository, TagRepository};
use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::sync::Arc;
use systemprompt_core_database::DatabaseProvider;
use systemprompt_models::ContentLink;

pub struct GenericIngestionService {
    content_repo: ContentRepository,
    tag_repo: TagRepository,
}

impl GenericIngestionService {
    pub fn new(db: Arc<dyn DatabaseProvider>) -> Self {
        Self {
            content_repo: ContentRepository::new(db.clone()),
            tag_repo: TagRepository::new(db),
        }
    }

    pub async fn ingest_directory(
        &self,
        path: &Path,
        source_id: String,
        category_id: String,
        allowed_content_types: &[&str],
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
    ) -> Result<()> {
        let markdown_text = std::fs::read_to_string(path)?;
        let (metadata, content_text) =
            Self::parse_frontmatter(&markdown_text, allowed_content_types)?;

        let resolved_category_id = metadata.category.clone().unwrap_or(category_id);

        let new_content = Self::create_content_from_metadata(
            path,
            &metadata,
            &content_text,
            source_id,
            resolved_category_id,
        )?;

        if self
            .content_repo
            .get_by_source_and_slug(&new_content.source_id, &new_content.slug)
            .await?
            .is_none()
        {
            let mut final_content = new_content;
            let hash = Self::compute_version_hash(&final_content);
            final_content.version_hash = hash;
            self.content_repo.create(&final_content).await?;

            for tag_name in &metadata.tags {
                let tag = self.tag_repo.get_or_create(tag_name).await?;
                self.tag_repo
                    .link_to_content(&tag.id, &final_content.id)
                    .await?;
            }
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

        for entry in WalkDir::new(dir).into_iter().filter_map(std::result::Result::ok) {
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
        Self::parse_frontmatter(&markdown_text, allowed_content_types)?;
        Ok(())
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

        let links = metadata
            .links
            .iter()
            .map(|link| ContentLink::new(&link.title, &link.url))
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
            category_id,
            source_id,
            version_hash: String::new(),
            public: metadata.public,
            parent_content_id: None,
            links,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    fn compute_version_hash(content: &Content) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.title.as_bytes());
        hasher.update(content.body.as_bytes());
        hasher.update(content.description.as_bytes());
        hasher.update(content.author.as_bytes());
        hasher.update(content.published_at.to_rfc3339().as_bytes());
        format!("{:x}", hasher.finalize())
    }
}
