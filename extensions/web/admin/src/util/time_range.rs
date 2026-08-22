//! The time window audit, analytics, and trace queries are scoped by.
//!
//! A date parser rather than a repository — it reaches the database only to
//! count rows in a candidate window — so it lives outside `repositories`.
//!
//! Parsed from `?from=&to=&preset=` on audit pages. `count_requests_in_range`
//! exists so a page can cheaply test a candidate window before committing to it
//! and widen when the default returns nothing.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

/// Query parameters parsed from `?from=&to=&preset=` on audit pages.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TimeRangeQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
}

/// Resolved absolute time range used by every governance audit query.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct TimeRange {
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
    pub preset: TimeRangePreset,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimeRangePreset {
    Min15,
    Hour1,
    Hours24,
    Days7,
    Days30,
    Custom,
}

impl TimeRangePreset {
    const fn duration(self) -> Option<Duration> {
        match self {
            Self::Min15 => Some(Duration::minutes(15)),
            Self::Hour1 => Some(Duration::hours(1)),
            Self::Hours24 => Some(Duration::hours(24)),
            Self::Days7 => Some(Duration::days(7)),
            Self::Days30 => Some(Duration::days(30)),
            Self::Custom => None,
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value {
            "15m" => Some(Self::Min15),
            "1h" => Some(Self::Hour1),
            "24h" => Some(Self::Hours24),
            "7d" => Some(Self::Days7),
            "30d" => Some(Self::Days30),
            "custom" => Some(Self::Custom),
            _ => None,
        }
    }
}

pub fn parse_time_range(query: &TimeRangeQuery) -> TimeRange {
    let now = Utc::now();

    if let Some(preset_str) = query.preset.as_deref()
        && let Some(preset) = TimeRangePreset::parse(preset_str)
        && let Some(d) = preset.duration()
    {
        return TimeRange {
            from: now - d,
            to: now,
            preset,
        };
    }

    let parsed_from = query.from.as_deref().and_then(parse_rfc3339);
    let parsed_to = query.to.as_deref().and_then(parse_rfc3339);
    if let (Some(from), Some(to)) = (parsed_from, parsed_to) {
        return clamp_custom(from, to);
    }

    TimeRange {
        from: now - Duration::hours(24),
        to: now,
        preset: TimeRangePreset::Hours24,
    }
}

// Why: The widest window any query may scan. Matches the widest preset (30d).
//
// Why: the presets are bounded by construction, but `?from=&to=` is not —
// and the percentile stats behind the trace page (`repositories/traces/
// stats.rs`) aggregate *every* row in the window, because exact p50/p95/p99
// cannot be derived from a rollup. An unbounded custom range is therefore a
// full-table scan anyone can trigger from a query string. Bounding the window
// is what keeps that query O(window) instead of O(all rows).
pub const MAX_CUSTOM_WINDOW_DAYS: i64 = 30;

// Why: inverted ranges are swapped, and an over-wide one keeps its `to` and
// pulls `from` up to the cap — anchoring on the recent end, which is the half
// a reader is actually looking at.
fn clamp_custom(from: DateTime<Utc>, to: DateTime<Utc>) -> TimeRange {
    let (from, to) = if from <= to { (from, to) } else { (to, from) };
    let max = Duration::days(MAX_CUSTOM_WINDOW_DAYS);
    let from = if to - from > max { to - max } else { from };
    TimeRange {
        from,
        to,
        preset: TimeRangePreset::Custom,
    }
}

fn parse_rfc3339(s: &str) -> Option<DateTime<Utc>> {
    // Why: malformed input from user-supplied query strings is the "None"
    // branch — the standard carve-out for parse failures.
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

// Why: Resolve a fixed [`TimeRangePreset`] (not Custom) to a concrete window
// anchored at `now()`. Returns the same window `parse_time_range` would have
// produced for the matching preset string.
pub fn preset_to_range(preset: TimeRangePreset) -> TimeRange {
    let now = Utc::now();
    let d = preset.duration().unwrap_or_else(|| Duration::hours(24));
    TimeRange {
        from: now - d,
        to: now,
        preset,
    }
}

// Why: Cheap probe for the auto-widen path: just count rows in `ai_requests`
// inside the candidate window. Used to decide whether the default 24h
// window has data, or whether to fall back to a wider preset.
pub async fn count_requests_in_range(pool: &PgPool, range: TimeRange) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT COUNT(*)::bigint AS "count!"
           FROM ai_requests
           WHERE created_at >= $1 AND created_at < $2"#,
        range.from,
        range.to,
    )
    .fetch_one(pool)
    .await?;
    Ok(row.count)
}
