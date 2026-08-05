//! Front-matter validation is the only gate between a markdown file on disk
//! and a published row, and the split between what blocks a publish and what
//! merely annoys the author is the contract: empty or oversized title,
//! malformed slug, missing date, unknown kind and oversized description are
//! errors, while an empty description, author or keywords list only warns and
//! still publishes. Slug rules are the fiddly half — they decide the public
//! URL, so lowercase/digit/hyphen only, and no leading, trailing or doubled
//! hyphen.

use systemprompt_web_extension::models::ContentMetadata;
use systemprompt_web_extension::services::validation::{ValidationResult, ValidationService};

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
fn complete_metadata_is_valid_without_warnings() {
    let result = ValidationService::validate_metadata(&metadata());
    assert!(result.is_valid);
    assert!(result.errors.is_empty());
    assert!(result.warnings.is_empty());
}

#[test]
fn empty_title_is_an_error() {
    let mut meta = metadata();
    meta.title = "   ".to_owned();
    let result = ValidationService::validate_metadata(&meta);
    assert!(!result.is_valid);
    assert_eq!(error_fields(&result), vec!["title"]);
}

#[test]
fn title_at_the_limit_is_accepted() {
    let mut meta = metadata();
    meta.title = "t".repeat(200);
    assert!(ValidationService::validate_metadata(&meta).is_valid);
}

#[test]
fn title_over_the_limit_is_an_error() {
    let mut meta = metadata();
    meta.title = "t".repeat(201);
    let result = ValidationService::validate_metadata(&meta);
    assert_eq!(error_fields(&result), vec!["title"]);
    assert!(result.errors[0].message.contains("200"));
}

#[test]
fn empty_slug_reports_emptiness_not_charset() {
    let mut meta = metadata();
    meta.slug = String::new();
    let result = ValidationService::validate_metadata(&meta);
    assert_eq!(error_fields(&result), vec!["slug"]);
    assert!(result.errors[0].message.contains("cannot be empty"));
}

#[test]
fn slug_of_lowercase_digits_and_hyphens_is_valid() {
    let mut meta = metadata();
    meta.slug = "guide-2026-part-3".to_owned();
    assert!(ValidationService::validate_metadata(&meta).is_valid);
}

#[test]
fn uppercase_slug_is_rejected() {
    let mut meta = metadata();
    meta.slug = "A-Title".to_owned();
    assert_eq!(
        error_fields(&ValidationService::validate_metadata(&meta)),
        vec!["slug"]
    );
}

#[test]
fn slug_with_underscore_or_space_is_rejected() {
    for slug in ["a_title", "a title", "a.title", "café"] {
        let mut meta = metadata();
        meta.slug = slug.to_owned();
        assert!(
            !ValidationService::validate_metadata(&meta).is_valid,
            "slug {slug} should be rejected"
        );
    }
}

#[test]
fn leading_hyphen_slug_is_rejected() {
    let mut meta = metadata();
    meta.slug = "-title".to_owned();
    assert!(!ValidationService::validate_metadata(&meta).is_valid);
}

#[test]
fn trailing_hyphen_slug_is_rejected() {
    let mut meta = metadata();
    meta.slug = "title-".to_owned();
    assert!(!ValidationService::validate_metadata(&meta).is_valid);
}

#[test]
fn double_hyphen_slug_is_rejected() {
    let mut meta = metadata();
    meta.slug = "a--title".to_owned();
    assert!(!ValidationService::validate_metadata(&meta).is_valid);
}

#[test]
fn single_character_slug_is_valid() {
    let mut meta = metadata();
    meta.slug = "a".to_owned();
    assert!(ValidationService::validate_metadata(&meta).is_valid);
}

#[test]
fn missing_published_at_is_an_error() {
    let mut meta = metadata();
    meta.published_at = " ".to_owned();
    assert_eq!(
        error_fields(&ValidationService::validate_metadata(&meta)),
        vec!["published_at"]
    );
}

#[test]
fn every_known_kind_is_accepted() {
    for kind in [
        "blog",
        "guide",
        "tutorial",
        "reference",
        "docs-index",
        "docs",
        "docs-list",
        "feature",
        "playbook",
        "legal",
    ] {
        let mut meta = metadata();
        meta.kind = kind.to_owned();
        assert!(
            ValidationService::validate_metadata(&meta).is_valid,
            "kind {kind} should be accepted"
        );
    }
}

#[test]
fn unknown_kind_is_an_error_naming_the_value() {
    let mut meta = metadata();
    meta.kind = "newsletter".to_owned();
    let result = ValidationService::validate_metadata(&meta);
    assert_eq!(error_fields(&result), vec!["kind"]);
    assert!(result.errors[0].message.contains("newsletter"));
}

#[test]
fn empty_description_warns_but_stays_valid() {
    let mut meta = metadata();
    meta.description = String::new();
    let result = ValidationService::validate_metadata(&meta);
    assert!(result.is_valid);
    assert_eq!(result.warnings.len(), 1);
    assert!(result.warnings[0].contains("SEO"));
}

#[test]
fn description_at_the_limit_is_accepted() {
    let mut meta = metadata();
    meta.description = "d".repeat(500);
    assert!(ValidationService::validate_metadata(&meta).is_valid);
}

#[test]
fn description_over_the_limit_is_an_error() {
    let mut meta = metadata();
    meta.description = "d".repeat(501);
    assert_eq!(
        error_fields(&ValidationService::validate_metadata(&meta)),
        vec!["description"]
    );
}

#[test]
fn missing_author_and_keywords_are_warnings_only() {
    let mut meta = metadata();
    meta.author = String::new();
    meta.keywords = "  ".to_owned();
    let result = ValidationService::validate_metadata(&meta);
    assert!(result.is_valid);
    assert_eq!(result.warnings.len(), 2);
}

#[test]
fn errors_accumulate_across_fields() {
    let mut meta = metadata();
    meta.title = String::new();
    meta.slug = "Bad Slug".to_owned();
    meta.published_at = String::new();
    meta.kind = "nope".to_owned();
    let result = ValidationService::validate_metadata(&meta);
    assert_eq!(
        error_fields(&result),
        vec!["title", "slug", "published_at", "kind"]
    );
}
