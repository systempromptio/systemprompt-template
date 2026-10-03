//! The calendar-month interim over core's fixed-length quota windows: the
//! arithmetic that keeps a monthly window recognisable, day-stable and
//! ending at the month's end, and the normalisation that keeps the daily
//! rewrite out of drift and export.

use chrono::{NaiveDate, TimeZone, Utc};
use systemprompt::gateway::{GatewayPolicySpec, QuotaWindow};
use systemprompt_web_admin::repositories::gateway_policies::month_window::{
    DAY_SECONDS, MONTH_WINDOW_SECONDS, align_window, days_remaining, declared_window_seconds,
    has_month_window, is_month_window, live_window_seconds, month_window_seconds, normalise_spec,
    window_label,
};

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap_or_default()
}

#[test]
fn the_sentinel_is_thirty_one_days_and_is_a_month_window() {
    assert_eq!(MONTH_WINDOW_SECONDS, 31 * DAY_SECONDS);
    assert!(is_month_window(MONTH_WINDOW_SECONDS));
}

#[test]
fn days_remaining_counts_today_and_ends_at_one() {
    assert_eq!(days_remaining(date(2026, 9, 1)), 30);
    assert_eq!(days_remaining(date(2026, 9, 30)), 1);
    assert_eq!(days_remaining(date(2026, 2, 1)), 28);
    assert_eq!(days_remaining(date(2028, 2, 1)), 29, "leap year");
    assert_eq!(days_remaining(date(2026, 12, 31)), 1, "year end");
}

// Why: the live value must be a multiple of a day (so every bucket boundary
// is a midnight and one bucket covers the whole day), and must sit in the
// reserved 32..=62-day band so no hand-set window is mistaken for it.
#[test]
fn the_live_value_is_day_aligned_and_in_the_reserved_band() {
    let mut day = date(2026, 1, 1);
    while day < date(2027, 1, 1) {
        let live = month_window_seconds(day);
        assert_eq!(live % DAY_SECONDS, 0, "{day}");
        let days = live / DAY_SECONDS;
        assert!((32..=62).contains(&days), "{day}: {days} days");
        assert!(is_month_window(live), "{day}");
        assert_eq!(declared_window_seconds(live), MONTH_WINDOW_SECONDS, "{day}");
        day = day.succ_opt().unwrap_or(day);
    }
}

#[test]
fn hand_set_windows_are_never_month_windows() {
    for fixed in [
        60,
        3_600,
        86_400,
        604_800,
        7 * DAY_SECONDS,
        30 * DAY_SECONDS,
        90 * DAY_SECONDS,
    ] {
        assert!(!is_month_window(fixed), "{fixed}");
        assert_eq!(declared_window_seconds(fixed), fixed);
        assert_eq!(live_window_seconds(fixed, date(2026, 9, 17)), fixed);
    }
}

#[test]
fn a_month_window_goes_live_on_the_day_value_and_the_last_day_is_the_shortest() {
    let today = date(2026, 9, 17);
    assert_eq!(
        live_window_seconds(MONTH_WINDOW_SECONDS, today),
        (31 + 14) * DAY_SECONDS
    );
    assert_eq!(month_window_seconds(date(2026, 9, 30)), 32 * DAY_SECONDS);
    assert_eq!(month_window_seconds(date(2026, 1, 1)), 62 * DAY_SECONDS);
}

// Why: with a day-aligned window every moment of one UTC day lands in the
// same bucket — the invariant the carry-forward relies on.
#[test]
fn one_utc_day_falls_in_one_bucket_for_a_live_value() {
    let today = date(2026, 9, 17);
    let live = month_window_seconds(today);
    let start = Utc
        .with_ymd_and_hms(2026, 9, 17, 0, 0, 0)
        .single()
        .unwrap_or_default();
    let end = Utc
        .with_ymd_and_hms(2026, 9, 17, 23, 59, 59)
        .single()
        .unwrap_or_default();
    assert_eq!(align_window(start, live), align_window(end, live));
}

#[test]
fn align_window_reproduces_core_epoch_alignment() {
    let now = Utc
        .with_ymd_and_hms(2026, 9, 17, 10, 42, 7)
        .single()
        .unwrap_or_default();
    let hour = align_window(now, 3_600);
    assert_eq!(
        hour,
        Utc.with_ymd_and_hms(2026, 9, 17, 10, 0, 0)
            .single()
            .unwrap_or_default()
    );
    let day = align_window(now, 86_400);
    assert_eq!(
        day,
        Utc.with_ymd_and_hms(2026, 9, 17, 0, 0, 0)
            .single()
            .unwrap_or_default()
    );
}

fn spec_with(window_seconds: i32) -> GatewayPolicySpec {
    GatewayPolicySpec {
        quota_windows: vec![
            QuotaWindow {
                window_seconds,
                subject: "organization".to_owned(),
                max_cost_microdollars: Some(200_000_000),
                ..QuotaWindow::default()
            },
            QuotaWindow {
                window_seconds: 3_600,
                subject: "user".to_owned(),
                max_requests: Some(600),
                ..QuotaWindow::default()
            },
        ],
        ..GatewayPolicySpec::default()
    }
}

#[test]
fn normalising_folds_the_live_value_back_to_the_sentinel_and_leaves_fixed_windows() {
    let live = spec_with(month_window_seconds(date(2026, 9, 17)));
    assert!(has_month_window(&live));
    let normalised = normalise_spec(&live);
    assert_eq!(
        normalised.quota_windows[0].window_seconds,
        MONTH_WINDOW_SECONDS
    );
    assert_eq!(normalised.quota_windows[1].window_seconds, 3_600);
    assert_eq!(
        normalised.quota_windows[0].max_cost_microdollars,
        Some(200_000_000),
        "ceilings survive normalisation"
    );
    assert!(!has_month_window(&spec_with(86_400)));
}

#[test]
fn labels_name_the_period_and_count_the_days_left_in_a_month() {
    let today = date(2026, 9, 17);
    assert_eq!(window_label(3_600, today), "hour");
    assert_eq!(window_label(86_400, today), "day");
    assert_eq!(window_label(604_800, today), "week");
    assert_eq!(
        window_label(MONTH_WINDOW_SECONDS, today),
        "calendar month (14 days left)"
    );
    assert_eq!(
        window_label(month_window_seconds(date(2026, 9, 30)), date(2026, 9, 30)),
        "calendar month (1 day left)"
    );
    assert_eq!(window_label(2 * DAY_SECONDS, today), "2 days");
    assert_eq!(window_label(7_200, today), "2 hours");
    assert_eq!(window_label(90, today), "90 s");
}
