//! Content ingestion service - parse and import markdown files.
//!
//! Uses typed IDs (`SourceId`, `CategoryId`) for type safety.
//! Paths are expected to be pre-validated by `BlogConfigValidated`.

use std::path::Path;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use systemprompt::identifiers::{CategoryId, SourceId};
use walkdir::WalkDir;

use crate::error::BlogError;
use crate::models::{ContentKind, ContentMetadata, CreateContentParams, IngestionReport};
use crate::repository::ContentRepository;

/// Service for ingesting markdown content from the filesystem.
#[derive(Debug, Clone)]
pub struct IngestionService {
    repo: ContentRepository,
}

impl IngestionService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            repo: ContentRepository::new(pool),
        }
    }

    /// Ingest content from a validated path.
    ///
    /// Path is expected to exist and be a directory (validated at startup).
    pub async fn ingest_path(
        &self,
        path: &Path,
        source_id: &SourceId,
        category_id: &CategoryId,
    ) -> Result<IngestionReport, BlogError> {
        let mut report = IngestionReport::new();

        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_type().is_file()
                    && e.path()
                        .extension()
                        .is_some_and(|ext| ext == "md")
            })
        {
            report.files_found += 1;

            match self.ingest_file(entry.path(), source_id, category_id).await {
                Ok(_) => report.files_processed += 1,
                Err(e) => report.errors.push(format!(
                    "Failed to ingest {}: {}",
                    entry.path().display(),
                    e
                )),
            }
        }

        Ok(report)
    }

    /// Ingest a single markdown file.
    async fn ingest_file(
        &self,
        path: &Path,
        source_id: &SourceId,
        category_id: &CategoryId,
    ) -> Result<(), BlogError> {
        let content = std::fs::read_to_string(path)?;
        let version_hash = compute_hash(&content);

        let (metadata, body) = parse_markdown(&content)?;
        let published_at = parse_datetime(&metadata.published_at)?;

        let kind = metadata
            .kind
            .parse::<ContentKind>()
            .unwrap_or(ContentKind::Article);

        let links = serde_json::to_value(&metadata.links)?;

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
        .with_links(links);

        self.repo.create(&params).await?;

        Ok(())
    }
}

/// Parse markdown frontmatter and body.
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

/// Parse datetime from string.
fn parse_datetime(s: &str) -> Result<DateTime<Utc>, BlogError> {
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }

    if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let dt = date.and_hms_opt(0, 0, 0).expect("valid time");
        return Ok(DateTime::from_naive_utc_and_offset(dt, Utc));
    }

    Err(BlogError::Parse(format!("Invalid datetime: {s}")))
}

/// Compute SHA256 hash of content.
fn compute_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
