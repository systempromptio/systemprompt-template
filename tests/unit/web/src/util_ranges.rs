//! Window resolution for the audit and usage-report pages.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    clippy::separated_literal_suffix,
    reason = "test code: panics are the assertion mechanism"
)]

use chrono::{Datelike, Duration, Months, TimeZone, Utc};
use systemprompt_web_admin::util::month_range::{
    MonthQuery, list_month_options, parse_month_range,
};
use systemprompt_web_admin::util::time_range::{
    TimeRangePreset, TimeRangeQuery, parse_time_range, preset_to_range,
};

fn query(from: Option<&str>, to: Option<&str>, preset: Option<&str>) -> TimeRangeQuery {
    TimeRangeQuery {
        from: from.map(ToOwned::to_owned),
        to: to.map(ToOwned::to_owned),
        preset: preset.map(ToOwned::to_owned),
    }
}

#[test]
fn every_preset_string_resolves_to_its_own_duration() {
    let cases = [
        ("15m", TimeRangePreset::Min15, Duration::minutes(15)),
        ("1h", TimeRangePreset::Hour1, Duration::hours(1)),
        ("24h", TimeRangePreset::Hours24, Duration::hours(24)),
        ("7d", TimeRangePreset::Days7, Duration::days(7)),
        ("30d", TimeRangePreset::Days30, Duration::days(30)),
    ];
    for (text, expected, span) in cases {
        let range = parse_time_range(&query(None, None, Some(text)));
        assert_eq!(range.preset, expected, "{text}");
        let width = range.to - range.from;
        assert!(
            (width - span).num_seconds().abs() <= 1,
            "{text}: {width} vs {span}"
        );
    }
}

#[test]
fn the_default_window_is_the_last_24_hours() {
    let range = parse_time_range(&query(None, None, None));
    assert_eq!(range.preset, TimeRangePreset::Hours24);
    assert!(
        (range.to - range.from - Duration::hours(24))
            .num_seconds()
            .abs()
            <= 1
    );
}

#[test]
fn an_unrecognised_preset_falls_back_to_the_default_window() {
    let range = parse_time_range(&query(None, None, Some("fortnight")));
    assert_eq!(range.preset, TimeRangePreset::Hours24);
}

#[test]
fn the_custom_preset_alone_carries_no_duration_and_falls_back() {
    let range = parse_time_range(&query(None, None, Some("custom")));
    assert_eq!(range.preset, TimeRangePreset::Hours24);
}

#[test]
fn a_complete_from_and_to_pair_is_taken_verbatim_as_custom() {
    let range = parse_time_range(&query(
        Some("2026-03-01T00:00:00Z"),
        Some("2026-03-08T00:00:00Z"),
        None,
    ));
    assert_eq!(range.preset, TimeRangePreset::Custom);
    assert_eq!(
        range.from,
        Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap()
    );
    assert_eq!(range.to, Utc.with_ymd_and_hms(2026, 3, 8, 0, 0, 0).unwrap());
}

#[test]
fn a_preset_wins_over_explicit_bounds() {
    let range = parse_time_range(&query(
        Some("2026-03-01T00:00:00Z"),
        Some("2026-03-08T00:00:00Z"),
        Some("1h"),
    ));
    assert_eq!(range.preset, TimeRangePreset::Hour1);
}

#[test]
fn a_half_or_malformed_bound_pair_falls_back_rather_than_erroring() {
    for (from, to) in [
        (Some("2026-03-01T00:00:00Z"), None),
        (None, Some("2026-03-08T00:00:00Z")),
        (Some("yesterday"), Some("today")),
        (Some("2026-03-01T00:00:00Z"), Some("not-a-date")),
    ] {
        let range = parse_time_range(&query(from, to, None));
        assert_eq!(range.preset, TimeRangePreset::Hours24, "{from:?}..{to:?}");
    }
}

// Why: the fallback is fine; being quiet about it is not. A URL naming a
// window the parser cannot read renders the default, and the flag below is
// what the page reads to say so. Without it a link to a window in the year
// 2000 shows the last 24 hours and claims nothing happened.
#[test]
fn a_window_that_could_not_be_read_is_marked_as_rejected() {
    let cases = [
        (None, None, Some("fortnight"), "an unknown preset"),
        (Some("yesterday"), Some("today"), None, "unparseable bounds"),
        (
            Some("2026-03-01T00:00:00Z"),
            Some("not-a-date"),
            None,
            "one unparseable bound",
        ),
        (
            Some("2026-03-01T00:00:00Z"),
            None,
            None,
            "a lone bound, which is not yet a range",
        ),
        (
            None,
            Some("2026-03-08T00:00:00Z"),
            None,
            "a lone upper bound",
        ),
    ];
    for (from, to, preset, label) in cases {
        let range = parse_time_range(&query(from, to, preset));
        assert_eq!(range.preset, TimeRangePreset::Hours24, "{label}");
        assert!(
            range.rejected_bounds,
            "{label}: the page is showing the default and must be able to say so"
        );
    }
}

