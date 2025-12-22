# Services & Extensions Relationship

**Services declare. Extensions implement.**

| Layer | Location | Format | Purpose |
|-------|----------|--------|---------|
| Services | `/services/` | YAML, Markdown | Configuration, content, schedules |
| Extensions | `/extensions/` | Rust | Validation, processing, storage |

```
┌─────────────────────────────────────────────────────────────┐
│                      services/ (YAML)                        │
│                                                              │
│   config.yml ─────► "what content exists, where it lives"   │
│   blog.yaml ──────► "which sources to ingest"               │
│   content/*.md ───► "the actual content"                    │
│   scheduler/*.yml ► "when jobs run"                         │
└──────────────────────────┬──────────────────────────────────┘
                           │ consumed by
┌──────────────────────────▼──────────────────────────────────┐
│                    extensions/ (Rust)                        │
│                                                              │
│   config.rs ──────► deserialize YAML into typed structs     │
│   services/*.rs ──► validate, transform, business logic     │
│   repository/*.rs ► persist to database                     │
│   jobs/*.rs ──────► scheduled processing                    │
└─────────────────────────────────────────────────────────────┘
```

---

## Complete Example: Blog Content System

### 1. Service Configuration (`services/config/blog.yaml`)

```yaml
# Declares WHAT content exists and WHERE to find it
# Extension validates and processes this configuration

content_sources:
  - source_id: "blog"
    category_id: "blog"
    path: "./services/content/blog"
    allowed_content_types:
      - article
      - tutorial
      - announcement
    enabled: true
    override_existing: false

  - source_id: "docs"
    category_id: "documentation"
    path: "./services/content/docs"
    allowed_content_types:
      - documentation
      - guide
      - reference
    enabled: true
    override_existing: true

base_url: "${BASE_URL:-http://localhost:3000}"
enable_link_tracking: true
```

### 2. Content File (`services/content/blog/hello-world/index.md`)

```markdown
---
slug: hello-world
title: Hello World
description: Getting started with SystemPrompt
author: Team
published_at: 2025-01-15
kind: article
keywords: getting-started, tutorial, hello-world
image: /images/hello-world.png
links:
  - url: https://github.com/systempromptio
    text: GitHub
    type: external
---

# Hello World

Welcome to SystemPrompt. This is your first blog post.

## Getting Started

Follow these steps to build your first extension...
```

### 3. Extension Config Struct (`extensions/blog/src/config.rs`)

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Typed representation of blog.yaml
/// Deserialized from services/config/blog.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogConfig {
    #[serde(default)]
    pub content_sources: Vec<ContentSource>,

    #[serde(default = "default_base_url")]
    pub base_url: String,

    #[serde(default)]
    pub enable_link_tracking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSource {
    pub source_id: String,
    pub category_id: String,
    pub path: PathBuf,
    #[serde(default)]
    pub allowed_content_types: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub override_existing: bool,
}

impl Default for BlogConfig {
    fn default() -> Self {
        Self {
            content_sources: vec![],
            base_url: default_base_url(),
            enable_link_tracking: true,
        }
    }
}

fn default_base_url() -> String {
    "http://localhost:3000".to_string()
}

fn default_true() -> bool {
    true
}
```

### 4. Content Metadata Struct (`extensions/blog/src/models/content.rs`)

```rust
use serde::{Deserialize, Serialize};

/// Typed representation of markdown frontmatter
/// Deserialized from ---yaml--- block in .md files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub published_at: String,
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(default)]
    pub keywords: String,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub links: Vec<ContentLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentLink {
    pub url: String,
    pub text: String,
    #[serde(default)]
    pub r#type: String,
}

fn default_kind() -> String {
    "article".to_string()
}
```

### 5. Ingestion Service (`extensions/blog/src/services/ingestion.rs`)

```rust
use std::path::Path;
use std::sync::Arc;
use sqlx::PgPool;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;
use systemprompt_identifiers::{CategoryId, SourceId};

use crate::config::ContentSource;
use crate::error::BlogError;
use crate::models::{ContentMetadata, CreateContentParams, IngestionReport};
use crate::repository::ContentRepository;

pub struct IngestionService {
    repo: ContentRepository,
}

impl IngestionService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { repo: ContentRepository::new(pool) }
    }

    /// Process a content source from YAML config
    pub async fn ingest_source(&self, source: &ContentSource) -> Result<IngestionReport, BlogError> {
        let mut report = IngestionReport::new();

        // Skip disabled sources (configured in YAML)
        if !source.enabled {
            return Ok(report);
        }

        // Validate path exists
        if !source.path.exists() {
            report.errors.push(format!("Path not found: {}", source.path.display()));
            return Ok(report);
        }

        // Walk directory for .md files
        for entry in WalkDir::new(&source.path)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
        {
            report.files_found += 1;

            match self.ingest_file(entry.path(), &source.source_id, &source.category_id).await {
                Ok(_) => report.files_processed += 1,
                Err(e) => report.errors.push(format!("{}: {}", entry.path().display(), e)),
            }
        }

        Ok(report)
    }

    async fn ingest_file(&self, path: &Path, source_id: &str, category_id: &str) -> Result<(), BlogError> {
        let content = std::fs::read_to_string(path)?;
        let version_hash = compute_hash(&content);

        // Parse YAML frontmatter from markdown
        let (metadata, body) = parse_markdown(&content)?;

        // Validate and transform
        let published_at = parse_datetime(&metadata.published_at)?;
        let links = serde_json::to_value(&metadata.links)?;

        // Build typed params
        let params = CreateContentParams::new(
            metadata.slug,
            metadata.title,
            metadata.description,
            body,
            metadata.author,
            published_at,
            SourceId::new(source_id.to_string()),
        )
        .with_version_hash(version_hash)
        .with_keywords(metadata.keywords)
        .with_category_id(Some(CategoryId::new(category_id.to_string())))
        .with_links(links);

        // Persist to database
        self.repo.create(&params).await?;
        Ok(())
    }
}

