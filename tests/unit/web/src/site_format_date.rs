//! `format_date` is the only date formatter the public-site providers use, and
//! ingested content carries dates in two shapes: RFC 3339 (`published_at` from
//! the content pipeline) and the looser `DateTime<Utc>` display form. Both must
//! render as "Month DD, YYYY"; anything else must be `None` so the template's
//! `{{#if}}` guard hides the byline rather than printing a raw timestamp.

use systemprompt_web_site::format::format_date;

#[test]
fn formats_an_rfc_3339_timestamp_as_month_day_year() {
    assert_eq!(
        format_date("2026-07-20T10:30:00Z").as_deref(),
        Some("July 20, 2026")
    );
}

#[test]
fn formats_an_rfc_3339_timestamp_with_a_numeric_offset() {
    assert_eq!(
        format_date("2026-01-05T23:00:00+05:00").as_deref(),
        Some("January 05, 2026")
    );
}

#[test]
fn falls_back_to_the_loose_datetime_display_form() {
    assert_eq!(
        format_date("2026-12-31 08:00:00 UTC").as_deref(),
        Some("December 31, 2026")
    );
}

#[test]
fn returns_none_for_anything_it_cannot_parse() {
    for raw in ["", "not a date", "2026-13-01T00:00:00Z", "20 July 2026"] {
        assert!(
            format_date(raw).is_none(),
            "{raw:?} must not produce a rendered date"
        );
    }
}
