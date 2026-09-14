//! HTML datetime controls use UTC without an offset; shared links may use
//! RFC3339.
use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub(super) fn deserialize<'de, D: Deserializer<'de>>(
    d: D,
) -> Result<Option<DateTime<Utc>>, D::Error> {
    let raw = Option::<String>::deserialize(d)?;
    raw.filter(|value| !value.is_empty())
        .map(|value| {
            if let Ok(parsed) = DateTime::parse_from_rfc3339(&value) {
                return Ok(parsed.with_timezone(&Utc));
            }
            ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%dT%H:%M"]
                .iter()
                .find_map(|format| NaiveDateTime::parse_from_str(&value, format).ok())
                .map(|value| value.and_utc())
                .ok_or_else(|| serde::de::Error::custom("Expected a UTC date and time"))
        })
        .transpose()
}

#[expect(
    clippy::ref_option,
    reason = "Serde serialize_with requires a reference to the field type"
)]
pub(super) fn serialize<S: Serializer>(
    value: &Option<DateTime<Utc>>,
    s: S,
) -> Result<S::Ok, S::Error> {
    value
        .map(|value| value.format("%Y-%m-%dT%H:%M:%S").to_string())
        .serialize(s)
}
