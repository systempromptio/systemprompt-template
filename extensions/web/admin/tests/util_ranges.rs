//! Time-range window resolution for the audit and analytics pages.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::needless_pass_by_value,
    clippy::redundant_clone,
    reason = "test code: panics are the assertion mechanism and clones keep fixtures readable"
)]

use chrono::{Duration, TimeZone, Utc};
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
