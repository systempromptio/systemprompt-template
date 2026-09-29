//! `ExtensionConfigErrors` accumulates rather than short-circuits: a
//! misconfigured deployment must report every problem in one startup failure,
//! and the rendered message must carry the optional path and suggestion that
//! tell an operator where to look and what to do.

use std::path::PathBuf;
use systemprompt_web_shared::config_errors::ExtensionConfigErrors;

#[test]
fn a_fresh_collection_is_empty_and_passes_its_value_through() {
    let errors = ExtensionConfigErrors::new("blog");
    assert!(errors.is_empty());
    assert_eq!(errors.into_result(42_u32).unwrap(), 42);
}

#[test]
fn pushed_errors_accumulate_instead_of_replacing_each_other() {
    let mut errors = ExtensionConfigErrors::new("blog");
    errors.push("base_url", "invalid URL");
    errors.push("content_sources[0].path", "does not exist");
    assert!(!errors.is_empty());
    assert_eq!(errors.errors.len(), 2);
    assert_eq!(errors.errors[0].field, "base_url");
    assert_eq!(errors.errors[1].message, "does not exist");
}

#[test]
fn a_plain_push_leaves_path_and_suggestion_unset() {
    let mut errors = ExtensionConfigErrors::new("blog");
    errors.push("field", "message");
    assert!(errors.errors[0].path.is_none());
    assert!(errors.errors[0].suggestion.is_none());
}

#[test]
fn push_with_path_and_push_with_suggestion_fill_one_field_each() {
    let mut errors = ExtensionConfigErrors::new("blog");
    errors.push_with_path("a.path", "missing", PathBuf::from("/srv/content"));
    errors.push_with_suggestion("base_url", "bad scheme", "Use https://example.com");

    assert_eq!(errors.errors[0].path, Some(PathBuf::from("/srv/content")));
    assert!(errors.errors[0].suggestion.is_none());
    assert!(errors.errors[1].path.is_none());
    assert_eq!(
        errors.errors[1].suggestion.as_deref(),
        Some("Use https://example.com")
    );
}

#[test]
fn a_non_empty_collection_turns_into_result_as_err() {
    let mut errors = ExtensionConfigErrors::new("blog");
    errors.push("field", "message");
    let err = errors.into_result("unused value").unwrap_err();
    assert_eq!(err.errors.len(), 1);
}

#[test]
fn display_names_the_extension_and_renders_path_and_fix_lines() {
    let mut errors = ExtensionConfigErrors::new("blog");
    errors.push_with_path("a.path", "missing", PathBuf::from("/srv/content"));
    errors.push_with_suggestion("base_url", "bad scheme", "Use https://example.com");
    let rendered = errors.to_string();

    assert!(rendered.contains("Extension 'blog' configuration errors:"));
    assert!(rendered.contains("[a.path] missing"));
    assert!(rendered.contains("Path: /srv/content"));
    assert!(rendered.contains("[base_url] bad scheme"));
    assert!(rendered.contains("Fix: Use https://example.com"));
}
