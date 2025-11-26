use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for image storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Base directory for storing generated images
    /// Default: ${`STORAGE_PATH}/generated_images` (typically /`app/storage/generated_images` in production)
    pub base_path: PathBuf,

    /// URL prefix for serving images
    /// Example: <https://example.com/generated/images>
    pub url_prefix: String,

    /// Maximum file size in bytes (default 10MB)
    #[serde(default = "default_max_file_size")]
    pub max_file_size_bytes: usize,

    /// Whether to organize images into subdirectories by date (YYYY/MM/DD)
    #[serde(default = "default_organize_by_date")]
    pub organize_by_date: bool,
}

const fn default_max_file_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

const fn default_organize_by_date() -> bool {
    true
}

impl Default for StorageConfig {
    fn default() -> Self {
        let storage_base = std::env::var("STORAGE_PATH")
            .or_else(|_| std::env::var("SYSTEM_PATH").map(|p| format!("{p}/storage")))
            .unwrap_or_else(|_| "/app/storage".to_string());

        Self {
            base_path: PathBuf::from(storage_base).join("generated_images"),
            url_prefix: "/generated/images".to_string(),
            max_file_size_bytes: default_max_file_size(),
            organize_by_date: true,
        }
    }
}

impl StorageConfig {
    pub fn new(base_path: PathBuf, url_prefix: String) -> Self {
        Self {
            base_path,
            url_prefix,
            ..Default::default()
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.url_prefix.is_empty() {
            return Err("url_prefix cannot be empty".to_string());
        }

        if self.max_file_size_bytes == 0 {
            return Err("max_file_size_bytes must be greater than 0".to_string());
        }

        Ok(())
    }
}
