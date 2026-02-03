use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

use serde::Deserialize;
use systemprompt::identifiers::{CategoryId, SourceId};
use systemprompt::models::AppPaths;
use url::Url;

pub use crate::config_errors::{ExtensionConfigError, ExtensionConfigErrors};

static FALLBACK_URL: LazyLock<Url> = LazyLock::new(|| {
    Url::parse("https://invalid.example.com")
        .unwrap_or_else(|_| Url::parse("https://localhost").unwrap_or_else(|_| unreachable!()))
});

#[derive(Debug, Clone, Deserialize)]
pub struct BlogConfigRaw {
    #[serde(default)]
    pub content_sources: Vec<ContentSourceRaw>,

    #[serde(default = "default_base_url")]
    pub base_url: String,

    #[serde(default = "default_true")]
    pub enable_link_tracking: bool,
}

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
    path: PathBuf,
    allowed_content_types: Vec<String>,
    enabled: bool,
    override_existing: bool,
}

impl BlogConfigValidated {
    #[allow(clippy::missing_panics_doc)]
    pub fn validate(raw: BlogConfigRaw, base_path: &Path) -> Result<Self, ExtensionConfigErrors> {
        let mut errors = ExtensionConfigErrors::new("blog");

        let base_url = match Url::parse(&raw.base_url) {
            Ok(url) => {
                if url.scheme() != "http" && url.scheme() != "https" {
                    let scheme = url.scheme();
                    errors.push_with_suggestion(
                        "base_url",
                        format!("URL must use http or https scheme, got: {scheme}"),
                        "Use a URL like https://example.com",
                    );
                }
                url
            }
            Err(e) => {
                errors.push_with_suggestion(
                    "base_url",
                    format!("Invalid URL: {e}"),
                    "Use a valid URL like https://example.com",
                );
                FALLBACK_URL.clone()
            }
        };

        let mut content_sources = Vec::with_capacity(raw.content_sources.len());

        for (i, src) in raw.content_sources.into_iter().enumerate() {
            let field_prefix = format!("content_sources[{i}]");

            if src.source_id.trim().is_empty() {
                errors.push(
                    format!("{field_prefix}.source_id"),
                    "source_id cannot be empty",
                );
                continue;
            }

            if src.category_id.trim().is_empty() {
                errors.push(
                    format!("{field_prefix}.category_id"),
                    "category_id cannot be empty",
                );
                continue;
            }

            let resolved_path = if Path::new(&src.path).is_absolute() {
                PathBuf::from(&src.path)
            } else if src.path.starts_with("./") {
                let services_dir = AppPaths::get().map_or_else(
                    |_| PathBuf::from("./services"),
                    |p| p.system().services().to_path_buf(),
                );
                let clean_path = src.path.strip_prefix("./services/").unwrap_or(&src.path);
                services_dir.join(clean_path)
            } else {
                base_path.join(&src.path)
            };

            if src.enabled {
                let source_id = &src.source_id;
                if !resolved_path.exists() {
                    errors.push_with_path(
                        format!("{field_prefix}.path"),
                        format!("Content source '{source_id}' path does not exist"),
                        &resolved_path,
                    );
                    continue;
                }

                if !resolved_path.is_dir() {
                    errors.push_with_path(
                        format!("{field_prefix}.path"),
                        format!("Content source '{source_id}' path is not a directory"),
                        &resolved_path,
                    );
                    continue;
                }
            }

            let canonical_path = resolved_path.canonicalize().unwrap_or(resolved_path);

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

    pub fn load_from_file(path: &Path) -> Result<Self, ExtensionConfigErrors> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            let mut errors = ExtensionConfigErrors::new("blog");
            errors.push("_file", format!("Failed to read config file: {e}"));
            errors
        })?;

        let raw: BlogConfigRaw = serde_yaml::from_str(&content).map_err(|e| {
            let mut errors = ExtensionConfigErrors::new("blog");
            errors.push("_parse", format!("Failed to parse config YAML: {e}"));
            errors
        })?;

        let base_path = path.parent().unwrap_or(Path::new("."));
        Self::validate(raw, base_path)
    }

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

    pub fn enabled_sources(&self) -> impl Iterator<Item = &ContentSourceValidated> {
        self.content_sources.iter().filter(|s| s.enabled)
    }

    #[must_use]
    pub fn all_sources(&self) -> &[ContentSourceValidated] {
        &self.content_sources
    }

    #[must_use]
    pub fn base_url(&self) -> &Url {
        &self.base_url
    }

    #[must_use]
    pub fn link_tracking_enabled(&self) -> bool {
        self.enable_link_tracking
    }
}

impl ContentSourceValidated {
    #[must_use]
    pub fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    #[must_use]
    pub fn category_id(&self) -> &CategoryId {
        &self.category_id
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn allowed_content_types(&self) -> &[String] {
        &self.allowed_content_types
    }

    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[must_use]
    pub fn override_existing(&self) -> bool {
        self.override_existing
    }
}
