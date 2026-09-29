//! Window bounds arrive as HTML date controls (`%Y-%m-%d`), datetime controls
//! (UTC without an offset) or RFC3339 from shared links; they render back as a
//! date whenever they sit on a UTC midnight so the date control round-trips.
//! Shared by the export dialog's window parser.
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};

pub fn parse(value: &str) -> Option<DateTime<Utc>> {
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return Some(parsed.with_timezone(&Utc));
    }
    if let Ok(day) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        return Some(day.and_time(NaiveTime::MIN).and_utc());
    }
    ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%dT%H:%M"]
        .iter()
        .find_map(|format| NaiveDateTime::parse_from_str(value, format).ok())
        .map(|value| value.and_utc())
}

pub fn render(value: DateTime<Utc>) -> String {
    if value.time() == NaiveTime::MIN {
        value.format("%Y-%m-%d").to_string()
    } else {
        value.format("%Y-%m-%dT%H:%M:%S").to_string()
    }
}
