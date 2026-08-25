//! The linear month-end spend projection behind the forecast-overrun warning
//! (REQ-011/014) and the SLO threshold resolution behind the latency split
//! (REQ-029).
//!
//! What is worth pinning is the guard rails, not the arithmetic: no projection
//! exists before the minimum elapsed window (an hour-old month multiplied out
//! is noise, and noise that alerts teaches people to ignore alerts), and an
//! SLO from an edited URL clamps to a sane bound instead of erroring.

use chrono::{TimeZone, Utc};
use systemprompt_web_admin::gateway_org_budget::projected_month_end_spend;
use systemprompt_web_admin::repositories::analytics::site::latency::resolve_slo_ms;

#[test]
fn no_projection_before_three_elapsed_days() {
    let early = Utc.with_ymd_and_hms(2026, 8, 3, 23, 0, 0).unwrap();
    assert_eq!(projected_month_end_spend(1_000_000, early), None);
}

#[test]
fn halfway_through_the_month_doubles_the_spend() {
    let now = Utc.with_ymd_and_hms(2026, 8, 16, 12, 0, 0).unwrap();
    let projected = projected_month_end_spend(10_000_000, now).expect("past the minimum window");
    assert_eq!(projected, 20_000_000);
}

#[test]
fn projection_scales_with_the_elapsed_fraction_not_whole_days() {
    let noon = Utc.with_ymd_and_hms(2026, 8, 10, 12, 0, 0).unwrap();
    let midnight = Utc.with_ymd_and_hms(2026, 8, 10, 0, 0, 0).unwrap();
    let at_noon = projected_month_end_spend(9_500_000, noon).expect("projection");
    let at_midnight = projected_month_end_spend(9_500_000, midnight).expect("projection");
    assert!(
        at_noon < at_midnight,
        "the same spend later in the day projects lower: {at_noon} vs {at_midnight}"
    );
}

#[test]
fn slo_defaults_and_clamps() {
    assert_eq!(resolve_slo_ms(None), 5_000);
    assert_eq!(resolve_slo_ms(Some(2_000)), 2_000);
    assert_eq!(resolve_slo_ms(Some(1)), 500, "floor");
    assert_eq!(resolve_slo_ms(Some(10_000_000)), 60_000, "ceiling");
}