#[test]
fn a_window_the_query_actually_got_is_not_marked_as_rejected() {
    let cases = [
        (None, None, None, "no window asked for"),
        (None, None, Some("7d"), "a preset that resolves"),
        (
            Some("2026-03-01T00:00:00Z"),
            Some("2026-03-08T00:00:00Z"),
            None,
            "a complete custom pair",
        ),
        (
            Some("2026-03-01T00:00:00Z"),
            Some("2026-03-08T00:00:00Z"),
            Some("1h"),
            "a preset that overrides the bounds it was given",
        ),
    ];
    for (from, to, preset, label) in cases {
        let range = parse_time_range(&query(from, to, preset));
        assert!(!range.rejected_bounds, "{label}");
    }
}

// Why: `preset=custom` with no bounds is the time-range partial's own "Custom"
// button before a date is picked. It is a control in its initial state, not a
// window that failed, so it must not raise the notice.
#[test]
fn the_custom_button_before_a_date_is_picked_is_not_a_rejection() {
    let range = parse_time_range(&query(None, None, Some("custom")));
    assert!(!range.rejected_bounds);
}

#[test]
fn preset_to_range_matches_what_parse_would_have_produced() {
    let parsed = parse_time_range(&query(None, None, Some("7d")));
    let direct = preset_to_range(TimeRangePreset::Days7);
    assert_eq!(direct.preset, parsed.preset);
    assert!((direct.from - parsed.from).num_seconds().abs() <= 1);
}

#[test]
fn preset_to_range_of_custom_falls_back_to_a_24_hour_window() {
    let range = preset_to_range(TimeRangePreset::Custom);
    assert_eq!(range.preset, TimeRangePreset::Custom);
    assert!(
        (range.to - range.from - Duration::hours(24))
            .num_seconds()
            .abs()
            <= 1
    );
}

fn month(key: &str) -> systemprompt_web_admin::util::month_range::MonthRange {
    parse_month_range(&MonthQuery {
        month: Some(key.to_owned()),
    })
}

#[test]
fn a_month_key_resolves_to_that_calendar_month() {
    let range = month("2026-02");
    assert_eq!(range.key, "2026-02");
    assert_eq!(range.label, "February 2026");
    assert_eq!(
        range.from,
        Utc.with_ymd_and_hms(2026, 2, 1, 0, 0, 0).unwrap()
    );
    assert_eq!(range.to, Utc.with_ymd_and_hms(2026, 3, 1, 0, 0, 0).unwrap());
    assert!(range.is_complete);
}

#[test]
fn february_is_28_or_29_days_not_a_rolling_30() {
    assert_eq!((month("2026-02").to - month("2026-02").from).num_days(), 28);
    assert_eq!((month("2024-02").to - month("2024-02").from).num_days(), 29);
    assert_eq!((month("2026-01").to - month("2026-01").from).num_days(), 31);
}

#[test]
fn an_absent_or_unparseable_month_falls_back_to_the_last_complete_one() {
    let expected = {
        let now = Utc::now();
        let first = Utc
            .with_ymd_and_hms(now.year(), now.month(), 1, 0, 0, 0)
            .unwrap()
            - Months::new(1);
        first.format("%Y-%m").to_string()
    };
    for raw in [
        None,
        Some("nonsense"),
        Some("2026"),
        Some("2026-13"),
        Some("abcd-01"),
    ] {
        let range = parse_month_range(&MonthQuery {
            month: raw.map(ToOwned::to_owned),
        });
        assert_eq!(range.key, expected, "{raw:?}");
        assert!(range.is_complete, "{raw:?}");
    }
}

#[test]
fn the_month_in_progress_is_marked_incomplete() {
    let now = Utc::now();
    let range = month(&now.format("%Y-%m").to_string());
    assert!(!range.is_complete);
}

#[test]
fn previous_and_next_walk_the_calendar() {
    let march = month("2026-03");
    assert_eq!(march.previous().key, "2026-02");
    assert_eq!(march.next().expect("april has started").key, "2026-04");

    let january = month("2026-01");
    assert_eq!(january.previous().key, "2025-12");
}

#[test]
fn next_stops_at_the_month_currently_in_progress() {
    let now = Utc::now();
    let current = month(&now.format("%Y-%m").to_string());
    assert!(current.next().is_none());
}

#[test]
fn a_month_range_converts_to_the_equivalent_custom_time_range() {
    let range = month("2026-02");
    let as_time = range.as_time_range();
    assert_eq!(as_time.preset, TimeRangePreset::Custom);
    assert_eq!(as_time.from, range.from);
    assert_eq!(as_time.to, range.to);
}

#[test]
fn the_picker_lists_thirteen_months_newest_first_with_one_selected() {
    let selected = parse_month_range(&MonthQuery { month: None });
    let options = list_month_options(&selected);
    assert_eq!(options.len(), 13);
    assert_eq!(options.iter().filter(|o| o.selected).count(), 1);
    assert!(options[0].key > options[12].key, "newest first");
    assert!(
        options[1].selected,
        "the last complete month sits below the current one"
    );
}
