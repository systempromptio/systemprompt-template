//! The calendar month a usage report is scoped to.
//!
//! Distinct from [`crate::util::time_range::TimeRange`], which is a rolling
//! window anchored at `now()`. A month-end report is billed against a calendar
//! boundary, and a rolling thirty days is never that boundary — February would
//! be over-counted and every other month under-counted by a day or two.
//!
//! The window is half-open (`>= from AND < to`) so a request at the last
//! microsecond of a month is counted exactly once, in that month.

use chrono::{DateTime, Datelike, Months, NaiveDate, TimeZone, Utc};
use serde::{Deserialize, Serialize};

use crate::util::time_range::{TimeRange, TimeRangePreset};

const MONTH_OPTIONS: u32 = 13;

/// Query parameter parsed from `?month=YYYY-MM`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MonthQuery {
    pub month: Option<String>,
}

/// A resolved calendar month.
#[derive(Debug, Clone)]
pub struct MonthRange {
    pub key: String,
    pub label: String,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    // Why: False for the month currently in progress. A report over a partial
    // month is still useful, but it must say so — a half-month of cost read
    // as a full month makes every margin look twice as good as it is.
    pub is_complete: bool,
}

/// One entry in the month picker.
#[derive(Debug, Clone, Serialize)]
pub struct MonthOption {
    pub key: String,
    pub label: String,
    pub selected: bool,
}

impl MonthRange {
    // Why: The equivalent rolling-window type, so repositories already written
    // against [`TimeRange`] can be reused for a month unchanged.
    #[must_use]
    pub const fn as_time_range(&self) -> TimeRange {
        TimeRange {
            from: self.from,
            to: self.to,
            preset: TimeRangePreset::Custom,
        }
    }

    #[must_use]
    pub fn previous(&self) -> Self {
        from_start(self.from - Months::new(1))
    }

    #[must_use]
    pub fn next(&self) -> Option<Self> {
        let candidate = from_start(self.to);
        (candidate.from <= Utc::now()).then_some(candidate)
    }
}

// Why: Anything absent or unparseable falls back to the last *complete* month.
// These are end-of-month reports, and opening one on the 2nd to a
// two-days-of-data month reads as a collapse in usage rather than as a
// month that has barely started.
#[must_use]
pub fn parse_month_range(query: &MonthQuery) -> MonthRange {
    query
        .month
        .as_deref()
        .and_then(parse_month_key)
        .map_or_else(last_complete_month, from_start)
}

#[must_use]
pub fn list_month_options(selected: &MonthRange) -> Vec<MonthOption> {
    let newest = month_start(Utc::now());
    (0..MONTH_OPTIONS)
        .map(|back| {
            let month = from_start(newest - Months::new(back));
            MonthOption {
                selected: month.key == selected.key,
                key: month.key,
                label: month.label,
            }
        })
        .collect()
}

fn last_complete_month() -> MonthRange {
    from_start(month_start(Utc::now()) - Months::new(1))
}

fn parse_month_key(raw: &str) -> Option<DateTime<Utc>> {
    let (year, month) = raw.split_once('-')?;
    let year: i32 = year.parse().ok()?;
    let month: u32 = month.parse().ok()?;
    let date = NaiveDate::from_ymd_opt(year, month, 1)?;
    Utc.from_local_datetime(&date.and_hms_opt(0, 0, 0)?)
        .single()
}

fn from_start(instant: DateTime<Utc>) -> MonthRange {
    let from = month_start(instant);
    let to = from + Months::new(1);
    MonthRange {
        key: from.format("%Y-%m").to_string(),
        label: from.format("%B %Y").to_string(),
        from,
        to,
        is_complete: to <= Utc::now(),
    }
}

pub(crate) fn month_start(instant: DateTime<Utc>) -> DateTime<Utc> {
    // Why: the first of the month at midnight always exists, so the fallible
    // constructors cannot fail here; returning the input unchanged rather than
    // panicking keeps a report renderable in the impossible case.
    NaiveDate::from_ymd_opt(instant.year(), instant.month(), 1)
        .and_then(|d| d.and_hms_opt(0, 0, 0))
        .and_then(|dt| Utc.from_local_datetime(&dt).single())
        .unwrap_or(instant)
}
