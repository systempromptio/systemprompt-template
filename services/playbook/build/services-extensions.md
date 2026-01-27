---
title: "Services & Extensions Relationship Playbook"
description: "How services and extensions interact in systemprompt-core."
keywords:
  - services
  - extensions
  - config
  - type-state
---

# Services & Extensions Relationship

**Services declare content. Extensions implement logic. Profiles configure both.**

> **Help**: `{ "command": "playbook build" }` via `systemprompt_help`

| Layer | Location | Format | Purpose |
|-------|----------|--------|---------|
| Profiles | `/profiles/` | YAML | Extension config, paths, secrets |
| Services | `/services/` | Markdown, YAML | Content, job schedules |
| Extensions | `/extensions/` | Rust | Validation, processing, storage |

```
┌─────────────────────────────────────────────────────────────┐
│                   profiles/*.profile.yml                     │
│                                                              │
│   paths.services ──► where content lives                    │
│   extensions.blog ─► extension-specific config              │
│                      (validated at STARTUP)                 │
└──────────────────────────┬──────────────────────────────────┘
                           │ validated by StartupValidator
┌──────────────────────────▼──────────────────────────────────┐
│                    extensions/ (Rust)                        │
│                                                              │
│   config.rs ──────► Raw → Validated type-state pattern      │
│   services/*.rs ──► business logic with validated config    │
│   repository/*.rs ► persist to database                     │
│   jobs/*.rs ──────► receive validated config directly       │
└──────────────────────────┬──────────────────────────────────┘
                           │ processes
┌──────────────────────────▼──────────────────────────────────┐
│                      services/ (Content)                     │
│                                                              │
│   content/*.md ───► markdown with YAML frontmatter          │
│   scheduler/*.yml ► job schedule overrides                  │
└─────────────────────────────────────────────────────────────┘
```

---

## Complete Example: Blog Content System

### 1. Profile Configuration (`profiles/local.secrets.profile.yml`)

```yaml
# Extension config lives in PROFILE, not services/
# Validated at STARTUP - missing paths = app won't start

name: local
display_name: "Local Development"

paths:
  services: /var/www/services
  # ... other paths ...

extensions:
  blog:
    base_url: https://myblog.com
    enable_link_tracking: true
    content_sources:
      - source_id: blog
        category_id: blog
        path: content/blog          # Relative to paths.services
        enabled: true
      - source_id: docs
        category_id: documentation
        path: content/docs
        allowed_content_types:
          - documentation
          - guide
        enabled: true
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

### 3. Extension Config (Type-State Pattern) (`extensions/blog/src/config.rs`)

```rust
use serde::Deserialize;
use std::path::{Path, PathBuf};
use systemprompt::extension::typed::{ExtensionConfig, ExtensionConfigErrors};
use systemprompt::identifiers::{CategoryId, SourceId};
use url::Url;

use crate::BlogExtension;

// RAW - Deserialized from profile, unvalidated (String for paths/URLs)

#[derive(Debug, Deserialize)]
pub struct BlogConfigRaw {
    #[serde(default)]
    pub content_sources: Vec<ContentSourceRaw>,
    #[serde(default = "default_base_url")]
    pub base_url: String,
    #[serde(default)]
    pub enable_link_tracking: bool,
}

#[derive(Debug, Deserialize)]
pub struct ContentSourceRaw {
    pub source_id: String,
    pub category_id: String,
    pub path: String,              // String, not PathBuf
    #[serde(default)]
    pub allowed_content_types: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_base_url() -> String { "https://example.com".into() }
fn default_true() -> bool { true }

// VALIDATED - Paths verified, URLs parsed (cannot be invalid)

#[derive(Debug, Clone)]
pub struct BlogConfigValidated {
    content_sources: Vec<ContentSourceValidated>,
    base_url: Url,                 // Parsed Url, not String
    enable_link_tracking: bool,
}

#[derive(Debug, Clone)]
pub struct ContentSourceValidated {
    source_id: SourceId,           // Typed ID, not String
    category_id: CategoryId,       // Typed ID, not String
    path: PathBuf,                 // Canonicalized, verified to exist
    allowed_content_types: Vec<String>,
    enabled: bool,
}

impl BlogConfigValidated {
    pub fn enabled_sources(&self) -> impl Iterator<Item = &ContentSourceValidated> {
        self.content_sources.iter().filter(|s| s.enabled)
    }
    pub fn base_url(&self) -> &Url { &self.base_url }
}

impl ContentSourceValidated {
    pub fn source_id(&self) -> &SourceId { &self.source_id }
    pub fn category_id(&self) -> &CategoryId { &self.category_id }
    pub fn path(&self) -> &Path { &self.path }
}

// VALIDATION - Transform Raw → Validated (consumes Raw)

impl ExtensionConfig for BlogExtension {
    type Raw = BlogConfigRaw;
    type Validated = BlogConfigValidated;
    const PREFIX: &'static str = "blog";

