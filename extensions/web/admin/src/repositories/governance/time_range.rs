//! The time window every governance audit query is scoped by.
//!
//! Parsed from `?from=&to=&preset=` on audit pages. `count_requests_in_range`
//! exists so a page can cheaply test a candidate window before committing to it
//! and widen when the default returns nothing.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TimeRangeQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
}

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
        return TimeRange {
            from,
            to,
            preset: TimeRangePreset::Custom,
        };
    }

    TimeRange {
        from: now - Duration::hours(24),
        to: now,
        preset: TimeRangePreset::Hours24,
    }
}

fn parse_rfc3339(s: &str) -> Option<DateTime<Utc>> {
    // Why: malformed input from user-supplied query strings is the "None"
    // branch — the standard carve-out for parse failures.
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

pub fn preset_to_range(preset: TimeRangePreset) -> TimeRange {
    let now = Utc::now();
    let d = preset.duration().unwrap_or_else(|| Duration::hours(24));
    TimeRange {
        from: now - d,
        to: now,
        preset,
    }
}

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
