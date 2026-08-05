//! The docs page provider turns a content item into template context. The
//! load-bearing part is `render_children_cards`, which writes raw HTML into the
//! page: every child field is attacker-controlled content-pipeline text, so all
//! three must go through `html_escape` or a doc title becomes a script tag.
//! `str_field` and `parse_children` guard the same boundary by refusing to
//! coerce non-string / non-array JSON into page data.

use serde_json::json;
use systemprompt::extension::prelude::PageDataProvider;
use systemprompt_web_site::docs::provider::{parse_children, str_field};
use systemprompt_web_site::docs::{ChildDoc, DocsPageDataProvider};

fn child(title: &str, description: &str, url: &str) -> ChildDoc {
    ChildDoc {
        slug: "child".to_owned(),
        title: title.to_owned(),
        description: description.to_owned(),
        url: url.to_owned(),
    }
}

#[test]
fn str_field_reads_only_json_strings() {
    let item = json!({ "title": "Governance", "count": 3, "flag": true, "nested": {} });

    assert_eq!(str_field(&item, "title").as_deref(), Some("Governance"));
    assert!(str_field(&item, "count").is_none());
    assert!(str_field(&item, "flag").is_none());
    assert!(str_field(&item, "nested").is_none());
    assert!(str_field(&item, "absent").is_none());
}

#[test]
fn parse_children_returns_the_declared_child_docs() {
    let children = parse_children(&json!({
        "children": [
            { "slug": "a", "title": "A", "description": "First", "url": "/docs/a" },
            { "slug": "b", "title": "B", "description": "Second", "url": "/docs/b" },
        ],
    }));

    assert_eq!(children.len(), 2);
    assert_eq!(children[1].url, "/docs/b");
}

#[test]
fn parse_children_yields_nothing_when_the_key_is_absent_or_malformed() {
    assert!(parse_children(&json!({})).is_empty());
    assert!(parse_children(&json!({ "children": "not a list" })).is_empty());
    assert!(parse_children(&json!({ "children": [{ "slug": "a" }] })).is_empty());
}

#[test]
fn no_children_renders_no_card_markup() {
    assert!(DocsPageDataProvider::render_children_cards(&[]).is_none());
}

#[test]
fn each_child_renders_one_anchor_joined_by_a_newline() {
    let html = DocsPageDataProvider::render_children_cards(&[
        child("First", "One", "/docs/one"),
        child("Second", "Two", "/docs/two"),
    ])
    .expect("two children render markup");

    assert_eq!(html.matches("class=\"docs-card\"").count(), 2);
    assert_eq!(html.lines().filter(|l| l.starts_with("<a ")).count(), 2);
    assert!(html.contains("href=\"/docs/one\""));
    assert!(html.contains("<h3 class=\"docs-card-title\">Second</h3>"));
}

#[test]
fn every_child_field_is_html_escaped() {
    let html = DocsPageDataProvider::render_children_cards(&[child(
        "<script>alert(1)</script>",
        "Tom & \"Jerry\"",
        "/docs/a?x=1&y=2",
    )])
    .expect("one child renders markup");

    assert!(
        !html.contains("<script>"),
        "an unescaped child title would inject markup: {html}"
    );
    assert!(html.contains("&lt;script&gt;"));
    assert!(html.contains("Tom &amp; "));
    assert!(html.contains("/docs/a?x=1&amp;y=2"));
}

#[test]
fn the_provider_claims_every_documentation_page_type() {
    let provider = DocsPageDataProvider::new();

    assert_eq!(provider.provider_id(), "docs-metadata");
    assert_eq!(provider.priority(), 60);

    let mut pages = provider.applies_to_pages();
    pages.sort();
    assert_eq!(
        pages,
        vec!["docs", "docs-page", "guide", "reference", "tutorial"]
    );
}