fn parse_markdown(content: &str) -> Result<(ContentMetadata, String), BlogError> {
    // Validate frontmatter exists
    if !content.starts_with("---") {
        return Err(BlogError::Parse("Missing YAML frontmatter".into()));
    }

    let rest = &content[3..];
    let end = rest.find("---")
        .ok_or_else(|| BlogError::Parse("Unclosed frontmatter".into()))?;

    let frontmatter = rest[..end].trim();
    let body = rest[end + 3..].trim().to_string();

    // Deserialize YAML to typed struct
    let metadata: ContentMetadata = serde_yaml::from_str(frontmatter)?;

    Ok((metadata, body))
}

fn parse_datetime(s: &str) -> Result<chrono::DateTime<chrono::Utc>, BlogError> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .or_else(|_| {
            chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
                .map(|d| d.and_hms_opt(0, 0, 0).unwrap())
                .map(|dt| chrono::DateTime::from_naive_utc_and_offset(dt, chrono::Utc))
        })
        .map_err(|_| BlogError::Parse(format!("Invalid date: {s}")))
}

fn compute_hash(content: &str) -> String {
    format!("{:x}", sha2::Sha256::digest(content.as_bytes()))
}
```

### 6. Background Job (`extensions/blog/src/jobs/ingestion.rs`)

```rust
use std::sync::Arc;
use sqlx::PgPool;
use systemprompt_traits::{Job, JobContext, JobResult};

use crate::config::BlogConfig;
use crate::services::IngestionService;

#[derive(Debug, Clone, Copy, Default)]
pub struct ContentIngestionJob;

#[async_trait::async_trait]
impl Job for ContentIngestionJob {
    fn name(&self) -> &'static str { "blog_content_ingestion" }
    fn description(&self) -> &'static str { "Ingests markdown from services/content/" }
    fn schedule(&self) -> &'static str { "0 0 * * * *" }  // Hourly default

    async fn execute(&self, ctx: &JobContext) -> anyhow::Result<JobResult> {
        let pool = ctx.db_pool::<PgPool>()
            .ok_or_else(|| anyhow::anyhow!("DB not available"))?;

        // Load YAML config from services/
        let config = load_config()?;

        let service = IngestionService::new(Arc::new(pool.clone()));

        let mut total = 0u64;
        let mut errors = 0u64;

        // Process each source defined in YAML
        for source in &config.content_sources {
            match service.ingest_source(source).await {
                Ok(report) => {
                    total += report.files_processed as u64;
                    errors += report.errors.len() as u64;
                }
                Err(e) => {
                    tracing::error!(source = %source.source_id, error = %e, "Ingestion failed");
                    errors += 1;
                }
            }
        }

        Ok(JobResult::success().with_stats(total, errors))
    }
}

fn load_config() -> anyhow::Result<BlogConfig> {
    let path = std::env::var("BLOG_CONFIG")
        .unwrap_or_else(|_| "./services/config/blog.yaml".to_string());
    let content = std::fs::read_to_string(&path)?;
    Ok(serde_yaml::from_str(&content)?)
}
```

### 7. Job Schedule Override (`services/scheduler/config.yml`)

```yaml
# Override job schedules without modifying Rust code
scheduler:
  jobs:
    - extension: blog
      job: blog_content_ingestion
      schedule: "0 */15 * * * *"  # Every 15 minutes instead of hourly
      enabled: true

    - extension: blog
      job: blog_content_ingestion
      schedule: "0 0 2 * * *"     # Or daily at 2 AM for production
      enabled: false              # Disabled - use the 15-min one
```

---

## Data Flow Summary

```
services/config/blog.yaml          services/content/blog/*.md
        │                                    │
        │ BlogConfig                         │ ContentMetadata
        ▼                                    ▼
┌───────────────────────────────────────────────────────────┐
│                   IngestionService                         │
│                                                            │
│  1. Load BlogConfig from YAML                              │
│  2. For each ContentSource:                                │
│     a. Walk path for .md files                             │
│     b. Parse frontmatter → ContentMetadata                 │
│     c. Validate (dates, required fields)                   │
│     d. Transform to CreateContentParams                    │
│     e. Persist via ContentRepository                       │
└────────────────────────┬──────────────────────────────────┘
                         │
                         ▼
                    PostgreSQL
                  (markdown_content)
```

---

## Key Patterns

| Pattern | Service (YAML) | Extension (Rust) |
|---------|----------------|------------------|
| Configuration | Declares sources, paths, options | Deserializes to typed structs |
| Content | Markdown with YAML frontmatter | Parses, validates, stores |
| Scheduling | Overrides job schedules | Defines job logic + default schedule |
| Validation | Schema implied by structure | Enforced at runtime with errors |
| Defaults | Not specified = use extension default | `#[serde(default)]` on struct fields |

---

## Rules

1. **Services never execute** — They only declare intent
2. **Extensions validate everything** — Don't trust YAML blindly
3. **Type your config** — `serde` structs, not `serde_json::Value`
4. **Defaults in Rust** — Use `#[serde(default)]`, not YAML anchors
5. **Paths are relative to project root** — `./services/content/blog`
6. **Jobs define defaults, YAML overrides** — Extension: `"0 0 * * * *"`, Service: `"0 */15 * * * *"`
