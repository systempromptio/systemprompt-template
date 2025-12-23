//! Content validation service.

use crate::error::BlogError;
use crate::models::{ContentKind, ContentMetadata};

/// Service for validating content before ingestion or update.
#[derive(Debug, Clone, Default)]
pub struct ValidationService;

/// Result of content validation.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the content is valid.
    pub is_valid: bool,
    /// Validation errors, if any.
    pub errors: Vec<ValidationError>,
    /// Validation warnings (non-blocking issues).
    pub warnings: Vec<String>,
}

/// A validation error with field and message.
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// The field that failed validation.
    pub field: String,
    /// The error message.
    pub message: String,
}

impl ValidationResult {
    /// Create a successful validation result.
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create a failed validation result.
    pub fn invalid(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    /// Add a warning to the result.
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    /// Add an error to the result.
    pub fn with_error(mut self, error: ValidationError) -> Self {
        self.is_valid = false;
        self.errors.push(error);
        self
    }
}

impl ValidationError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

impl ValidationService {
    pub fn new() -> Self {
        Self
    }

    /// Validate content metadata.
    pub fn validate_metadata(&self, metadata: &ContentMetadata) -> ValidationResult {
        let mut result = ValidationResult::valid();

        // Validate title
        if metadata.title.trim().is_empty() {
            result = result.with_error(ValidationError::new("title", "Title cannot be empty"));
        } else if metadata.title.len() > 200 {
            result = result.with_error(ValidationError::new(
                "title",
                "Title cannot exceed 200 characters",
            ));
        }

        // Validate slug
        if metadata.slug.trim().is_empty() {
            result = result.with_error(ValidationError::new("slug", "Slug cannot be empty"));
        } else if !Self::is_valid_slug(&metadata.slug) {
            result = result.with_error(ValidationError::new(
                "slug",
                "Slug must contain only lowercase letters, numbers, and hyphens",
            ));
        }

        // Validate published_at
        if metadata.published_at.trim().is_empty() {
            result = result.with_error(ValidationError::new(
                "published_at",
                "Published date is required",
            ));
        }

        // Validate kind
        if metadata.kind.parse::<ContentKind>().is_err() {
            result = result.with_error(ValidationError::new(
                "kind",
                format!(
                    "Invalid content kind '{}'. Must be one of: article, paper, guide, tutorial",
                    metadata.kind
                ),
            ));
        }

        // Validate description (warning if empty)
        if metadata.description.trim().is_empty() {
            result = result.with_warning("Description is empty - this may affect SEO".to_string());
        } else if metadata.description.len() > 500 {
            result = result.with_error(ValidationError::new(
                "description",
                "Description cannot exceed 500 characters",
            ));
        }

        // Validate author (warning if empty)
        if metadata.author.trim().is_empty() {
            result = result.with_warning("Author is not specified".to_string());
        }

        // Validate keywords (warning if empty)
        if metadata.keywords.trim().is_empty() {
            result = result.with_warning("Keywords are empty - this may affect SEO".to_string());
        }

        result
    }

    /// Validate content body.
    pub fn validate_body(&self, body: &str) -> ValidationResult {
        let mut result = ValidationResult::valid();

        if body.trim().is_empty() {
            result = result.with_error(ValidationError::new("body", "Content body cannot be empty"));
        } else if body.len() < 100 {
            result = result.with_warning("Content body is very short (< 100 characters)".to_string());
        }

        result
    }

    /// Validate a URL format.
    pub fn validate_url(&self, url: &str, field_name: &str) -> ValidationResult {
        let mut result = ValidationResult::valid();

        if url.trim().is_empty() {
            return result; // Empty URLs are allowed (optional field)
        }

        // Basic URL validation
        if !url.starts_with("http://") && !url.starts_with("https://") {
            result = result.with_error(ValidationError::new(
                field_name,
                "URL must start with http:// or https://",
            ));
        }

        result
    }

    /// Validate content for ingestion (metadata + body).
    pub fn validate_content(
        &self,
        metadata: &ContentMetadata,
        body: &str,
    ) -> Result<(), BlogError> {
        let metadata_result = self.validate_metadata(metadata);
        let body_result = self.validate_body(body);

        let mut all_errors = metadata_result.errors;
        all_errors.extend(body_result.errors);

        if all_errors.is_empty() {
            Ok(())
        } else {
            let error_messages: Vec<String> = all_errors
                .iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect();
            Err(BlogError::Validation(error_messages.join("; ")))
        }
    }

    /// Check if a slug is valid (lowercase letters, numbers, hyphens only).
    fn is_valid_slug(slug: &str) -> bool {
        !slug.is_empty()
            && slug
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            && !slug.starts_with('-')
            && !slug.ends_with('-')
            && !slug.contains("--")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_slug() {
        assert!(ValidationService::is_valid_slug("hello-world"));
        assert!(ValidationService::is_valid_slug("post-123"));
        assert!(ValidationService::is_valid_slug("a"));
        assert!(!ValidationService::is_valid_slug(""));
        assert!(!ValidationService::is_valid_slug("-start"));
        assert!(!ValidationService::is_valid_slug("end-"));
        assert!(!ValidationService::is_valid_slug("double--hyphen"));
        assert!(!ValidationService::is_valid_slug("UPPERCASE"));
        assert!(!ValidationService::is_valid_slug("with spaces"));
    }
}
