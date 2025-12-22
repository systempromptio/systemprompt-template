# Blog Extension Validation Integration

## Overview

Connect the blog extension to Core's startup validation system using idiomatic Rust patterns:
- **Associated types** for type-safe config handling
- **Type-state pattern**: `Raw` → `Validated` transformation
- **Parse, don't validate**: Transform to richer types, don't check and discard
- **Store validated configs**: No re-parsing at runtime

## Current State

The blog extension currently:
- Loads config from `BLOG_CONFIG` env var or `./services/config/blog.yaml`
- Validates content at **ingestion time** (job execution), not startup
- Uses `String` for paths, URLs, IDs (stringly-typed)
- Does NOT implement any config extension trait
- Is NOT validated by `StartupValidator`

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    STARTUP SEQUENCE                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Load profile YAML                                        │
│     └── extensions.blog → BlogConfigRaw (just deserialized) │
│                                                              │
│  2. ExtensionConfig::validate(raw, base_path)               │
│     └── Consumes BlogConfigRaw                              │
│     └── Validates paths exist, URLs parse, IDs valid        │
│     └── Produces BlogConfigValidated (richer types)         │
│                                                              │
│  3. Store in ExtensionConfigRegistry                         │
│     └── registry.configs["blog"] = BlogConfigValidated      │
│                                                              │
│  4. At runtime, jobs receive &BlogConfigValidated            │
│     └── No parsing, no fallibility, paths guaranteed valid  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Phase 1: Core Trait Definition

**File:** `crates/shared/extension/src/typed/config.rs`

Replace existing `ConfigExtensionTyped` with associated-type version:

```rust
use serde::de::DeserializeOwned;
use std::any::Any;
use std::path::Path;

/// Extension configuration trait using type-state pattern.
///
/// Extensions implement this to define their config types and validation logic.
/// The `validate` method consumes `Raw` and produces `Validated`, ensuring
/// invalid configs cannot exist at runtime.
///
/// # Type-State
///
/// ```text
/// Profile YAML → Raw (deserialized, unvalidated)
///                 ↓ validate() consumes self
///             Validated (paths verified, URLs parsed, IDs typed)
/// ```
pub trait ExtensionConfig: Send + Sync + 'static {
    /// Raw config type - deserializable from YAML/JSON, unvalidated.
    /// Uses `String` for paths, URLs, IDs.
    type Raw: DeserializeOwned + Send;

    /// Validated config type - paths are `PathBuf` (canonicalized),
    /// URLs are `Url`, IDs are typed. Guaranteed valid.
    type Validated: Clone + Send + Sync + Any;

    /// Extension identifier used as key in profile `extensions` section.
    const PREFIX: &'static str;

    /// Transform raw config into validated config.
    ///
    /// This method:
    /// - Consumes the raw config (move semantics)
    /// - Validates all paths exist on disk
    /// - Parses URLs, typed IDs
    /// - Collects ALL errors (not just first)
    /// - Returns validated config or errors
    ///
    /// # Arguments
    ///
    /// * `raw` - The deserialized but unvalidated config
    /// * `base_path` - Base path for resolving relative paths
    fn validate(raw: Self::Raw, base_path: &Path) -> Result<Self::Validated, ExtensionConfigErrors>;
}

/// Collection of validation errors for an extension.
#[derive(Debug, Default)]
pub struct ExtensionConfigErrors {
    pub extension: &'static str,
    pub errors: Vec<ExtensionConfigError>,
}

#[derive(Debug)]
pub struct ExtensionConfigError {
    pub field: String,
    pub message: String,
    pub path: Option<std::path::PathBuf>,
    pub suggestion: Option<String>,
}

impl ExtensionConfigErrors {
    pub fn new(extension: &'static str) -> Self {
        Self {
            extension,
            errors: Vec::new(),
        }
    }

    pub fn push(&mut self, field: impl Into<String>, message: impl Into<String>) {
        self.errors.push(ExtensionConfigError {
            field: field.into(),
            message: message.into(),
            path: None,
            suggestion: None,
        });
    }

    pub fn push_with_path(
        &mut self,
        field: impl Into<String>,
        message: impl Into<String>,
        path: impl Into<std::path::PathBuf>,
    ) {
        self.errors.push(ExtensionConfigError {
            field: field.into(),
            message: message.into(),
            path: Some(path.into()),
            suggestion: None,
        });
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Convert to Result - Ok if no errors, Err if any errors.
    pub fn into_result<T>(self, value: T) -> Result<T, Self> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            Err(self)
        }
    }
}
```

---

## Phase 2: Extension Config Registry

**File:** `crates/shared/extension/src/registry.rs`

Add typed config storage:

```rust
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

