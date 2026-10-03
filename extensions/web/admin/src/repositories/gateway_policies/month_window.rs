//! Calendar-month quota windows over core's fixed-length buckets — the
//! arithmetic, with no database in it.
//!
//! Core keys a quota bucket on `(subject, window_seconds, window_start)` with
//! `window_start = floor(now / window_seconds) * window_seconds`: every window
//! is a fixed length aligned to the Unix epoch, and "this calendar month" is
//! not expressible. This module is the interim until core grows calendar
//! windows, and it works like this:
//!
//! * The **declaration** — the file and the console editor — marks a monthly
//!   window with [`MONTH_WINDOW_SECONDS`] (31 days), the longest a month can
//!   be.
//! * The **database** row core reads never carries that sentinel. A daily job
//!   ([`super::month_window_db`]) rewrites each monthly window to
//!   [`month_window_seconds`]: `86400 × (31 + days left in the month, today
//!   included)`. A multiple of a day keeps every bucket boundary at a UTC
//!   midnight, so one bucket covers the whole of today; the `+ 31` places every
//!   live value in `32..=62` days, a band no hand-set window uses, which is how
//!   [`is_month_window`] tells a monthly window from a fixed one without a
//!   marker the core spec would reject.
//! * A new day is a new `window_seconds` and therefore a **new bucket**, so the
//!   job carries yesterday's counters into today's bucket before the policy row
//!   changes. The bucket the gateway reserves against then holds month-to-date
//!   usage, and the first of the month starts from zero.
//!
//! [`normalise_spec`] folds any live value back to the sentinel so drift and
//! export compare and render the declaration, never the day's rewrite.

use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use systemprompt::gateway::{GatewayPolicySpec, QuotaWindow};

pub const DAY_SECONDS: i32 = 86_400;

// Why: The declared marker for "one calendar month": 31 days.
pub const MONTH_WINDOW_SECONDS: i32 = 31 * DAY_SECONDS;

// Why: the live band — 32 to 62 days. Below it are the fixed windows an
// operator writes by hand (an hour, a day, a week, the sentinel itself);
// nothing legitimate is longer than a month and shorter than two.
const LIVE_MIN_DAYS: i32 = 32;
const LIVE_MAX_DAYS: i32 = 62;

#[must_use]
pub fn days_in_month(date: NaiveDate) -> u32 {
    let (year, month) = if date.month() == 12 {
        (date.year() + 1, 1)
    } else {
        (date.year(), date.month() + 1)
    };
    // Why: discard-ok: the first of the following month always exists
    NaiveDate::from_ymd_opt(year, month, 1)
        .and_then(|next| {
            next.signed_duration_since(date.with_day(1)?)
                .num_days()
                .try_into()
                .ok()
        })
        .unwrap_or(31)
}

// Why: Days left in `date`'s month, `date` included: 1 on the last day.
#[must_use]
pub fn days_remaining(date: NaiveDate) -> u32 {
    days_in_month(date) - date.day() + 1
}

// Why: The `window_seconds` a monthly window carries in the database on `date`.
#[must_use]
pub fn month_window_seconds(date: NaiveDate) -> i32 {
    let remaining = i32::try_from(days_remaining(date)).unwrap_or(1);
    (31 + remaining) * DAY_SECONDS
}

// Why: Whether a `window_seconds` is the month sentinel or a value the daily
// rewrite produced.
#[must_use]
pub fn is_month_window(window_seconds: i32) -> bool {
    if window_seconds == MONTH_WINDOW_SECONDS {
        return true;
    }
    if window_seconds % DAY_SECONDS != 0 {
        return false;
    }
    (LIVE_MIN_DAYS..=LIVE_MAX_DAYS).contains(&(window_seconds / DAY_SECONDS))
}

// Why: Core's own bucket alignment, reproduced so the console reads the bucket
// the gateway is writing to.
#[must_use]
pub fn align_window(now: DateTime<Utc>, window_seconds: i32) -> DateTime<Utc> {
    let secs = now.timestamp();
    let w = i64::from(window_seconds.max(1));
    let aligned = (secs / w) * w;
    Utc.timestamp_opt(aligned, 0).single().unwrap_or(now)
}

// Why: The `window_seconds` a window should carry in the database on `date`:
// the day's live value for a monthly window, the value itself otherwise.
#[must_use]
pub fn live_window_seconds(window_seconds: i32, date: NaiveDate) -> i32 {
    if is_month_window(window_seconds) {
        month_window_seconds(date)
    } else {
        window_seconds
    }
}

// Why: The declared form of a live value: the sentinel for any monthly window.
#[must_use]
pub fn declared_window_seconds(window_seconds: i32) -> i32 {
    if is_month_window(window_seconds) {
        MONTH_WINDOW_SECONDS
    } else {
        window_seconds
    }
}

fn normalise_window(window: &QuotaWindow) -> QuotaWindow {
    QuotaWindow {
        window_seconds: declared_window_seconds(window.window_seconds),
        subject: window.subject.clone(),
        max_requests: window.max_requests,
        max_input_tokens: window.max_input_tokens,
        max_output_tokens: window.max_output_tokens,
        max_cost_microdollars: window.max_cost_microdollars,
    }
}

// Why: A spec with every monthly window folded back to the sentinel — what the
// file declares, whatever day the row was last rewritten on.
#[must_use]
pub fn normalise_spec(spec: &GatewayPolicySpec) -> GatewayPolicySpec {
    GatewayPolicySpec {
        quota_mode: spec.quota_mode,
        quota_windows: spec.quota_windows.iter().map(normalise_window).collect(),
        safety: spec.safety.clone(),
    }
}

// Why: Whether any window in the spec is a monthly one.
#[must_use]
pub fn has_month_window(spec: &GatewayPolicySpec) -> bool {
    spec.quota_windows
        .iter()
        .any(|w| is_month_window(w.window_seconds))
}

// Why: The window as the console prints it.
#[must_use]
pub fn window_label(window_seconds: i32, today: NaiveDate) -> String {
    if is_month_window(window_seconds) {
        let left = days_remaining(today);
        let noun = if left == 1 { "day" } else { "days" };
        return format!("calendar month ({left} {noun} left)");
    }
    match window_seconds {
        60 => "minute".to_owned(),
        3_600 => "hour".to_owned(),
        86_400 => "day".to_owned(),
        604_800 => "week".to_owned(),
        s if s % DAY_SECONDS == 0 => format!("{} days", s / DAY_SECONDS),
        s if s % 3_600 == 0 => format!("{} hours", s / 3_600),
        s => format!("{s} s"),
    }
}
