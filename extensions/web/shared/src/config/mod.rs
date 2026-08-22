//! The `services/content/` configuration model.
//!
//! Raw deserialisation and validation are separate types: a
//! `BlogConfigValidated` can only be produced by `validate`, so downstream code
//! never handles a partially checked config.

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use serde::Deserialize;
use systemprompt::identifiers::{CategoryId, SourceId};
use url::Url;

pub use crate::config_errors::{ExtensionConfigError, ExtensionConfigErrors};

mod config_paths;
use config_paths::{resolve_blog_config_path, validate_content_source};

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
    pub source_id: SourceId,
    pub category_id: CategoryId,
    pub path: String,
    #[serde(default)]
    pub allowed_content_types: Vec<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub override_existing: bool,
}

fn default_base_url() -> String {
    "https://example.com".to_owned()
}

const fn default_true() -> bool {
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
    pub fn validate(raw: BlogConfigRaw, base_path: &Path) -> Result<Self, ExtensionConfigErrors> {
        let mut errors = ExtensionConfigErrors::new("blog");

        let base_url = validate_base_url(&raw.base_url, &mut errors);

        let mut content_sources = Vec::with_capacity(raw.content_sources.len());
        for (i, src) in raw.content_sources.into_iter().enumerate() {
            if let Some(validated) = validate_content_source(src, i, base_path, &mut errors) {
                content_sources.push(validated);
            }
        }

        // Why: `base_url` is `None` only when the URL failed to parse, which
        // always records an error above — so `into_result` surfaces it as `Err`.
        match base_url {
            Some(base_url) => errors.into_result(Self {
                content_sources,
                base_url,
                enable_link_tracking: raw.enable_link_tracking,
            }),
            None => Err(errors),
        }
    }

    pub fn load_from_file(path: &Path) -> Result<Self, ExtensionConfigErrors> {
        // Why: ExtensionConfigErrors is a field-keyed validation accumulator, not a
        // variant enum; messages are its contract. lint-ok: error-adapt
        let content = std::fs::read_to_string(path).map_err(|e| {
            let mut errors = ExtensionConfigErrors::new("blog");
            errors.push("_file", format!("Failed to read config file: {e}"));
            errors
        })?;

        // Why: ExtensionConfigErrors is a field-keyed validation accumulator, not a
        // variant enum; messages are its contract. lint-ok: error-adapt
        let raw: BlogConfigRaw = serde_yaml::from_str(&content).map_err(|e| {
            let mut errors = ExtensionConfigErrors::new("blog");
            errors.push("_parse", format!("Failed to parse config YAML: {e}"));
            errors
        })?;

        let base_path = path.parent().unwrap_or_else(|| Path::new("."));
        Self::validate(raw, base_path)
    }

    // Why: A missing file resolves to `Ok(None)`: "blog disabled" is a supported
    // state, not a degraded one. `Err` is reserved for a file that exists but
    // cannot be read, parsed, or validated.
    pub fn load_from_env_or_none() -> Result<Option<Arc<Self>>, ExtensionConfigErrors> {
        let config_path = resolve_blog_config_path();
        if config_path.exists() {
            Self::load_from_file(&config_path).map(|c| Some(Arc::new(c)))
        } else {
            Ok(None)
        }
    }

    // Why: Every consumer of the blog config (link API routing, content
    // ingestion) must go through this single load path so they cannot
    // disagree about which config the process is running with.
    pub fn cached() -> Result<Option<Arc<Self>>, String> {
        static CACHED: OnceLock<Result<Option<Arc<BlogConfigValidated>>, String>> = OnceLock::new();
        CACHED
            .get_or_init(|| Self::load_from_env_or_none().map_err(|e| e.to_string()))
            .clone()
    }

    pub fn enabled_sources(&self) -> impl Iterator<Item = &ContentSourceValidated> {
        self.content_sources.iter().filter(|s| s.enabled)
    }

    #[must_use]
    pub fn all_sources(&self) -> &[ContentSourceValidated] {
        &self.content_sources
    }

    #[must_use]
    pub const fn base_url(&self) -> &Url {
        &self.base_url
    }

    #[must_use]
    pub const fn link_tracking_enabled(&self) -> bool {
        self.enable_link_tracking
    }
}

fn validate_base_url(raw_url: &str, errors: &mut ExtensionConfigErrors) -> Option<Url> {
    match Url::parse(raw_url) {
        Ok(url) => {
            if url.scheme() != "http" && url.scheme() != "https" {
                let scheme = url.scheme();
                errors.push_with_suggestion(
                    "base_url",
                    format!("URL must use http or https scheme, got: {scheme}"),
                    "Use a URL like https://example.com",
                );
            }
            Some(url)
        },
        Err(e) => {
            errors.push_with_suggestion(
                "base_url",
                format!("Invalid URL: {e}"),
                "Use a valid URL like https://example.com",
            );
            None
        },
    }
}

impl ContentSourceValidated {
    #[must_use]
    pub const fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    #[must_use]
    pub const fn category_id(&self) -> &CategoryId {
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
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }

    #[must_use]
    pub const fn override_existing(&self) -> bool {
        self.override_existing
    }
}
