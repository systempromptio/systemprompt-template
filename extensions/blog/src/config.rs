//! Blog extension configuration.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the blog extension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogConfig {
    /// Content sources to ingest
    #[serde(default)]
    pub content_sources: Vec<ContentSource>,

    /// Base URL for generated links
    #[serde(default = "default_base_url")]
    pub base_url: String,

    /// Enable link tracking
    #[serde(default = "default_true")]
    pub enable_link_tracking: bool,
}

/// A content source configuration for ingestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSource {
    /// Unique identifier for this source
    pub source_id: String,

    /// Category ID for content from this source
    pub category_id: String,

    /// Filesystem path to content directory
    pub path: PathBuf,

    /// Allowed content types (e.g., ["article", "tutorial"])
    #[serde(default)]
    pub allowed_content_types: Vec<String>,

    /// Whether this source is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Override existing content on re-ingestion
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
    "https://example.com".to_string()
}

fn default_true() -> bool {
    true
}
