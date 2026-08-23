#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: panics are the assertion mechanism"
)]

use chrono::{TimeZone, Utc};
use systemprompt_web_admin::gateway_org_budget::seconds_until_next_month;

fn at(y: i32, m: u32, d: u32, hh: u32, mm: u32, ss: u32) -> chrono::DateTime<Utc> {
    Utc.with_ymd_and_hms(y, m, d, hh, mm, ss)
        .single()
        .expect("valid UTC instant")
}

#[test]
fn mid_month_counts_to_the_next_month_start() {
    // 2026-08-22 10:15:52 UTC — the instant from the incident this hint exists for.
    let hint = seconds_until_next_month(at(2026, 8, 22, 10, 15, 52));
    // Aug 22 -> Sep 1 is 10 days, less the elapsed part of the 22nd.
    assert_eq!(hint, 10 * 86_400 - (10 * 3_600 + 15 * 60 + 52));
}

#[test]
fn the_last_second_of_a_month_is_one_second() {
    assert_eq!(seconds_until_next_month(at(2026, 8, 31, 23, 59, 59)), 1);
}

#[test]
fn december_rolls_the_year_over() {
    assert_eq!(seconds_until_next_month(at(2026, 12, 31, 23, 0, 0)), 3_600);
    assert_eq!(
        seconds_until_next_month(at(2026, 12, 1, 0, 0, 0)),
        31 * 86_400
    );
}

#[test]
fn february_length_follows_the_leap_year() {
    assert_eq!(
        seconds_until_next_month(at(2028, 2, 1, 0, 0, 0)),
        29 * 86_400
    );
    assert_eq!(
        seconds_until_next_month(at(2027, 2, 1, 0, 0, 0)),
        28 * 86_400
    );
}

#[test]
fn the_first_instant_of_a_month_is_the_whole_month() {
    assert_eq!(
        seconds_until_next_month(at(2026, 8, 1, 0, 0, 0)),
        31 * 86_400
    );
}