/// Registry storing validated extension configs.
///
/// Configs are validated once at startup and stored here.
/// Runtime code retrieves them by extension type - no re-parsing.
pub struct ExtensionConfigRegistry {
    /// Validated configs keyed by extension PREFIX.
    configs: HashMap<&'static str, Arc<dyn Any + Send + Sync>>,
}

impl ExtensionConfigRegistry {
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
        }
    }

    /// Store a validated config.
    pub fn insert<E: ExtensionConfig>(&mut self, config: E::Validated) {
        self.configs.insert(E::PREFIX, Arc::new(config));
    }

    /// Get a validated config by extension type.
    ///
    /// Returns `None` if the extension has no config in the profile.
    /// Panics if the config type doesn't match (programmer error).
    pub fn get<E: ExtensionConfig>(&self) -> Option<Arc<E::Validated>> {
        self.configs.get(E::PREFIX).map(|c| {
            c.clone()
                .downcast::<E::Validated>()
                .expect("ExtensionConfig type mismatch - this is a bug")
        })
    }

    /// Check if an extension has config registered.
    pub fn contains(&self, prefix: &str) -> bool {
        self.configs.contains_key(prefix)
    }
}

impl Default for ExtensionConfigRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Phase 3: Blog Extension Types

**File:** `/var/www/html/systemprompt-template/extensions/blog/src/config.rs`

Replace with type-state types:

