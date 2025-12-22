//! Content ingestion service - parse and import markdown files.

use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::path::Path;
use std::sync::Arc;
use systemprompt::identifiers::{CategoryId, SourceId};
use walkdir::WalkDir;

use crate::config::ContentSource;
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

    /// Ingest content from a configured source.
    pub async fn ingest_source(&self, source: &ContentSource) -> Result<IngestionReport, BlogError> {
        let mut report = IngestionReport::new();

        if !source.enabled {
            return Ok(report);
        }

        let path = &source.path;
        if !path.exists() {
            report.errors.push(format!("Source path does not exist: {}", path.display()));
            return Ok(report);
        }

        // Walk the directory and find markdown files
        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_type().is_file()
                    && e.path()
                        .extension()
                        .map(|ext| ext == "md")
                        .unwrap_or(false)
            })
        {
            report.files_found += 1;

            match self
                .ingest_file(entry.path(), &source.source_id, &source.category_id)
                .await
            {
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
        source_id: &str,
        category_id: &str,
    ) -> Result<(), BlogError> {
        let content = std::fs::read_to_string(path)?;
        let version_hash = compute_hash(&content);

        // Parse frontmatter and body
        let (metadata, body) = parse_markdown(&content)?;

        // Parse published_at
        let published_at = parse_datetime(&metadata.published_at)?;

        // Parse content kind
        let kind = metadata
            .kind
            .parse::<ContentKind>()
            .unwrap_or(ContentKind::Article);

        // Build links JSON
        let links = serde_json::to_value(&metadata.links)?;

        let source_id = SourceId::new(source_id.to_string());
        let category_id = CategoryId::new(category_id.to_string());

        let params = CreateContentParams::new(
            metadata.slug,
            metadata.title,
            metadata.description,
            body,
            metadata.author,
            published_at,
            source_id,
        )
        .with_version_hash(version_hash)
        .with_keywords(metadata.keywords)
        .with_kind(kind)
        .with_image(metadata.image)
        .with_category_id(Some(category_id))
        .with_links(links);

        self.repo.create(&params).await?;

        Ok(())
    }
}

/// Parse markdown frontmatter and body.
fn parse_markdown(content: &str) -> Result<(ContentMetadata, String), BlogError> {
    // Check for YAML frontmatter
    if !content.starts_with("---") {
        return Err(BlogError::Parse("Missing YAML frontmatter".to_string()));
    }

    // Find the end of frontmatter
    let rest = &content[3..];
    let end_idx = rest
        .find("---")
        .ok_or_else(|| BlogError::Parse("Unclosed YAML frontmatter".to_string()))?;

    let frontmatter = &rest[..end_idx].trim();
    let body = rest[end_idx + 3..].trim().to_string();

    // Parse YAML
    let metadata: ContentMetadata = serde_yaml::from_str(frontmatter)?;

    Ok((metadata, body))
}

/// Parse datetime from string.
fn parse_datetime(s: &str) -> Result<DateTime<Utc>, BlogError> {
    // Try ISO 8601 format first
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Try date-only format
    if let Ok(date) = chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let dt = date.and_hms_opt(0, 0, 0).unwrap();
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
