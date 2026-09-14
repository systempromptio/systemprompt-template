//! Calendar-month resolution for the month-end reports.
//!
//! The month a report is scoped to decides which requests it bills, so the
//! boundaries are the part worth pinning: a half-open window that is off by an
//! instant double-counts a request or drops it, and neither shows up as an
//! error — only as a total that does not reconcile.

use chrono::{Datelike, Timelike};
use systemprompt_web_admin::util::month_range::{
    MonthQuery, list_month_options, parse_month_range,
};

fn month(key: &str) -> systemprompt_web_admin::util::month_range::MonthRange {
    parse_month_range(&MonthQuery {
        month: Some(key.to_owned()),
    })
}

#[test]
fn window_starts_at_midnight_on_the_first() {
    let m = month("2026-03");
    assert_eq!(m.from.year(), 2026);
    assert_eq!(m.from.month(), 3);
    assert_eq!(m.from.day(), 1);
    assert_eq!((m.from.hour(), m.from.minute(), m.from.second()), (0, 0, 0));
}

#[test]
fn upper_bound_is_the_next_month_not_the_last_day() {
    let m = month("2026-03");
    assert_eq!(m.to.month(), 4);
    assert_eq!(m.to.day(), 1);
}

#[test]
fn december_rolls_into_the_next_year() {
    let m = month("2025-12");
    assert_eq!((m.to.year(), m.to.month()), (2026, 1));
}

#[test]
fn february_in_a_leap_year_is_29_days() {
    let m = month("2024-02");
    assert_eq!((m.to - m.from).num_days(), 29);
}

#[test]
fn february_in_a_common_year_is_28_days() {
    let m = month("2025-02");
    assert_eq!((m.to - m.from).num_days(), 28);
}

#[test]
fn a_past_month_is_complete() {
    assert!(month("2020-01").is_complete);
}

#[test]
fn key_round_trips_through_the_label() {
    let m = month("2026-03");
    assert_eq!(m.key, "2026-03");
    assert_eq!(m.label, "March 2026");
}

// Anything unparseable falls back rather than erroring: a mistyped month in a
// shared report link must still render a report.
#[test]
fn garbage_falls_back_to_a_complete_month() {
    for raw in ["", "not-a-month", "2026-13", "2026", "abcd-ef"] {
        let m = parse_month_range(&MonthQuery {
            month: Some(raw.to_owned()),
        });
        assert!(
            m.is_complete,
            "'{raw}' should fall back to the last complete month"
        );
    }
}

#[test]
fn absent_month_falls_back_to_a_complete_month() {
    assert!(parse_month_range(&MonthQuery::default()).is_complete);
}

#[test]
fn previous_and_next_are_inverses() {
    let m = month("2026-01");
    let back = m.previous();
    assert_eq!(back.key, "2025-12");
    assert_eq!(back.next().map(|n| n.key), Some("2026-01".to_owned()));
}

// The selector must never offer a month that has not started.
#[test]
fn the_current_month_has_no_next() {
    let now = chrono::Utc::now();
    let current = month(&now.format("%Y-%m").to_string());
    assert!(current.next().is_none());
}

#[test]
fn options_mark_exactly_one_selection() {
    let m = month(
        &(chrono::Utc::now() - chrono::Duration::days(40))
            .format("%Y-%m")
            .to_string(),
    );
    let options = list_month_options(&m);
    assert_eq!(options.iter().filter(|o| o.selected).count(), 1);
    assert!(options.len() > 1);
}

// An out-of-range month is still resolvable — it just matches nothing in the
// picker, which is what the page shows when someone edits the URL by hand.
#[test]
fn options_may_have_no_selection_for_an_old_month() {
    let options = list_month_options(&month("2019-05"));
    assert!(options.iter().all(|o| !o.selected));
}
