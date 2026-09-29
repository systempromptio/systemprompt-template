//! Resolving the window an export asks for, per the dataset's contract.
//!
//! A window the reader could not have meant — an unknown preset, a bound that
//! is not a date, one end of a custom range without the other — is a 400
//! that names the problem, never a silent fall back to a default the reader
//! did not pick. A range wider than the contract allows is narrowed and
//! flagged `clamped`, so the preview can say the file covers less.

use chrono::{DateTime, Duration, NaiveTime, Utc};
use serde::Deserialize;

use super::model::{ExportWindow, Window};
use crate::error::{AdminError, AdminResult};
use crate::handlers::ssr::analysis::time;
use crate::util::month_range::{MonthQuery, parse_month_range};
use crate::util::time_range::TimeRangePreset;

pub(crate) const RETAINED_DAYS: [u32; 5] = [1, 7, 30, 90, 365];
const MAX_RETAINED_DAYS: i64 = 366;

// Why: the export's own ceiling on a live custom range, matching the widest
// live preset (90d). The pages keep their tighter
// `time_range::MAX_CUSTOM_WINDOW_DAYS`, which guards percentile scans an
// export never runs; a file is already bounded by the dataset's row cap.
const MAX_LIVE_EXPORT_DAYS: i64 = 90;
const DEFAULT_LIVE_HOURS: i64 = 24;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct WindowQuery {
    preset: Option<String>,
    from: Option<String>,
    to: Option<String>,
    days: Option<u32>,
    start: Option<String>,
    end: Option<String>,
    month: Option<String>,
}

pub(crate) fn resolve(kind: Window, query: &WindowQuery) -> AdminResult<Option<ExportWindow>> {
    match kind {
        Window::None => Ok(None),
        Window::Live => live(query).map(Some),
        Window::Days => days_only(query).map(Some),
        Window::Retained => retained(query).map(Some),
        Window::Month => Ok(Some(month(query))),
    }
}

fn bad(message: &str) -> AdminError {
    AdminError::BadRequest(message.to_owned())
}

// Why: the dialog's `datetime-local` controls send `%Y-%m-%dT%H:%M`; every
// spelling `time::parse` accepts is a valid bound.
fn bound(value: Option<&str>, name: &str) -> AdminResult<Option<DateTime<Utc>>> {
    value.map(str::trim).filter(|v| !v.is_empty()).map_or_else(
        || Ok(None),
        |v| {
            time::parse(v)
                .map(Some)
                .ok_or_else(|| bad(&format!("The window's {name} `{v}` is not a date or time.")))
        },
    )
}

fn live(query: &WindowQuery) -> AdminResult<ExportWindow> {
    let now = Utc::now();
    let preset = query
        .preset
        .as_deref()
        .map(str::trim)
        .filter(|p| !p.is_empty());
    if let Some(p) = preset.filter(|p| *p != "custom") {
        let duration = TimeRangePreset::parse(p)
            .and_then(TimeRangePreset::duration)
            .ok_or_else(|| {
                bad(&format!(
                    "`{p}` is not a window; use 15m, 1h, 24h, 7d, 30d or 90d."
                ))
            })?;
        return Ok(ExportWindow {
            from: now - duration,
            to: now,
            clamped: false,
        });
    }
    let from = bound(query.from.as_deref(), "start")?;
    let to = bound(query.to.as_deref(), "end")?;
    match (from, to) {
        (Some(from), Some(to)) => Ok(clamp(from, to, Duration::days(MAX_LIVE_EXPORT_DAYS))),
        (None, None) if preset.is_none() => Ok(ExportWindow {
            from: now - Duration::hours(DEFAULT_LIVE_HOURS),
            to: now,
            clamped: false,
        }),
        _ => Err(bad("A custom window needs both a start and an end.")),
    }
}

// Why: an inverted range is swapped, and an over-wide one keeps its end and
// pulls its start up to the cap — the recent half is the half a reader wants.
fn clamp(from: DateTime<Utc>, to: DateTime<Utc>, max: Duration) -> ExportWindow {
    let (from, to) = if from <= to { (from, to) } else { (to, from) };
    if to - from > max {
        ExportWindow {
            from: to - max,
            to,
            clamped: true,
        }
    } else {
        ExportWindow {
            from,
            to,
            clamped: false,
        }
    }
}

// Why: the snapshot pipeline buckets facts under UTC days and the current day
// is still filling, so a day window ends at the next midnight — the same
// boundary the analysis pages use.
pub(crate) fn next_midnight() -> DateTime<Utc> {
    (Utc::now().date_naive() + Duration::days(1))
        .and_time(NaiveTime::MIN)
        .and_utc()
}

fn days_only(query: &WindowQuery) -> AdminResult<ExportWindow> {
    let days = query.days.unwrap_or(30);
    if !RETAINED_DAYS.contains(&days) {
        return Err(bad("Choose a window of 1, 7, 30, 90 or 365 days."));
    }
    let to = next_midnight();
    Ok(ExportWindow {
        from: to - Duration::days(i64::from(days)),
        to,
        clamped: false,
    })
}

// Why: the dialog's end control is a date, and a reader who picks 1–30
// September means to include the 30th; a date-only end therefore closes at
// the following midnight. An end with a time is taken as given.
fn inclusive_end(value: Option<&str>) -> AdminResult<Option<DateTime<Utc>>> {
    let end = bound(value, "end")?;
    let date_only = value.is_some_and(|v| v.trim().len() == "YYYY-MM-DD".len());
    Ok(end.map(|e| if date_only { e + Duration::days(1) } else { e }))
}

fn retained(query: &WindowQuery) -> AdminResult<ExportWindow> {
    let start = bound(query.start.as_deref(), "start")?;
    let end = inclusive_end(query.end.as_deref())?;
    let (from, to) = match (start, end) {
        (None, None) => return days_only(query),
        (Some(from), Some(to)) => (from, to),
        _ => return Err(bad("A custom window needs both a start and an end date.")),
    };
    if from >= to {
        return Err(bad("The window's end must be after its start."));
    }
    Ok(clamp(from, to, Duration::days(MAX_RETAINED_DAYS)))
}

// Why: the reports' own month rule — absent or unparseable is the last
// complete month — so the dialog and the report page agree on the default.
fn month(query: &WindowQuery) -> ExportWindow {
    let range = parse_month_range(&MonthQuery {
        month: query.month.clone(),
    });
    ExportWindow {
        from: range.from,
        to: range.to,
        clamped: false,
    }
}