```rust
use serde::Deserialize;
use std::path::{Path, PathBuf};
use systemprompt::extension::typed::{ExtensionConfig, ExtensionConfigErrors};
use systemprompt::identifiers::{CategoryId, SourceId};
use url::Url;

use crate::BlogExtension;

// ============================================================================
// RAW CONFIG - Deserialized from YAML, unvalidated
// ============================================================================

/// Raw blog config - just deserialized, paths are strings.
#[derive(Debug, Deserialize)]
pub struct BlogConfigRaw {
    #[serde(default)]
    pub content_sources: Vec<ContentSourceRaw>,

    #[serde(default = "default_base_url")]
    pub base_url: String,

    #[serde(default = "default_true")]
    pub enable_link_tracking: bool,
}

#[derive(Debug, Deserialize)]
pub struct ContentSourceRaw {
    pub source_id: String,
    pub category_id: String,
    pub path: String,
    #[serde(default)]
    pub allowed_content_types: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub override_existing: bool,
}

fn default_base_url() -> String {
    "https://example.com".to_string()
}

fn default_true() -> bool {
    true
}

// ============================================================================
// VALIDATED CONFIG - Paths verified, URLs parsed, IDs typed
// ============================================================================

/// Validated blog config - paths are PathBuf, URL is parsed, IDs are typed.
///
/// This type can only be constructed via `ExtensionConfig::validate()`,
/// guaranteeing all paths exist and URLs are valid.
#[derive(Debug, Clone)]
pub struct BlogConfigValidated {
    content_sources: Vec<ContentSourceValidated>,
    base_url: Url,
    enable_link_tracking: bool,
}

#[derive(Debug, Clone)]
pub struct ContentSourceValidated {
    source_id: SourceId,
    category_id: CategoryId,
    path: PathBuf,  // Canonicalized, verified to exist
    allowed_content_types: Vec<String>,
    enabled: bool,
    override_existing: bool,
}

impl BlogConfigValidated {
    /// Get content sources (only enabled ones).
    pub fn enabled_sources(&self) -> impl Iterator<Item = &ContentSourceValidated> {
        self.content_sources.iter().filter(|s| s.enabled)
    }

    /// Get all content sources.
    pub fn all_sources(&self) -> &[ContentSourceValidated] {
        &self.content_sources
    }

    /// Get base URL.
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    /// Check if link tracking is enabled.
    pub fn link_tracking_enabled(&self) -> bool {
        self.enable_link_tracking
    }
}

impl ContentSourceValidated {
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    pub fn category_id(&self) -> &CategoryId {
        &self.category_id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn allowed_content_types(&self) -> &[String] {
        &self.allowed_content_types
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn override_existing(&self) -> bool {
        self.override_existing
    }
}

// ============================================================================
// VALIDATION - Transform Raw → Validated
// ============================================================================

impl ExtensionConfig for BlogExtension {
    type Raw = BlogConfigRaw;
    type Validated = BlogConfigValidated;

    const PREFIX: &'static str = "blog";

    fn validate(raw: Self::Raw, base_path: &Path) -> Result<Self::Validated, ExtensionConfigErrors> {
        let mut errors = ExtensionConfigErrors::new(Self::PREFIX);

        // Validate and parse base_url
        let base_url = match Url::parse(&raw.base_url) {
            Ok(url) => {
                if url.scheme() != "http" && url.scheme() != "https" {
                    errors.push("base_url", "URL must use http or https scheme");
                }
                url
            }
            Err(e) => {
                errors.push("base_url", format!("Invalid URL: {}", e));
                // Use placeholder to continue collecting errors
                Url::parse("https://invalid.example.com").unwrap()
            }
        };

        // Validate and transform content sources
        let mut content_sources = Vec::with_capacity(raw.content_sources.len());

        for (i, src) in raw.content_sources.into_iter().enumerate() {
            let field_prefix = format!("content_sources[{}]", i);

            // Validate source_id
            if src.source_id.trim().is_empty() {
                errors.push(format!("{}.source_id", field_prefix), "source_id cannot be empty");
                continue;
            }

            // Validate category_id
            if src.category_id.trim().is_empty() {
                errors.push(
                    format!("{}.category_id", field_prefix),
                    "category_id cannot be empty",
                );
                continue;
            }

            // Resolve and validate path
            let resolved_path = if Path::new(&src.path).is_absolute() {
                PathBuf::from(&src.path)
            } else {
                base_path.join(&src.path)
            };

            // Only validate path exists if source is enabled
            if src.enabled {
                if !resolved_path.exists() {
                    errors.push_with_path(
                        format!("{}.path", field_prefix),
                        format!(
                            "Content source '{}' path does not exist",
                            src.source_id
                        ),
                        &resolved_path,
                    );
                    continue;
                }

                if !resolved_path.is_dir() {
                    errors.push_with_path(
                        format!("{}.path", field_prefix),
                        format!(
                            "Content source '{}' path is not a directory",
                            src.source_id
                        ),
                        &resolved_path,
                    );
                    continue;
                }
            }

            // Canonicalize path if it exists
            let canonical_path = if resolved_path.exists() {
                resolved_path.canonicalize().unwrap_or(resolved_path)
            } else {
                resolved_path
            };

            content_sources.push(ContentSourceValidated {
                source_id: SourceId::new(src.source_id),
                category_id: CategoryId::new(src.category_id),
                path: canonical_path,
                allowed_content_types: src.allowed_content_types,
                enabled: src.enabled,
                override_existing: src.override_existing,
            });
        }

        errors.into_result(BlogConfigValidated {
            content_sources,
            base_url,
            enable_link_tracking: raw.enable_link_tracking,
        })
    }
}
```

---

## Phase 4: Registration Macro

**File:** `crates/shared/extension/src/lib.rs`

Add compile-time registration for config extensions:

```rust
/// Register an extension's config for startup validation.
///
/// This macro registers the extension with the config validation system.
/// At startup, Core will:
/// 1. Look for `extensions.{PREFIX}` in the profile
/// 2. Deserialize to `E::Raw`
/// 3. Call `E::validate()` to produce `E::Validated`
/// 4. Store in `ExtensionConfigRegistry`
#[macro_export]
macro_rules! register_config_extension {
    ($ext_type:ty) => {
        $crate::inventory::submit! {
            $crate::ConfigExtensionRegistration {
                prefix: <$ext_type as $crate::typed::ExtensionConfig>::PREFIX,
                validate: |json, base_path| {
                    // Deserialize Raw
                    let raw: <$ext_type as $crate::typed::ExtensionConfig>::Raw =
                        serde_json::from_value(json)
                            .map_err(|e| {
                                let mut errors = $crate::typed::ExtensionConfigErrors::new(
                                    <$ext_type as $crate::typed::ExtensionConfig>::PREFIX
                                );
                                errors.push("_parse", e.to_string());
                                errors
                            })?;

                    // Validate Raw → Validated
                    let validated = <$ext_type as $crate::typed::ExtensionConfig>::validate(
                        raw,
                        base_path,
                    )?;

                    // Box for type erasure
                    Ok(Box::new(validated) as Box<dyn std::any::Any + Send + Sync>)
                },
            }
        }
    };
}

/// Registration entry for a config extension.
pub struct ConfigExtensionRegistration {
    pub prefix: &'static str,
    pub validate: fn(
        serde_json::Value,
        &std::path::Path,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>, typed::ExtensionConfigErrors>,
}

inventory::collect!(ConfigExtensionRegistration);
```

