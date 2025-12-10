use super::storage_config::StorageConfig;
use crate::error::AiError;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use chrono::{Datelike, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Handles filesystem storage operations for generated images
#[derive(Debug)]
pub struct ImageStorage {
    config: StorageConfig,
}

impl ImageStorage {
    /// Create a new `ImageStorage` instance
    pub fn new(config: StorageConfig) -> Result<Self, AiError> {
        config
            .validate()
            .map_err(|e| AiError::StorageError(format!("Invalid storage configuration: {e}")))?;

        if !config.base_path.exists() {
            fs::create_dir_all(&config.base_path).map_err(|e| {
                AiError::StorageError(format!(
                    "Failed to create storage directory {}: {}",
                    config.base_path.display(),
                    e
                ))
            })?;
        }

        Ok(Self { config })
    }

    /// Save a base64-encoded image to the filesystem
    /// Returns (`file_path`, `public_url`)
    pub fn save_base64_image(
        &self,
        base64_data: &str,
        mime_type: &str,
    ) -> Result<(PathBuf, String), AiError> {
        let image_bytes = BASE64
            .decode(base64_data)
            .map_err(|e| AiError::StorageError(format!("Failed to decode base64 image: {e}")))?;

        self.save_image_bytes(&image_bytes, mime_type)
    }

    /// Save raw image bytes to the filesystem
    /// Returns (`file_path`, `public_url`)
    pub fn save_image_bytes(
        &self,
        image_bytes: &[u8],
        mime_type: &str,
    ) -> Result<(PathBuf, String), AiError> {
        if image_bytes.len() > self.config.max_file_size_bytes {
            return Err(AiError::StorageError(format!(
                "Image size {} bytes exceeds maximum allowed size {} bytes",
                image_bytes.len(),
                self.config.max_file_size_bytes
            )));
        }

        let extension = Self::mime_type_to_extension(mime_type);
        let filename = format!(
            "{}_{}.{}",
            Uuid::new_v4(),
            Utc::now().timestamp(),
            extension
        );

        let relative_path = if self.config.organize_by_date {
            let now = Utc::now();
            PathBuf::from(format!(
                "{}/{:04}/{:02}/{:02}/{}",
                self.config.base_path.display(),
                now.year(),
                now.month(),
                now.day(),
                filename
            ))
        } else {
            self.config.base_path.join(&filename)
        };

        if let Some(parent) = relative_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    AiError::StorageError(format!(
                        "Failed to create directory {}: {e}",
                        parent.display()
                    ))
                })?;
            }
        }

        fs::write(&relative_path, image_bytes).map_err(|e| {
            AiError::StorageError(format!(
                "Failed to write image file {}: {e}",
                relative_path.display()
            ))
        })?;

        let url_path = if self.config.organize_by_date {
            let now = Utc::now();
            format!(
                "{}/{:04}/{:02}/{:02}/{}",
                self.config.url_prefix,
                now.year(),
                now.month(),
                now.day(),
                filename
            )
        } else {
            format!("{}/{}", self.config.url_prefix, filename)
        };

        Ok((relative_path, url_path))
    }

    /// Delete an image file from the filesystem
    pub fn delete_image(&self, file_path: &Path) -> Result<(), AiError> {
        if !file_path.exists() {
            return Err(AiError::StorageError(format!(
                "File does not exist: {}",
                file_path.display()
            )));
        }

        fs::remove_file(file_path).map_err(|e| {
            AiError::StorageError(format!(
                "Failed to delete file {}: {e}",
                file_path.display()
            ))
        })?;

        if let Some(parent) = file_path.parent() {
            let _ = self.cleanup_empty_directories(parent);
        }

        Ok(())
    }

    /// Check if an image file exists
    pub fn exists(file_path: &Path) -> bool {
        file_path.exists()
    }

    /// Get the full path for a relative path
    pub fn get_full_path(&self, relative_path: &str) -> PathBuf {
        self.config.base_path.join(relative_path)
    }

    /// Convert MIME type to file extension
    fn mime_type_to_extension(mime_type: &str) -> String {
        match mime_type {
            "image/jpeg" | "image/jpg" => "jpg",
            "image/webp" => "webp",
            "image/gif" => "gif",
            _ => "png", // default to PNG (also handles "image/png")
        }
        .to_string()
    }

    /// Clean up empty directories recursively
    fn cleanup_empty_directories(&self, dir: &Path) -> Result<(), std::io::Error> {
        if dir == self.config.base_path {
            return Ok(());
        }

        if dir.read_dir()?.next().is_none() {
            fs::remove_dir(dir)?;

            if let Some(parent) = dir.parent() {
                let _ = self.cleanup_empty_directories(parent);
            }
        }

        Ok(())
    }
}
