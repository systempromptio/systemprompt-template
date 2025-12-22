//! Blog extension configuration with type-state validation.
//!
//! This module implements the "parse, don't validate" pattern:
//! - `BlogConfigRaw`: Deserialized from YAML, unvalidated strings
//! - `BlogConfigValidated`: Paths verified, URLs parsed, IDs typed
//!
//! The validated config can only be constructed via `validate()`,
//! guaranteeing all paths exist and URLs are valid at runtime.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::Deserialize;
use systemprompt::identifiers::{CategoryId, SourceId};
use url::Url;

// ============================================================================
// EXTENSION CONFIG ERRORS - Defined locally until Core provides this
// ============================================================================

/// Collection of validation errors for an extension.
#[derive(Debug, Default)]
pub struct ExtensionConfigErrors {
    pub extension: &'static str,
    pub errors: Vec<ExtensionConfigError>,
}

/// Single validation error with context.
#[derive(Debug)]
pub struct ExtensionConfigError {
    pub field: String,
    pub message: String,
    pub path: Option<PathBuf>,
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
        path: impl Into<PathBuf>,
    ) {
        self.errors.push(ExtensionConfigError {
            field: field.into(),
            message: message.into(),
            path: Some(path.into()),
            suggestion: None,
        });
    }

    pub fn push_with_suggestion(
        &mut self,
        field: impl Into<String>,
        message: impl Into<String>,
        suggestion: impl Into<String>,
    ) {
        self.errors.push(ExtensionConfigError {
            field: field.into(),
            message: message.into(),
            path: None,
            suggestion: Some(suggestion.into()),
        });
    }

    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn into_result<T>(self, value: T) -> Result<T, Self> {
        if self.errors.is_empty() {
            Ok(value)
        } else {
            Err(self)
        }
    }
}

