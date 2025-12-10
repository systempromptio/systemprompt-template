use crate::models::{Content, ContentMetadata, IngestionReport};
use crate::repository::ContentRepository;
use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};
use std::path::Path;
use systemprompt_core_database::DbPool;

#[derive(Debug)]
pub struct IngestionService {
    content_repo: ContentRepository,
}

impl IngestionService {
    pub fn new(db: DbPool) -> Self {
        Self {
            content_repo: ContentRepository::new(db),
        }
    }

    pub async fn ingest_directory(&self, path: &Path) -> Result<IngestionReport> {
        self.ingest_directory_with_source(path, None).await
    }

    pub async fn ingest_directory_with_source(
        &self,
        path: &Path,
        source_id: Option<String>,
    ) -> Result<IngestionReport> {
        self.ingest_directory_with_tracking(path, source_id, None, None)
            .await
    }

    pub async fn ingest_directory_with_tracking(
        &self,
        path: &Path,
        source_id: Option<String>,
        change_reason: Option<String>,
        changed_by: Option<String>,
    ) -> Result<IngestionReport> {
        self.ingest_directory_with_tracking_and_category(
            path,
            source_id.clone(),
            source_id,
            change_reason,
            changed_by,
        )
        .await
    }

    pub async fn ingest_directory_with_tracking_and_category(
        &self,
        path: &Path,
        source_id: Option<String>,
        category_id: Option<String>,
        change_reason: Option<String>,
        changed_by: Option<String>,
    ) -> Result<IngestionReport> {
        self.ingest_directory_with_types(
            path,
            source_id,
            category_id,
            change_reason,
            changed_by,
            &["article", "post", "page", "blog", "documentation"],
        )
        .await
    }

    pub async fn ingest_directory_with_types(
        &self,
        path: &Path,
        source_id: Option<String>,
        category_id: Option<String>,
        change_reason: Option<String>,
        changed_by: Option<String>,
        allowed_content_types: &[&str],
    ) -> Result<IngestionReport> {
        let mut report = IngestionReport::new();

        let markdown_files = Self::scan_markdown_files(path, allowed_content_types)?;
        report.files_found = markdown_files.len();

        for file_path in markdown_files {
            let reason = change_reason
                .clone()
                .unwrap_or_else(|| "system ingestion".to_string());
            let who = changed_by.clone().unwrap_or_else(|| "system".to_string());

            match self
                .ingest_file_with_tracking_and_category(
                    &file_path,
                    source_id.clone(),
                    category_id.clone(),
                    &reason,
                    &who,
                    allowed_content_types,
                )
                .await
            {
                Ok(_) => {
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

    async fn ingest_file_with_tracking_and_category(
        &self,
        path: &Path,
        source_id: Option<String>,
        category_id: Option<String>,
        _change_reason: &str,
        _changed_by: &str,
        allowed_content_types: &[&str],
    ) -> Result<(usize, bool)> {
        let markdown_text = std::fs::read_to_string(path)?;
        let (metadata, content_text) =
            Self::parse_frontmatter(&markdown_text, allowed_content_types)?;

        let resolved_source_id = source_id.unwrap_or_else(|| "unknown".to_string());
        let resolved_category_id = metadata
            .category
            .clone()
            .or(category_id)
            .unwrap_or_else(|| resolved_source_id.clone());

        let new_content = Self::create_content_from_metadata(
            path,
            &metadata,
            &content_text,
            resolved_source_id.clone(),
            resolved_category_id.clone(),
        )?;

        match self
            .content_repo
            .get_by_source_and_slug(&resolved_source_id, &new_content.slug)
            .await?
        {
            None => {
                let version_hash = Self::compute_version_hash(&new_content);
                let created = self
                    .content_repo
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

                Self::update_fts_index(&created).await?;
                Ok((1, true))
            },
            Some(_existing) => Ok((0, false)),
        }
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

    async fn update_fts_index(_content: &Content) -> Result<()> {
        Ok(())
    }

    fn scan_markdown_files(
        dir: &Path,
        allowed_content_types: &[&str],
    ) -> Result<Vec<std::path::PathBuf>> {
        use walkdir::WalkDir;

        let mut files = Vec::new();

        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "md" {
                        if !Self::should_include_file(entry.path(), allowed_content_types) {
                            continue;
                        }
                        files.push(entry.path().to_path_buf());
                    }
                }
            }
        }

        Ok(files)
    }

    fn should_include_file(path: &Path, allowed_content_types: &[&str]) -> bool {
        let markdown_text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) => {
                println!("   ⚠  SKIPPED: Cannot read file - {e}");
                return false;
            },
        };

        let (_, _) = match Self::parse_frontmatter(&markdown_text, allowed_content_types) {
            Ok(result) => result,
            Err(e) => {
                println!("   ⚠  SKIPPED: Invalid frontmatter - {e}");
                return false;
            },
        };

        true
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
            links: serde_json::Value::Array(vec![]),
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
