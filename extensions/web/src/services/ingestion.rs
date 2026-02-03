use std::path::Path;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use systemprompt::identifiers::{CategoryId, SourceId};
use walkdir::WalkDir;

use crate::error::BlogError;
use crate::models::{
    ContentKind, ContentMetadata, CreateContentParams, IngestionOptions, IngestionReport,
};
use crate::repository::ContentRepository;

#[derive(Debug, Clone)]
pub struct IngestionService {
    repo: ContentRepository,
}

impl IngestionService {
    #[must_use]
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            repo: ContentRepository::new(pool),
        }
    }

    pub async fn ingest_path(
        &self,
        path: &Path,
        source_id: &SourceId,
        category_id: &CategoryId,
    ) -> Result<IngestionReport, BlogError> {
        self.ingest_path_with_options(path, source_id, category_id, IngestionOptions::default())
            .await
    }

    pub async fn ingest_path_with_options(
        &self,
        path: &Path,
        source_id: &SourceId,
        category_id: &CategoryId,
        options: IngestionOptions,
    ) -> Result<IngestionReport, BlogError> {
        let mut report = IngestionReport::new();
        let mut found_slugs: Vec<String> = Vec::new();

        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_type().is_file() && e.path().extension().is_some_and(|ext| ext == "md")
            })
        {
            report.files_found += 1;

            match self
                .ingest_file_tracking_slug(entry.path(), source_id, category_id)
                .await
            {
                Ok(slug) => {
                    report.files_processed += 1;
                    found_slugs.push(slug);
                }
                Err(e) => report.errors.push(format!(
                    "Failed to ingest {}: {}",
                    entry.path().display(),
                    e
                )),
            }
        }

        if options.delete_orphans && !found_slugs.is_empty() {
            match self
                .repo
                .delete_orphaned_slugs(source_id, &found_slugs)
                .await
            {
                Ok(deleted) => {
                    #[allow(clippy::cast_possible_truncation)]
                    {
                        report.orphans_deleted = deleted as usize;
                    }
                    if deleted > 0 {
                        tracing::info!(
                            source_id = %source_id,
                            deleted = deleted,
                            "Deleted orphaned content records"
                        );
                    }
                }
                Err(e) => {
                    report.errors.push(format!(
                        "Failed to delete orphaned slugs for source {source_id}: {e}"
                    ));
                }
            }
        }

        Ok(report)
    }

    async fn ingest_file_tracking_slug(
        &self,
        path: &Path,
        source_id: &SourceId,
        category_id: &CategoryId,
    ) -> Result<String, BlogError> {
        let content = std::fs::read_to_string(path)?;
        let version_hash = compute_hash(&content);

        let (metadata, body) = parse_markdown(&content)?;
        let published_at = parse_datetime(&metadata.published_at)?;
        let slug = metadata.slug.clone();

        let kind = metadata
            .kind
            .parse::<ContentKind>()
            .unwrap_or(ContentKind::Blog);

        let links = serde_json::to_value(&metadata.links)?;
        let after_reading_this = serde_json::to_value(&metadata.after_reading_this)?;
        let related_playbooks = serde_json::to_value(&metadata.related_playbooks)?;
        let related_code = serde_json::to_value(&metadata.related_code)?;
        let related_docs = serde_json::to_value(&metadata.related_docs)?;

        let params = CreateContentParams::new(
            metadata.slug,
            metadata.title,
            metadata.description,
            body,
            metadata.author,
            published_at,
            source_id.clone(),
        )
        .with_version_hash(version_hash)
        .with_keywords(metadata.keywords)
        .with_kind(kind)
        .with_image(metadata.image)
        .with_category_id(Some(category_id.clone()))
        .with_category(metadata.category)
        .with_links(links)
        .with_after_reading_this(after_reading_this)
        .with_related_playbooks(related_playbooks)
        .with_related_code(related_code)
        .with_related_docs(related_docs);

        self.repo.create(&params).await?;

        Ok(slug)
    }
}

fn parse_markdown(content: &str) -> Result<(ContentMetadata, String), BlogError> {
    if !content.starts_with("---") {
        return Err(BlogError::Parse("Missing YAML frontmatter".to_string()));
    }

    let rest = &content[3..];
    let end_idx = rest
        .find("---")
        .ok_or_else(|| BlogError::Parse("Unclosed YAML frontmatter".to_string()))?;

    let frontmatter = &rest[..end_idx].trim();
    let body = rest[end_idx + 3..].trim().to_string();

    let metadata: ContentMetadata = serde_yaml::from_str(frontmatter)?;

    Ok((metadata, body))
}

fn parse_datetime(s: &str) -> Result<DateTime<Utc>, BlogError> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }

    if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let dt = date
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| BlogError::Parse(format!("Invalid time for date: {s}")))?;
        return Ok(DateTime::from_naive_utc_and_offset(dt, Utc));
    }

    Err(BlogError::Parse(format!("Invalid datetime: {s}")))
}

fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