impl std::fmt::Display for ExtensionConfigErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Extension '{}' configuration errors:", self.extension)?;
        for error in &self.errors {
            write!(f, "  [{}] {}", error.field, error.message)?;
            if let Some(path) = &error.path {
                write!(f, "\n    Path: {}", path.display())?;
            }
            if let Some(suggestion) = &error.suggestion {
                write!(f, "\n    Fix: {}", suggestion)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

impl std::error::Error for ExtensionConfigErrors {}

// ============================================================================
// RAW CONFIG - Deserialized from YAML, unvalidated
// ============================================================================

/// Raw blog config - just deserialized, paths are strings.
///
/// This is the intermediate form between YAML and validated config.
/// All fields use basic types (String) that can fail validation.
#[derive(Debug, Clone, Deserialize)]
pub struct BlogConfigRaw {
    #[serde(default)]
    pub content_sources: Vec<ContentSourceRaw>,

    #[serde(default = "default_base_url")]
    pub base_url: String,

    #[serde(default = "default_true")]
    pub enable_link_tracking: bool,
}

/// Raw content source - unvalidated strings.
#[derive(Debug, Clone, Deserialize)]
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

impl Default for BlogConfigRaw {
    fn default() -> Self {
        Self {
            content_sources: Vec::new(),
            base_url: default_base_url(),
            enable_link_tracking: true,
        }
    }
}

// ============================================================================
// VALIDATED CONFIG - Paths verified, URLs parsed, IDs typed
// ============================================================================

/// Validated blog config - paths are `PathBuf`, URL is `Url`, IDs are typed.
///
/// This type can only be constructed via `BlogConfigValidated::validate()`,
/// guaranteeing all paths exist and URLs are valid.
///
/// # Invariants
///
/// - All enabled content source paths exist and are directories
/// - `base_url` is a valid HTTP/HTTPS URL
/// - All source_id and category_id values are non-empty
#[derive(Debug, Clone)]
pub struct BlogConfigValidated {
    content_sources: Vec<ContentSourceValidated>,
    base_url: Url,
    enable_link_tracking: bool,
}

/// Validated content source - paths canonicalized, IDs typed.
#[derive(Debug, Clone)]
pub struct ContentSourceValidated {
    source_id: SourceId,
    category_id: CategoryId,
    path: PathBuf,
    allowed_content_types: Vec<String>,
    enabled: bool,
    override_existing: bool,
}

impl BlogConfigValidated {
    /// Validate raw config and produce validated config.
    ///
    /// This consumes the raw config (move semantics) and:
    /// - Validates all paths exist on disk (for enabled sources)
    /// - Parses and validates the base URL
    /// - Converts string IDs to typed IDs
    /// - Collects ALL errors (not just first)
    ///
    /// # Arguments
    ///
    /// * `raw` - The deserialized but unvalidated config
    /// * `base_path` - Base path for resolving relative paths
    pub fn validate(raw: BlogConfigRaw, base_path: &Path) -> Result<Self, ExtensionConfigErrors> {
        let mut errors = ExtensionConfigErrors::new("blog");

        let base_url = match Url::parse(&raw.base_url) {
            Ok(url) => {
                if url.scheme() != "http" && url.scheme() != "https" {
                    errors.push_with_suggestion(
                        "base_url",
                        format!("URL must use http or https scheme, got: {}", url.scheme()),
                        "Use a URL like https://example.com",
                    );
                }
                url
            }
            Err(e) => {
                errors.push_with_suggestion(
                    "base_url",
                    format!("Invalid URL: {}", e),
                    "Use a valid URL like https://example.com",
                );
                Url::parse("https://invalid.example.com").expect("placeholder URL")
            }
        };

        let mut content_sources = Vec::with_capacity(raw.content_sources.len());

        for (i, src) in raw.content_sources.into_iter().enumerate() {
            let field_prefix = format!("content_sources[{}]", i);

            if src.source_id.trim().is_empty() {
                errors.push(
                    format!("{}.source_id", field_prefix),
                    "source_id cannot be empty",
                );
                continue;
            }

            if src.category_id.trim().is_empty() {
                errors.push(
                    format!("{}.category_id", field_prefix),
                    "category_id cannot be empty",
                );
                continue;
            }

            let resolved_path = if Path::new(&src.path).is_absolute() {
                PathBuf::from(&src.path)
            } else {
                base_path.join(&src.path)
            };

            if src.enabled {
                if !resolved_path.exists() {
                    errors.push_with_path(
                        format!("{}.path", field_prefix),
                        format!("Content source '{}' path does not exist", src.source_id),
                        &resolved_path,
                    );
                    continue;
                }

                if !resolved_path.is_dir() {
                    errors.push_with_path(
                        format!("{}.path", field_prefix),
                        format!("Content source '{}' path is not a directory", src.source_id),
                        &resolved_path,
                    );
                    continue;
                }
            }

            let canonical_path = resolved_path
                .canonicalize()
                .unwrap_or(resolved_path);

            content_sources.push(ContentSourceValidated {
                source_id: SourceId::new(src.source_id),
                category_id: CategoryId::new(src.category_id),
                path: canonical_path,
                allowed_content_types: src.allowed_content_types,
                enabled: src.enabled,
                override_existing: src.override_existing,
            });
        }

        errors.into_result(Self {
            content_sources,
            base_url,
            enable_link_tracking: raw.enable_link_tracking,
        })
    }

    /// Load and validate config from a YAML file.
    ///
    /// Convenience method that loads, parses, and validates in one step.
    pub fn load_from_file(path: &Path) -> Result<Self, ExtensionConfigErrors> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            let mut errors = ExtensionConfigErrors::new("blog");
            errors.push("_file", format!("Failed to read config file: {}", e));
            errors
        })?;

        let raw: BlogConfigRaw = serde_yaml::from_str(&content).map_err(|e| {
            let mut errors = ExtensionConfigErrors::new("blog");
            errors.push("_parse", format!("Failed to parse config YAML: {}", e));
            errors
        })?;

        let base_path = path.parent().unwrap_or(Path::new("."));
        Self::validate(raw, base_path)
    }

    /// Load config from environment variable or default path.
    ///
    /// Tries:
    /// 1. `BLOG_CONFIG` environment variable
    /// 2. `./services/config/blog.yaml` (default)
    /// 3. Empty default config if neither exists
    pub fn load_from_env_or_default() -> Result<Arc<Self>, ExtensionConfigErrors> {
        let config_path = std::env::var("BLOG_CONFIG")
            .unwrap_or_else(|_| "./services/config/blog.yaml".to_string());

        let path = Path::new(&config_path);
        if path.exists() {
            Self::load_from_file(path).map(Arc::new)
        } else {
            tracing::warn!(
                path = %config_path,
                "Blog config not found, using defaults"
            );
            let raw = BlogConfigRaw::default();
            Self::validate(raw, Path::new(".")).map(Arc::new)
        }
    }

    /// Get enabled content sources.
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

