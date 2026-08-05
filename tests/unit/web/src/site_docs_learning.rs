//! `DocsLearningContent` reads the optional "what you'll learn" block off a
//! content item, and its template data drives `{{#if}}` guards in
//! `docs-page.html`. Two invariants matter: a malformed or missing field must
//! degrade to the type default rather than fail the page, and falsey values
//! (empty lists, `has_learning_content: false`) must be *absent* from the
//! rendered map — a present-but-false key would open the guard.

use serde_json::{Value, json};
use systemprompt_web_site::docs::DocsLearningContent;

fn keys(value: &Value) -> Vec<String> {
    value
        .as_object()
        .expect("template data is an object")
        .keys()
        .cloned()
        .collect()
}

#[test]
fn reads_all_three_lists_off_the_content_item() {
    let content = DocsLearningContent::from_content_item(&json!({
        "after_reading_this": ["Route a tool call", "Read the audit trail"],
        "related_playbooks": [{ "title": "Governance", "url": "/docs/governance" }],
        "related_code": [{ "title": "extension.rs", "url": "/code/extension" }],
    }));

    assert_eq!(content.after_reading_this.len(), 2);
    assert_eq!(content.related_playbooks[0].title, "Governance");
    assert_eq!(content.related_code[0].url, "/code/extension");
}

#[test]
fn an_absent_field_defaults_to_an_empty_list() {
    let content = DocsLearningContent::from_content_item(&json!({ "title": "Docs" }));

    assert!(content.after_reading_this.is_empty());
    assert!(content.related_playbooks.is_empty());
    assert!(content.related_code.is_empty());
}

#[test]
fn a_malformed_field_degrades_to_the_default_instead_of_failing() {
    let content = DocsLearningContent::from_content_item(&json!({
        "after_reading_this": "not a list",
        "related_playbooks": [{ "title": "missing url" }],
        "related_code": 42,
    }));

    assert!(content.after_reading_this.is_empty());
    assert!(content.related_playbooks.is_empty());
    assert!(content.related_code.is_empty());
}

#[test]
fn has_content_is_true_when_any_single_list_is_populated() {
    let empty = DocsLearningContent::default();
    assert!(!empty.has_content());

    let only_code = DocsLearningContent::from_content_item(&json!({
        "related_code": [{ "title": "lib.rs", "url": "/code/lib" }],
    }));
    assert!(only_code.has_content());
}

#[test]
fn empty_content_renders_no_keys_at_all() {
    let data = DocsLearningContent::default().to_template_data();
    assert!(
        keys(&data).is_empty(),
        "falsey learning content must contribute no template keys, got {:?}",
        keys(&data)
    );
}

#[test]
fn populated_content_sets_the_has_learning_content_flag() {
    let content = DocsLearningContent::from_content_item(&json!({
        "after_reading_this": ["Ship a skill"],
    }));
    let data = content.to_template_data();

    assert_eq!(data["HAS_LEARNING_CONTENT"], json!(true));
    assert_eq!(data["AFTER_READING_THIS"], json!(["Ship a skill"]));
    assert!(data.get("RELATED_PLAYBOOKS").is_none());
    assert!(data.get("RELATED_CODE").is_none());
}

#[test]
fn template_keys_are_the_uppercase_names_the_template_reads() {
    let content = DocsLearningContent::from_content_item(&json!({
        "after_reading_this": ["a"],
        "related_playbooks": [{ "title": "p", "url": "/p" }],
        "related_code": [{ "title": "c", "url": "/c" }],
    }));
    let mut names = keys(&content.to_template_data());
    names.sort();

    assert_eq!(
        names,
        vec![
            "AFTER_READING_THIS",
            "HAS_LEARNING_CONTENT",
            "RELATED_CODE",
            "RELATED_PLAYBOOKS",
        ]
    );
}
