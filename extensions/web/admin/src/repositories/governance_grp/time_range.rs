use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

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

/// Parse `?from=&to=&preset=` into a resolved `TimeRange`.
///
/// Resolution order:
/// 1. If `preset` is one of the known windows, anchor `to` at `now()` and derive `from`.
/// 2. Else parse RFC3339 `from`/`to` if both present.
/// 3. Else default to last 24 hours.
pub fn parse_time_range(query: &TimeRangeQuery) -> TimeRange {
    let now = Utc::now();

    if let Some(preset_str) = query.preset.as_deref() {
        if let Some(preset) = TimeRangePreset::parse(preset_str) {
            if let Some(d) = preset.duration() {
                return TimeRange {
                    from: now - d,
                    to: now,
                    preset,
                };
            }
        }
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
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}
