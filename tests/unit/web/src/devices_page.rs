//! Structural invariants of the `/admin/devices` template.
//!
//! The page's own logic lives inside the admin crate and is exercised by the
//! contract and Playwright suites. What this file guards is the part neither
//! of those can see fail cheaply: a template that renders a destructive
//! control outside its authorisation guard, or a tab that lost the partial
//! that draws it. Both are one deleted line away and both survive a compile.

use crate::support::repo_root;

fn template() -> String {
    let path = repo_root().join("storage/files/admin/templates/devices.hbs");
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn every_revoke_control_sits_inside_the_manage_guard() {
    let source = template();
    let mut depth: i32 = 0;
    let mut guarded = 0_usize;
    for line in source.lines() {
        if line.contains("{{#if can_manage}}")
            || line.contains("{{#if ../can_manage}}")
            || line.contains("{{#if ../../can_manage}}")
        {
            depth += 1;
        }
        if line.contains("data-revoke-id") {
            assert!(
                depth > 0,
                "a revoke control is rendered outside the can_manage guard: {line}"
            );
            guarded += 1;
        }
        if line.contains("{{/if}}") && depth > 0 {
            depth -= 1;
        }
    }
    assert_eq!(
        guarded, 2,
        "expected one revoke control on the token tab and one on the certificate tab"
    );
}

#[test]
fn the_page_draws_all_four_credential_tables() {
    let source = template();
    for collection in [
        "{{#each sessions}}",
        "{{#each pats}}",
        "{{#each certs}}",
        "{{#each links}}",
    ] {
        assert!(
            source.contains(collection),
            "the devices template no longer renders {collection}"
        );
    }
}

#[test]
fn every_credential_table_is_folded_under_a_person() {
    let source = template();
    let groups = source
        .find("{{#each groups}}")
        .expect("the devices template no longer groups rows by person");
    let detail = source
        .find("sp-table__row-detail")
        .expect("a person's row has no detail row to open");
    assert!(
        groups < detail,
        "the detail row sits outside the person loop"
    );
    for collection in [
        "{{#each sessions}}",
        "{{#each pats}}",
        "{{#each certs}}",
        "{{#each links}}",
    ] {
        let at = source.find(collection).unwrap_or(0);
        assert!(
            at > detail,
            "{collection} is rendered outside the person's detail row"
        );
    }
}

#[test]
fn the_version_histogram_is_a_spaced_section() {
    let source = template();
    let section = source
        .find("class=\"sp-section sp-p-devices__estate\"")
        .expect("the versions block is not wrapped in a section, so it has no rhythm above it");
    let list = source
        .find("sp-p-devices__versions")
        .expect("the versions list is gone");
    assert!(section < list, "the versions list sits outside its section");
}

#[test]
fn the_listing_is_paginated_and_has_an_empty_state() {
    let source = template();
    assert!(
        source.contains("pagination=pagination"),
        "the table renders no pagination footer"
    );
    assert!(
        source.contains("components/empty-state"),
        "a filter that matches nothing would render a bare table"
    );
}

#[test]
fn the_version_histogram_labels_itself_for_a_screen_reader() {
    let source = template();
    assert!(
        source.contains("aria-label=\"{{this.devices}} devices on {{this.label}}\""),
        "the SVG bars carry no accessible name, so the histogram is invisible to a reader"
    );
}
