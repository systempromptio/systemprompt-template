//! The documentation page-data provider, driven through a real `PageContext`.
//!
//! Every key this provider emits is a `{{UPPERCASE}}` slot in
//! `docs-page.html`, and the template renders under strict mode: a key that
//! stops being emitted is a render failure on every documentation page, and a
//! key emitted as present-but-empty defeats the `{{#if}}` guards around the
//! learning-content block. Both directions are asserted here.
//!
//! The provider also refuses a context with no content item rather than
//! emitting an empty page, which is the difference between a visibly missing
//! doc and a silently blank one.

use systemprompt::extension::prelude::{PageContext, PageDataProvider};
use systemprompt::models::services::WebConfig;
use systemprompt_web_site::docs::DocsPageDataProvider;

const WEB_CONFIG_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../services/web/config.yaml"
);

fn web_config() -> WebConfig {
    let raw = std::fs::read_to_string(WEB_CONFIG_PATH).expect("the deployment ships a web config");
    serde_yaml::from_str(&raw).expect("services/web/config.yaml deserialises into a WebConfig")
}

fn block_on<T>(future: impl Future<Output = Result<T, systemprompt::traits::ProviderError>>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("build a current-thread runtime")
        .block_on(future)
        .expect("the provider succeeds")
}

// JSON: the provider's contract is a JSON template context
fn page_data(item: &serde_json::Value) -> serde_json::Value {
    let config = web_config();
    let erased = ();
    let ctx = PageContext::new("docs-page", &config, &erased, &erased).with_content_item(item);
    block_on(DocsPageDataProvider::new().provide_page_data(&ctx))
}

#[test]
fn a_context_without_a_content_item_is_refused() {
    let config = web_config();
    let erased = ();
    let ctx = PageContext::new("docs-page", &config, &erased, &erased);

    let error = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("build a current-thread runtime")
        .block_on(DocsPageDataProvider::new().provide_page_data(&ctx))
        .expect_err("a documentation page without content cannot be rendered");

    assert!(
        error.to_string().to_lowercase().contains("content item"),
        "the refusal names what was missing: {error}"
    );
}

#[test]
fn every_string_field_lands_in_its_uppercase_template_slot() {
    let data = page_data(&serde_json::json!({
        "title": "Governance Pipeline",
        "description": "How a tool call is audited.",
        "slug": "services/governance",
        "author": "Astound Digital",
        "keywords": "governance, audit",
        "image": "/files/images/governance.png",
    }));

    assert_eq!(data["TITLE"], "Governance Pipeline");
    assert_eq!(data["DESCRIPTION"], "How a tool call is audited.");
    assert_eq!(data["SLUG"], "services/governance");
    assert_eq!(data["AUTHOR"], "Astound Digital");
    assert_eq!(data["KEYWORDS"], "governance, audit");
    assert_eq!(data["IMAGE"], "/files/images/governance.png");
}

#[test]
fn a_field_that_is_absent_or_not_a_string_is_omitted_entirely() {
    let data = page_data(&serde_json::json!({
        "title": "Only A Title",
        "description": 42,
        "author": serde_json::Value::Null,
    }));

    assert_eq!(data["TITLE"], "Only A Title");
    assert!(
        data.get("DESCRIPTION").is_none(),
        "a non-string value is dropped rather than stringified"
    );
    assert!(
        data.get("AUTHOR").is_none(),
        "a null is dropped rather than rendered as the word null"
    );
    assert!(data.get("SLUG").is_none(), "an absent key stays absent");
}

#[test]
fn each_timestamp_emits_both_its_iso_and_its_display_form() {
    let data = page_data(&serde_json::json!({
        "title": "Dated",
        "published_at": "2026-03-04T10:00:00Z",
        "updated_at": "2026-07-19T10:00:00Z",
    }));

    assert_eq!(data["DATE_ISO"], "2026-03-04T10:00:00Z");
    assert_eq!(data["DATE_MODIFIED_ISO"], "2026-07-19T10:00:00Z");
    assert!(
        data["DATE"].is_string() && data["DATE"] != data["DATE_ISO"],
        "the display date is formatted, not the raw timestamp: {}",
        data["DATE"]
    );
    assert!(
        data["DATE_MODIFIED"].is_string(),
        "the modified date is formatted too"
    );
}

