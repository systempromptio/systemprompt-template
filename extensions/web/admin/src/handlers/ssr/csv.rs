//! Minimal CSV assembly for the report export endpoints.
//!
//! RFC 4180 quoting, UTF-8, one download response builder. Deliberately not a
//! dependency: the exports write a handful of typed rows, and a CSV crate's
//! serde machinery would be more surface than the whole feature.

use axum::http::header;
use axum::response::Response;

#[derive(Debug, Default)]
pub(crate) struct CsvBuilder {
    out: String,
}

impl CsvBuilder {
    pub(crate) fn new(header: &[&str]) -> Self {
        let mut b = Self::default();
        b.row(header);
        b
    }

    pub(crate) fn row(&mut self, fields: &[&str]) {
        let mut first = true;
        for field in fields {
            if !first {
                self.out.push(',');
            }
            first = false;
            self.out.push_str(&escape(field));
        }
        self.out.push_str("\r\n");
    }

    // Why: genuinely renders the finished download, never an error — the
    // callers are AdminResult handlers and their error variants pick statuses.
    // Why: lint-ok: http-error
    pub(crate) fn into_response(self, filename: &str) -> Response {
        Response::builder()
            .header(header::CONTENT_TYPE, "text/csv; charset=utf-8")
            .header(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            )
            .body(self.out.into())
            .unwrap_or_default()
    }
}

fn escape(field: &str) -> String {
    if field.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_owned()
    }
}

// Why: integer formatting, not f64 — these files feed finance spreadsheets,
// and the microdollar ledger must survive the export exactly. Six decimals is
// the full stored precision.
pub(crate) fn usd(microdollars: i64) -> String {
    let sign = if microdollars < 0 { "-" } else { "" };
    let abs = microdollars.unsigned_abs();
    format!("{sign}{}.{:06}", abs / 1_000_000, abs % 1_000_000)
}
