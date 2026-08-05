//! `ContentKind` is the bridge between a markdown front-matter string and a
//! typed kind, so `as_str` and `FromStr` must stay inverse — with the two
//! deliberate asymmetries pinned here: parsing is case-insensitive, and the
//! legacy `page` spelling still resolves to `Legal`.

use std::str::FromStr;
use systemprompt_web_shared::models::{ContentKind, IngestionOptions, IngestionReport};

const ALL_KINDS: [ContentKind; 10] = [
    ContentKind::Blog,
    ContentKind::Guide,
    ContentKind::Tutorial,
    ContentKind::Reference,
    ContentKind::DocsIndex,
    ContentKind::Docs,
    ContentKind::DocsList,
    ContentKind::Feature,
    ContentKind::Playbook,
    ContentKind::Legal,
];

#[test]
fn every_kind_round_trips_through_its_wire_string() {
    for kind in ALL_KINDS {
        assert_eq!(ContentKind::from_str(kind.as_str()).unwrap(), kind);
        assert_eq!(kind.to_string(), kind.as_str());
    }
}

#[test]
fn the_hyphenated_kinds_keep_their_hyphens() {
    assert_eq!(ContentKind::DocsIndex.as_str(), "docs-index");
    assert_eq!(ContentKind::DocsList.as_str(), "docs-list");
    assert_eq!(
        ContentKind::from_str("docs-list").unwrap(),
        ContentKind::DocsList
    );
}

#[test]
fn parsing_is_case_insensitive() {
    assert_eq!(ContentKind::from_str("BLOG").unwrap(), ContentKind::Blog);
    assert_eq!(
        ContentKind::from_str("Docs-Index").unwrap(),
        ContentKind::DocsIndex
    );
}

#[test]
fn the_legacy_page_spelling_still_resolves_to_legal() {
    assert_eq!(ContentKind::from_str("page").unwrap(), ContentKind::Legal);
}

#[test]
fn unknown_kinds_are_rejected_and_quoted_in_the_error() {
    let err = ContentKind::from_str("newsletter").unwrap_err();
    assert!(
        err.contains("newsletter"),
        "error should quote input: {err}"
    );
    assert!(ContentKind::from_str("").is_err());
}

#[test]
fn content_defaults_to_blog() {
    assert_eq!(ContentKind::default(), ContentKind::Blog);
}

#[test]
fn a_new_ingestion_report_is_zeroed_and_counts_as_success() {
    let report = IngestionReport::new();
    assert_eq!(report.files_found, 0);
    assert_eq!(report.files_processed, 0);
    assert_eq!(report.orphans_deleted, 0);
    assert!(report.errors.is_empty());
    assert!(report.is_success());
    assert!(IngestionReport::default().is_success());
}

#[test]
fn a_report_with_any_error_is_not_a_success_even_when_files_were_processed() {
    let report = IngestionReport {
        files_found: 10,
        files_processed: 9,
        orphans_deleted: 1,
        errors: vec!["bad front matter in a.md".to_owned()],
    };
    assert!(!report.is_success());
}

#[test]
fn ingestion_options_default_to_all_flags_off() {
    let options = IngestionOptions::default();
    assert!(!options.override_existing);
    assert!(!options.recursive);
    assert!(!options.delete_orphans);
}

#[test]
fn the_with_builders_set_each_flag_independently() {
    let options = IngestionOptions::default()
        .with_override(true)
        .with_recursive(true)
        .with_delete_orphans(true);
    assert!(options.override_existing);
    assert!(options.recursive);
    assert!(options.delete_orphans);

    let only_recursive = IngestionOptions::default().with_recursive(true);
    assert!(only_recursive.recursive);
    assert!(!only_recursive.override_existing);
    assert!(!only_recursive.delete_orphans);
}
