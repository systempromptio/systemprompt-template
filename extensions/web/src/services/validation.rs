use crate::error::BlogError;
use crate::models::{ContentKind, ContentMetadata};

const MAX_TITLE_LENGTH: usize = 200;
const MAX_DESCRIPTION_LENGTH: usize = 500;
const MIN_BODY_LENGTH: usize = 100;

#[derive(Debug, Clone, Default)]
pub struct ValidationService;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl ValidationResult {
    #[must_use]
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    #[must_use]
    pub fn invalid(errors: Vec<ValidationError>) -> Self {
        Self {
            is_valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }

    #[must_use]
    pub fn with_error(mut self, error: ValidationError) -> Self {
        self.is_valid = false;
        self.errors.push(error);
        self
    }
}

impl ValidationError {
    #[must_use]
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

impl ValidationService {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn validate_metadata(&self, metadata: &ContentMetadata) -> ValidationResult {
        let mut result = ValidationResult::valid();

        if metadata.title.trim().is_empty() {
            result = result.with_error(ValidationError::new("title", "Title cannot be empty"));
        } else if metadata.title.len() > MAX_TITLE_LENGTH {
            result = result.with_error(ValidationError::new(
                "title",
                format!("Title cannot exceed {MAX_TITLE_LENGTH} characters"),
            ));
        }

        if metadata.slug.trim().is_empty() {
            result = result.with_error(ValidationError::new("slug", "Slug cannot be empty"));
        } else if !Self::is_valid_slug(&metadata.slug) {
            result = result.with_error(ValidationError::new(
                "slug",
                "Slug must contain only lowercase letters, numbers, and hyphens",
            ));
        }

        if metadata.published_at.trim().is_empty() {
            result = result.with_error(ValidationError::new(
                "published_at",
                "Published date is required",
            ));
        }

        if metadata.kind.parse::<ContentKind>().is_err() {
            result = result.with_error(ValidationError::new(
                "kind",
                format!(
                    "Invalid content kind '{}'. Must be one of: blog, guide, tutorial, reference, docs-index, docs, docs-list, feature, playbook, legal",
                    metadata.kind
                ),
            ));
        }

        if metadata.description.trim().is_empty() {
            result = result.with_warning("Description is empty - this may affect SEO".to_string());
        } else if metadata.description.len() > MAX_DESCRIPTION_LENGTH {
            result = result.with_error(ValidationError::new(
                "description",
                format!("Description cannot exceed {MAX_DESCRIPTION_LENGTH} characters"),
            ));
        }

        if metadata.author.trim().is_empty() {
            result = result.with_warning("Author is not specified".to_string());
        }

        if metadata.keywords.trim().is_empty() {
            result = result.with_warning("Keywords are empty - this may affect SEO".to_string());
        }

        result
    }

    #[must_use]
    pub fn validate_body(&self, body: &str) -> ValidationResult {
        let mut result = ValidationResult::valid();

        if body.trim().is_empty() {
            result =
                result.with_error(ValidationError::new("body", "Content body cannot be empty"));
        } else if body.len() < MIN_BODY_LENGTH {
            result = result.with_warning(format!(
                "Content body is very short (< {MIN_BODY_LENGTH} characters)"
            ));
        }

        result
    }

    #[must_use]
    pub fn validate_url(&self, url: &str, field_name: &str) -> ValidationResult {
        let mut result = ValidationResult::valid();

        if url.trim().is_empty() {
            return result;
        }

        if !url.starts_with("http://") && !url.starts_with("https://") {
            result = result.with_error(ValidationError::new(
                field_name,
                "URL must start with http:// or https://",
            ));
        }

        result
    }

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
