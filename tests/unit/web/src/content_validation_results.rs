//! Body, URL and result-aggregation halves of the editorial gate.
//!
//! `validate_body` and `validate_url` are where "empty" means two different
//! things: an empty body blocks the publish, an empty URL is simply an absent
//! optional field and passes. `validate_content` is the caller-facing fold —
//! it merges metadata and body *errors* into one `BlogError::Validation`
//! string and deliberately drops warnings, so a page missing only its author
//! still ingests. The `ValidationResult` combinators underneath decide that:
//! `with_error` flips `is_valid`, `with_warning` never does.

use systemprompt_web_extension::error::BlogError;
use systemprompt_web_extension::models::ContentMetadata;
use systemprompt_web_extension::services::validation::{
    ValidationError, ValidationResult, ValidationService,
};

fn metadata() -> ContentMetadata {
    ContentMetadata {
        title: "A Title".to_owned(),
        description: "A description".to_owned(),
        author: "Ed".to_owned(),
        published_at: "2026-01-01".to_owned(),
        slug: "a-title".to_owned(),
        keywords: "one,two".to_owned(),
        kind: "guide".to_owned(),
        image: None,
        category: None,
        tags: Vec::new(),
        links: Vec::new(),
        after_reading_this: Vec::new(),
        related_playbooks: Vec::new(),
        related_code: Vec::new(),
        related_docs: Vec::new(),
    }
}

fn error_fields(result: &ValidationResult) -> Vec<&str> {
    result.errors.iter().map(|e| e.field.as_str()).collect()
}

#[test]
fn empty_body_is_an_error() {
    let result = ValidationService::validate_body("\n \t ");
    assert!(!result.is_valid);
    assert_eq!(error_fields(&result), vec!["body"]);
    assert!(result.warnings.is_empty());
}

#[test]
fn short_body_warns_but_stays_valid() {
    let result = ValidationService::validate_body("too short");
    assert!(result.is_valid);
    assert_eq!(result.warnings.len(), 1);
    assert!(result.warnings[0].contains("100"));
}

#[test]
fn body_at_the_minimum_length_is_clean() {
    let result = ValidationService::validate_body(&"b".repeat(100));
    assert!(result.is_valid);
    assert!(result.warnings.is_empty());
}

#[test]
fn empty_url_is_allowed() {
    let result = ValidationService::validate_url("   ", "canonical");
    assert!(result.is_valid);
    assert!(result.errors.is_empty());
}

#[test]
fn http_and_https_urls_are_allowed() {
    assert!(ValidationService::validate_url("http://a.test/x", "canonical").is_valid);
    assert!(ValidationService::validate_url("https://a.test/x", "canonical").is_valid);
}

#[test]
fn schemeless_url_is_an_error_on_the_named_field() {
    let result = ValidationService::validate_url("a.test/x", "canonical");
    assert!(!result.is_valid);
    assert_eq!(error_fields(&result), vec!["canonical"]);
}

#[test]
fn validate_content_accepts_clean_metadata_and_body() {
    assert!(ValidationService::validate_content(&metadata(), &"b".repeat(100)).is_ok());
}

#[test]
fn validate_content_ignores_warnings() {
    let mut meta = metadata();
    meta.author = String::new();
    assert!(ValidationService::validate_content(&meta, "short body").is_ok());
}

#[test]
fn validate_content_joins_metadata_and_body_errors() {
    let mut meta = metadata();
    meta.title = String::new();
    let Err(BlogError::Validation(message)) = ValidationService::validate_content(&meta, "  ")
    else {
        panic!("expected a validation error");
    };
    assert!(message.contains("title: Title cannot be empty"));
    assert!(message.contains("body: Content body cannot be empty"));
    assert!(message.contains("; "));
}

#[test]
fn with_warning_keeps_the_result_valid() {
    let result = ValidationResult::valid().with_warning("careful".to_owned());
    assert!(result.is_valid);
    assert_eq!(result.warnings, vec!["careful".to_owned()]);
}

#[test]
fn with_error_flips_valid_and_keeps_order() {
    let result = ValidationResult::valid()
        .with_error(ValidationError::new("a", "first"))
        .with_error(ValidationError::new("b", "second"));
    assert!(!result.is_valid);
    assert_eq!(error_fields(&result), vec!["a", "b"]);
}

#[test]
fn invalid_carries_errors_without_warnings() {
    let result = ValidationResult::invalid(vec![ValidationError::new("slug", "bad")]);
    assert!(!result.is_valid);
    assert_eq!(result.errors.len(), 1);
    assert!(result.warnings.is_empty());
}
