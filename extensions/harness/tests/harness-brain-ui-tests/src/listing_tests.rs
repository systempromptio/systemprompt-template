//! The fact list: paging, superseded reads, failures and filters.

use harness_brain_ui::model::{BrainFailure, FactFilter, FactListing, Load, PAGE_SIZE};
use sp_protocol::wire::brain::{BrainFactId, FactStatus};

use crate::fixtures::{FACT_A, FACT_B, FACT_C, page};

fn ids(listing: &FactListing) -> Vec<String> {
    listing
        .facts
        .ready()
        .expect("the listing is ready")
        .iter()
        .map(|fact| fact.id.as_str().to_owned())
        .collect()
}

#[test]
fn first_page_replaces_and_next_page_appends_without_duplicates() {
    let mut listing = FactListing::default();
    let first = listing.begin(false);
    assert!(listing.facts.is_loading());
    assert!(listing.apply(first, page(&[FACT_A, FACT_B], Some("cursor-1"))));
    assert_eq!(listing.next.as_deref(), Some("cursor-1"));

    let second = listing.begin(true);
    assert!(listing.appending);
    assert!(listing.apply(second, page(&[FACT_B, FACT_C], None)));
    assert_eq!(ids(&listing), [FACT_A, FACT_B, FACT_C]);
    assert_eq!(listing.next, None);
    assert!(!listing.appending);
}

#[test]
fn an_answer_to_a_superseded_read_is_dropped() {
    let mut listing = FactListing::default();
    let stale = listing.begin(false);
    let current = listing.begin(false);
    assert!(!listing.apply(stale, page(&[FACT_A], None)));
    assert!(!listing.fail(stale, BrainFailure::Unreachable));
    assert!(listing.facts.is_loading());
    assert!(listing.apply(current, page(&[FACT_B], None)));
    assert_eq!(ids(&listing), [FACT_B]);
}

#[test]
fn a_refresh_keeps_the_shown_facts_until_it_answers() {
    let mut listing = FactListing::default();
    let first = listing.begin(false);
    listing.apply(first, page(&[FACT_A], None));
    listing.begin(false);
    assert_eq!(ids(&listing), [FACT_A]);
}

#[test]
fn a_failed_first_page_fails_the_list_but_a_failed_next_page_keeps_it() {
    let mut listing = FactListing::default();
    let first = listing.begin(false);
    listing.apply(first, page(&[FACT_A], Some("cursor-1")));
    let more = listing.begin(true);
    assert!(listing.fail(more, BrainFailure::RateLimited));
    assert_eq!(listing.more_failed, Some(BrainFailure::RateLimited));
    assert_eq!(ids(&listing), [FACT_A]);
    assert_eq!(listing.next.as_deref(), Some("cursor-1"));

    let mut empty = FactListing::default();
    let read = empty.begin(false);
    assert!(empty.fail(read, BrainFailure::Forbidden));
    assert_eq!(empty.facts, Load::Failed(BrainFailure::Forbidden));
}

#[test]
fn a_new_filter_restarts_the_listing_and_clear_keeps_the_filter() {
    let mut listing = FactListing::default();
    let first = listing.begin(false);
    listing.apply(first, page(&[FACT_A], Some("cursor-1")));
    let filter = FactFilter {
        statuses: vec![FactStatus::Disputed],
        ..FactFilter::default()
    };
    assert!(listing.set_filter(filter.clone()));
    assert!(!listing.set_filter(filter.clone()));
    assert!(listing.facts.is_loading());
    assert_eq!(listing.next, None);

    listing.selected = Some(BrainFactId::try_new(FACT_A).expect("a fact id"));
    listing.clear();
    assert_eq!(listing.filter, filter);
    assert_eq!(listing.facts, Load::Idle);
    assert_eq!(listing.selected, None);
}

#[test]
fn the_filter_parses_typed_values_and_builds_the_query() {
    assert_eq!(
        FactFilter::parse_kind("  ").expect("empty is no filter"),
        None
    );
    assert_eq!(
        FactFilter::parse_kind("decision")
            .expect("a kind")
            .map(|kind| kind.as_str().to_owned()),
        Some("decision".to_owned())
    );
    assert!(FactFilter::parse_kind("Not A Kind").is_err());
    let subject = FactFilter::parse_subject(" client:acme ")
        .expect("a subject")
        .expect("present");
    assert_eq!(subject.to_string(), "client:acme");
    assert!(FactFilter::parse_subject("acme").is_err());

    let mut filter = FactFilter::default();
    assert!(filter.is_empty());
    filter.toggle_status(FactStatus::Stale);
    filter.toggle_status(FactStatus::Candidate);
    assert_eq!(filter.statuses, [FactStatus::Candidate, FactStatus::Stale]);
    filter.toggle_status(FactStatus::Stale);
    assert_eq!(filter.statuses, [FactStatus::Candidate]);

    let query = filter.query(Some("cursor-2".to_owned()));
    assert_eq!(query.limit, Some(PAGE_SIZE));
    assert_eq!(query.after.as_deref(), Some("cursor-2"));
    assert_eq!(query.statuses, [FactStatus::Candidate]);
}
