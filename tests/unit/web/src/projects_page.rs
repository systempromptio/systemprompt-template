//! Structural invariants of the two project templates.
//!
//! The handlers are exercised by the contract and Playwright suites. What this
//! file guards is what neither of those catches cheaply: a write control that
//! drifts outside its authorisation guard, a table that loses its empty state,
//! and — the one that would be read as a data bug rather than a markup bug —
//! the member table losing the sentence that says it is the one table on the
//! page counted by membership rather than exclusively.

use crate::support::repo_root;

fn template(name: &str) -> String {
    let path = repo_root().join("storage/files/admin/templates").join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

fn listing() -> String {
    template("projects.hbs")
}

fn detail() -> String {
    template("project-detail.hbs")
}

#[test]
fn every_listing_column_is_a_sort_header() {
    let source = listing();
    for column in [
        "sort_headers.name",
        "sort_headers.members",
        "sort_headers.groups",
        "sort_headers.requests",
        "sort_headers.cost",
        "sort_headers.tools",
        "sort_headers.skills",
    ] {
        assert!(
            source.contains(column),
            "the projects listing no longer sorts by {column}"
        );
    }
}

#[test]
fn the_listing_is_paginated_and_has_an_empty_state() {
    let source = listing();
    assert!(
        source.contains("pagination=pagination"),
        "the projects table renders no pagination footer"
    );
    assert!(
        source.contains("sp-table__empty"),
        "the projects table has no empty row"
    );
}

#[test]
fn the_create_control_sits_inside_the_manage_guard() {
    assert_guarded(&listing(), "new-project", 1);
}

#[test]
fn every_detail_write_control_sits_inside_a_guard() {
    let source = detail();
    for control in [
        "save-project",
        "delete-project",
        "add-mapping",
        "remove-mapping",
    ] {
        assert_guarded(&source, control, 1);
    }
}

// Why: the members table is the only member-attributed view in the console.
// Without the label it reads as disagreeing with the tiles above it, which is
// a correctness claim about the data rather than a missing sentence.
#[test]
fn the_member_table_declares_its_attribution() {
    let source = detail();
    assert!(
        source.contains("Member view"),
        "the members section no longer says it is a member-attributed view"
    );
}

#[test]
fn the_detail_page_draws_all_three_tabs() {
    let source = detail();
    for guard in ["{{#if members}}", "{{#if usage}}", "{{#if settings}}"] {
        assert!(
            source.contains(guard),
            "the detail page lost the {guard} body"
        );
    }
    assert!(
        source.contains("components/tabs"),
        "the detail page renders no tab strip"
    );
}

#[test]
fn every_usage_table_has_an_empty_state() {
    let source = detail();
    // One per table: models, skills, tools, sessions, commits, members,
    // mappings and gated entities.
    assert_eq!(
        source.matches("sp-table__empty").count(),
        8,
        "a table on the project pages lost its empty row"
    );
}

#[test]
fn every_table_carries_a_caption() {
    for source in [listing(), detail()] {
        let tables = source.matches("{{#> components/table").count();
        let captions = source.matches("caption=").count();
        assert_eq!(
            tables, captions,
            "a table on the project pages has no caption for a screen reader"
        );
    }
}

// Assert that every occurrence of `control` is inside an `{{#if}}` guard, and
// that there are exactly `expected` of them.
fn assert_guarded(source: &str, control: &str, expected: usize) {
    let needle = format!("data-action=\"{control}\"");
    let mut depth: i32 = 0;
    let mut guarded = 0_usize;
    for line in source.lines() {
        let opens = line.matches("{{#if ").count();
        let closes = line.matches("{{/if}}").count();
        depth += i32::try_from(opens).unwrap_or(0);
        if line.contains(&needle) {
            assert!(
                depth > 0,
                "{control} is rendered outside every guard: {line}"
            );
            guarded += 1;
        }
        depth -= i32::try_from(closes).unwrap_or(0);
        depth = depth.max(0);
    }
    assert_eq!(guarded, expected, "unexpected number of {control} controls");
}