---

## Phase 5: Update Blog Extension Registration

**File:** `/var/www/html/systemprompt-template/extensions/blog/src/extension.rs`

Add config registration:

```rust
use systemprompt::extension::prelude::*;

// ... existing code ...

// Register extension and its config
register_extension!(BlogExtension);
register_config_extension!(BlogExtension);  // NEW
```

---

## Phase 6: StartupValidator Integration

**File:** `crates/app/runtime/src/startup_validation.rs`

Update extension validation phase:

```rust
use systemprompt_extension::{ConfigExtensionRegistration, ExtensionConfigRegistry};

impl StartupValidator {
    pub async fn validate(&mut self, config: &Config) -> StartupValidationReport {
        // ... Phase 1-2: Domain validation ...

        // Phase 3: Extension config validation
        CliService::info("Validating extension configurations...");

        let mut config_registry = ExtensionConfigRegistry::new();
        let base_path = config.services_path();

        for reg in inventory::iter::<ConfigExtensionRegistration>() {
            let prefix = reg.prefix;

            // Get extension's config from profile
            let ext_config = match self.get_extension_config(config, prefix) {
                Some(json) => json,
                None => {
                    // Extension has no config in profile - skip
                    CliService::debug(&format!("  [ext:{}] No config in profile", prefix));
                    continue;
                }
            };

            // Validate: Raw → Validated
            match (reg.validate)(ext_config, base_path) {
                Ok(validated) => {
                    CliService::success(&format!("  [ext:{}] Valid", prefix));
                    // Store validated config (type-erased)
                    config_registry.insert_boxed(prefix, validated);
                }
                Err(errors) => {
                    CliService::error(&format!(
                        "  [ext:{}] {} error(s)",
                        prefix,
                        errors.errors.len()
                    ));

                    let mut ext_report = ValidationReport::new(format!("ext:{}", prefix));
                    for error in errors.errors {
                        ext_report.add_error(ValidationError::new(error.field, error.message));
                    }
                    report.add_extension(ext_report);
                }
            }
        }

        // Store registry for runtime access
        self.extension_configs = Some(config_registry);

        report
    }
}
```

---

## Phase 7: Update ContentIngestionJob

**File:** `/var/www/html/systemprompt-template/extensions/blog/src/jobs/ingestion.rs`

Use validated config directly:

```rust
use crate::config::BlogConfigValidated;
use systemprompt::extension::ExtensionConfigRegistry;

#[async_trait::async_trait]
impl Job for ContentIngestionJob {
    async fn execute(&self, ctx: &JobContext) -> Result<JobResult> {
        // Get ALREADY VALIDATED config - no parsing, guaranteed valid
        let config: Arc<BlogConfigValidated> = ctx
            .extension_config::<BlogExtension>()
            .ok_or_else(|| anyhow::anyhow!(
                "Blog extension config not found. Add 'extensions.blog' to profile."
            ))?;

        let start = std::time::Instant::now();
        tracing::info!("Blog content ingestion started");

        let pool = ctx.db_pool::<PgPool>()?;
        let ingestion_service = IngestionService::new(Arc::new(pool.clone()));

        let mut total_processed = 0u64;
        let mut total_errors = 0u64;

        // Iterate enabled sources - paths are GUARANTEED to exist
        for source in config.enabled_sources() {
            tracing::debug!(
                source_id = %source.source_id(),
                path = %source.path().display(),
                "Ingesting source"
            );

            match ingestion_service.ingest_path(
                source.path(),
                source.source_id(),
                source.category_id(),
            ).await {
                Ok(report) => {
                    total_processed += report.files_processed as u64;
                    total_errors += report.errors.len() as u64;
                }
                Err(e) => {
                    tracing::error!(
                        source_id = %source.source_id(),
                        error = %e,
                        "Source ingestion failed"
                    );
                    total_errors += 1;
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        tracing::info!(
            files_processed = total_processed,
            errors = total_errors,
            duration_ms = duration_ms,
            "Blog content ingestion completed"
        );

        Ok(JobResult::success()
            .with_stats(total_processed, total_errors)
            .with_duration(duration_ms))
    }
}
```

---

## Phase 8: Update IngestionService