    fn validate(raw: Self::Raw, base_path: &Path) -> Result<Self::Validated, ExtensionConfigErrors> {
        let mut errors = ExtensionConfigErrors::new(Self::PREFIX);

        // Parse and validate URL
        let base_url = Url::parse(&raw.base_url)
            .map_err(|e| errors.push("base_url", e.to_string()))
            .unwrap_or_else(|_| Url::parse("https://invalid").unwrap());

        // Validate and transform each source
        let mut sources = Vec::new();
        for (i, src) in raw.content_sources.into_iter().enumerate() {
            let path = base_path.join(&src.path);

            if src.enabled && !path.exists() {
                errors.push_with_path(
                    format!("content_sources[{}].path", i),
                    format!("Path does not exist for source '{}'", src.source_id),
                    &path,
                );
                continue;
            }

            sources.push(ContentSourceValidated {
                source_id: SourceId::new(src.source_id),
                category_id: CategoryId::new(src.category_id),
                path: path.canonicalize().unwrap_or(path),
                allowed_content_types: src.allowed_content_types,
                enabled: src.enabled,
            });
        }

        errors.into_result(BlogConfigValidated {
            content_sources: sources,
            base_url,
            enable_link_tracking: raw.enable_link_tracking,
        })
    }
}
```

### 4. Background Job (`extensions/blog/src/jobs/ingestion.rs`)

```rust
use std::sync::Arc;
use sqlx::PgPool;
use systemprompt::traits::{Job, JobContext, JobResult};

use crate::BlogExtension;
use crate::config::BlogConfigValidated;
use crate::services::IngestionService;

#[derive(Debug, Clone, Copy, Default)]
pub struct ContentIngestionJob;

#[async_trait::async_trait]
impl Job for ContentIngestionJob {
    fn name(&self) -> &'static str { "blog_content_ingestion" }
    fn description(&self) -> &'static str { "Ingests markdown from content sources" }
    fn schedule(&self) -> &'static str { "0 0 * * * *" }

    async fn execute(&self, ctx: &JobContext) -> anyhow::Result<JobResult> {
        let pool = ctx.db_pool::<PgPool>()
            .ok_or_else(|| anyhow::anyhow!("DB not available"))?;

        // Get ALREADY VALIDATED config - no parsing, no file loading!
        // Paths are GUARANTEED to exist (validated at startup)
        let config: Arc<BlogConfigValidated> = ctx
            .extension_config::<BlogExtension>()
            .ok_or_else(|| anyhow::anyhow!(
                "Blog config not found. Add 'extensions.blog' to profile."
            ))?;

        let service = IngestionService::new(Arc::new(pool.clone()));

        let mut total = 0u64;
        let mut errors = 0u64;

        // Process only enabled sources (filtered by validated config)
        for source in config.enabled_sources() {
            match service.ingest_source(source).await {
                Ok(report) => {
                    total += report.files_processed as u64;
                    errors += report.errors.len() as u64;
                }
                Err(e) => {
                    tracing::error!(
                        source_id = %source.source_id(),
                        error = %e,
                        "Ingestion failed"
                    );
                    errors += 1;
                }
            }
        }

        Ok(JobResult::success().with_stats(total, errors))
    }
}

// NO load_config() function needed!
// Config is validated at startup and retrieved from JobContext
```

### 5. Job Schedule Override (`services/scheduler/config.yml`)

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
┌─────────────────────────────────────────────────────────────┐
│                      STARTUP                                 │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  profiles/*.profile.yml                                      │
│         │                                                    │
│         │ extensions.blog                                    │
│         ▼                                                    │
│  BlogConfigRaw (deserialized)                                │
│         │                                                    │
│         │ ExtensionConfig::validate()                        │
│         ▼                                                    │
│  BlogConfigValidated (paths verified, URLs parsed)           │
│         │                                                    │
│         │ stored in ExtensionConfigRegistry                  │
│         ▼                                                    │
│  If errors → display report → exit(1)                       │
│                                                              │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                      RUNTIME (Job)                           │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ctx.extension_config::<BlogExtension>()                    │
│         │                                                    │
│         │ Arc<BlogConfigValidated>                          │
│         ▼                                                    │
│  IngestionService.ingest_source()                           │
│         │                                                    │
│         │ paths GUARANTEED to exist                         │
│         ▼                                                    │
│  services/content/*.md → parse → ContentRepository          │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Key Patterns

| Pattern | Profile (YAML) | Extension (Rust) | Services (Content) |
|---------|----------------|------------------|-------------------|
| Configuration | `extensions.{name}` section | Type-state: Raw -> Validated | N/A |
| Path validation | String paths | `PathBuf` (canonicalized, verified) | N/A |
| URL validation | String URLs | `Url` (parsed, scheme checked) | N/A |
| ID types | String IDs | `SourceId`, `CategoryId` (typed) | N/A |
| Content | N/A | Parses, validates, stores | Markdown + frontmatter |
| Scheduling | N/A | Defines default schedule | Override schedules |
| Validation timing | N/A | **Startup** (not runtime) | N/A |

---

## Rules

1. **Config lives in profiles** -- Not `services/config/`, not env vars
2. **Type-state for config** -- `Raw` -> `Validated` transformation
3. **Validate at startup** -- Missing paths = app won't start
4. **Jobs receive validated config** -- No file loading, no parsing
5. **Paths are guaranteed** -- Once validated, paths exist
6. **No fallbacks** -- Invalid config = error, not silent default
7. **Rich types** -- `PathBuf`, `Url`, typed IDs (not `String`)
8. **Collect all errors** -- Don't stop at first failure

---

## Quick Reference

| Task | Command |
|------|---------|
| View profile | `cat profiles/local.profile.yml` |
| View job config | `cat services/scheduler/config.yml` |
| List content | `ls services/content/` |
| Run job | `systemprompt jobs run blog_content_ingestion` |