#[test]
fn an_unparseable_timestamp_keeps_its_iso_slot_and_drops_the_display_one() {
    let data = page_data(&serde_json::json!({
        "title": "Badly Dated",
        "published_at": "not a timestamp",
    }));

    assert_eq!(
        data["DATE_ISO"], "not a timestamp",
        "the raw value is still emitted for the machine-readable slot"
    );
    assert!(
        data.get("DATE").is_none(),
        "no display date is invented from a value that does not parse"
    );
}

#[test]
fn learning_content_is_omitted_wholesale_when_the_doc_has_none() {
    let data = page_data(&serde_json::json!({ "title": "Bare" }));

    for key in [
        "AFTER_READING_THIS",
        "RELATED_PLAYBOOKS",
        "RELATED_CODE",
        "HAS_LEARNING_CONTENT",
    ] {
        assert!(
            data.get(key).is_none(),
            "{key} must be absent, not empty, so the template's if-guard skips the block"
        );
    }
}

#[test]
fn learning_content_sets_its_guard_flag_when_any_list_is_populated() {
    let data = page_data(&serde_json::json!({
        "title": "Rich",
        "after_reading_this": ["Audit a tool call", "Read a trace"],
        "related_playbooks": [{ "title": "Governance", "url": "/documentation/governance" }],
        "related_code": [],
    }));

    assert_eq!(data["HAS_LEARNING_CONTENT"], true);
    assert_eq!(data["AFTER_READING_THIS"][1], "Read a trace");
    assert_eq!(
        data["RELATED_PLAYBOOKS"][0]["url"],
        "/documentation/governance"
    );
    assert!(
        data.get("RELATED_CODE").is_none(),
        "an explicitly empty list is still omitted"
    );
}

#[test]
fn a_malformed_learning_list_degrades_to_absent_rather_than_failing_the_page() {
    let data = page_data(&serde_json::json!({
        "title": "Half Broken",
        "after_reading_this": ["Still readable"],
        "related_playbooks": "not a list at all",
    }));

    assert_eq!(data["AFTER_READING_THIS"][0], "Still readable");
    assert!(
        data.get("RELATED_PLAYBOOKS").is_none(),
        "the unparseable field falls back to its default and is then omitted"
    );
    assert_eq!(
        data["HAS_LEARNING_CONTENT"], true,
        "the surviving field still earns the block"
    );
}

#[test]
fn declared_children_render_into_the_children_slot() {
    let data = page_data(&serde_json::json!({
        "title": "Index",
        "children": [
            {
                "slug": "services/governance",
                "title": "Governance",
                "description": "The four-stage pipeline.",
                "url": "/documentation/services/governance",
            },
        ],
    }));

    let children = data["CHILDREN"]
        .as_str()
        .expect("children render to a markup string");
    assert!(children.contains(r#"href="/documentation/services/governance""#));
    assert!(children.contains("Governance"));
    assert!(children.contains("The four-stage pipeline."));
}

#[test]
fn a_page_with_no_children_omits_the_slot() {
    let data = page_data(&serde_json::json!({ "title": "Leaf" }));

    assert!(
        data.get("CHILDREN").is_none(),
        "a leaf page emits no children markup at all"
    );
}

#[test]
fn the_provider_is_registered_at_the_priority_the_pipeline_expects() {
    let provider = DocsPageDataProvider::default();

    assert_eq!(provider.provider_id(), "docs-metadata");
    assert_eq!(
        provider.priority(),
        60,
        "docs metadata is applied after the site-wide providers"
    );
}