**File:** `/var/www/html/systemprompt-template/extensions/blog/src/services/ingestion.rs`

Use typed IDs:

```rust
use systemprompt::identifiers::{CategoryId, SourceId};

impl IngestionService {
    /// Ingest content from a validated path.
    ///
    /// Path is guaranteed to exist (validated at startup).
    pub async fn ingest_path(
        &self,
        path: &Path,
        source_id: &SourceId,
        category_id: &CategoryId,
    ) -> Result<IngestionReport, BlogError> {
        let mut report = IngestionReport::new();

        // Path guaranteed to exist - no existence check needed
        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_type().is_file()
                    && e.path().extension().map(|ext| ext == "md").unwrap_or(false)
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
}
```

---

## Profile Format

```yaml
# local.secrets.profile.yml

name: local
display_name: "Local Development"

paths:
  system: /var/www/html/systemprompt-core
  services: /var/www/services
  # ... other paths ...

# Extension configurations - validated at startup
extensions:
  blog:
    base_url: https://example.com
    enable_link_tracking: true
    content_sources:
      - source_id: blog
        category_id: blog
        path: content/blog          # Relative to services path
        enabled: true
      - source_id: guides
        category_id: guides
        path: content/guides
        enabled: true
        allowed_content_types:
          - tutorial
          - guide
```

---

## Error Output Example

```
Startup Validation Failed

Profile: /var/www/services/profiles/local.secrets.profile.yml

ERRORS:

[ext:blog] content_sources[0].path
  Content source 'blog' path does not exist
  Path: /var/www/services/content/blog
  To fix: Create the directory or set enabled: false

[ext:blog] base_url
  Invalid URL: relative URL without a base
  To fix: Use absolute URL like https://example.com
```

---

## Type-State Summary

```
┌─────────────────────────────────────────────────────────────┐
│                     TYPE-STATE CHAIN                         │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  BlogConfigRaw                                               │
│  ├── content_sources: Vec<ContentSourceRaw>                 │
│  │   └── path: String          ← Unvalidated string         │
│  │   └── source_id: String     ← Unvalidated string         │
│  ├── base_url: String          ← Unvalidated string         │
│  └── #[derive(Deserialize)]    ← Can parse from YAML        │
│                                                              │
│              │                                               │
│              │ ExtensionConfig::validate()                   │
│              │ (consumes Raw, produces Validated)            │
│              ▼                                               │
│                                                              │
│  BlogConfigValidated                                         │
│  ├── content_sources: Vec<ContentSourceValidated>           │
│  │   └── path: PathBuf         ← Canonicalized, exists      │
│  │   └── source_id: SourceId   ← Typed ID                   │
│  ├── base_url: Url             ← Parsed, valid scheme       │
│  └── NO Deserialize            ← Can only come from validate│
│                                                              │
│  IMPOSSIBLE to have BlogConfigValidated with invalid paths  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## Files to Modify

| Location | File | Changes |
|----------|------|---------|
| Core | `crates/shared/extension/src/typed/config.rs` | New `ExtensionConfig` trait with associated types |
| Core | `crates/shared/extension/src/registry.rs` | Add `ExtensionConfigRegistry` |
| Core | `crates/shared/extension/src/lib.rs` | Add `register_config_extension!` macro |
| Core | `crates/app/runtime/src/startup_validation.rs` | Integrate extension config validation |
| Template | `extensions/blog/src/config.rs` | Replace with Raw/Validated types |
| Template | `extensions/blog/src/extension.rs` | Add `register_config_extension!` |
| Template | `extensions/blog/src/jobs/ingestion.rs` | Use validated config |
| Template | `extensions/blog/src/services/ingestion.rs` | Use typed IDs and paths |

---

## Acceptance Criteria

- [ ] `ExtensionConfig` trait uses associated types (`Raw`, `Validated`)
- [ ] `validate()` consumes `Raw` and produces `Validated` (move semantics)
- [ ] `BlogConfigValidated` has `PathBuf` (not `String`) for paths
- [ ] `BlogConfigValidated` has `Url` (not `String`) for base_url
- [ ] `BlogConfigValidated` has `SourceId`/`CategoryId` (not `String`)
- [ ] Validated configs stored in `ExtensionConfigRegistry`
- [ ] Jobs receive `&BlogConfigValidated` - no re-parsing
- [ ] Invalid paths cause startup failure
- [ ] All errors collected (not just first)
- [ ] Cannot construct `BlogConfigValidated` except via `validate()`
