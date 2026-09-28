//! The file formats an export is written in.
//!
//! One writer per format over the same typed [`Table`]: CSV (RFC 4180, UTF-8,
//! CRLF), JSON (an array of objects keyed by column), JSONL (one object per
//! line) and Markdown (a pipe table with numeric columns right-aligned).
//! Deliberately no CSV or spreadsheet crate: the writers are a few dozen lines
//! and a serde-backed dependency would be more surface than the feature.

use axum::http::header;
use axum::response::Response;

use super::model::{Cell, CellKind, Column, Table};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Format {
    Csv,
    Json,
    Jsonl,
    Markdown,
}

impl Format {
    pub(crate) const ALL: [Self; 4] = [Self::Csv, Self::Json, Self::Jsonl, Self::Markdown];

    pub(crate) const fn extension(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Jsonl => "jsonl",
            Self::Markdown => "md",
        }
    }

    pub(crate) const fn param(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Json => "json",
            Self::Jsonl => "jsonl",
            Self::Markdown => "markdown",
        }
    }

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Csv => "CSV",
            Self::Json => "JSON",
            Self::Jsonl => "JSON Lines",
            Self::Markdown => "Markdown",
        }
    }

    const fn content_type(self) -> &'static str {
        match self {
            Self::Csv => "text/csv; charset=utf-8",
            Self::Json => "application/json; charset=utf-8",
            Self::Jsonl => "application/x-ndjson; charset=utf-8",
            Self::Markdown => "text/markdown; charset=utf-8",
        }
    }

    pub(crate) fn write(self, columns: &[Column], table: &Table) -> String {
        match self {
            Self::Csv => csv(columns, table),
            Self::Json => json(columns, table),
            Self::Jsonl => jsonl(columns, table),
            Self::Markdown => markdown(columns, table),
        }
    }

    pub(crate) fn into_response(self, filename: &str, body: String) -> Response {
        Response::builder()
            .header(header::CONTENT_TYPE, self.content_type())
            .header(
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            )
            .body(body.into())
            .unwrap_or_default()
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

fn plain(cell: &Cell) -> String {
    match cell {
        Cell::Empty => String::new(),
        Cell::Text(s) => s.clone(),
        Cell::Integer(n) => n.to_string(),
        Cell::Decimal(f) => format!("{f:.3}"),
        Cell::Money(m) => usd(*m),
        Cell::Timestamp(t) => t.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        Cell::Bool(b) => b.to_string(),
    }
}

fn csv(columns: &[Column], table: &Table) -> String {
    let mut out = String::new();
    csv_line(&mut out, columns.iter().map(|c| c.key.to_owned()));
    for row in &table.rows {
        csv_line(&mut out, row.iter().map(plain));
    }
    out
}

fn csv_line(out: &mut String, fields: impl Iterator<Item = String>) {
    for (i, field) in fields.enumerate() {
        if i > 0 {
            out.push(',');
        }
        if field.contains([',', '"', '\n', '\r']) {
            out.push('"');
            out.push_str(&field.replace('"', "\"\""));
            out.push('"');
        } else {
            out.push_str(&field);
        }
    }
    out.push_str("\r\n");
}

// JSON: the serialiser's own output — one object per row keyed by column,
// each cell carrying its type. Built as a `Value` because this is the format
// boundary itself, not data the crate goes on to read.
fn json_value(cell: &Cell) -> serde_json::Value {
    match cell {
        Cell::Empty => serde_json::Value::Null,
        Cell::Text(s) => serde_json::Value::from(s.as_str()),
        Cell::Integer(n) => serde_json::Value::from(*n),
        Cell::Decimal(f) => serde_json::Value::from(*f),
        Cell::Money(m) => serde_json::Value::from(usd(*m)),
        Cell::Timestamp(t) => {
            serde_json::Value::from(t.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
        },
        Cell::Bool(b) => serde_json::Value::from(*b),
    }
}

fn json_line(columns: &[Column], row: &[Cell]) -> String {
    let object: serde_json::Map<String, serde_json::Value> = columns
        .iter()
        .zip(row)
        .map(|(c, cell)| (c.key.to_owned(), json_value(cell)))
        .collect();
    serde_json::to_string(&object).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to serialise an export row");
        String::new()
    })
}

fn json(columns: &[Column], table: &Table) -> String {
    let mut out = String::from("[\n");
    for (i, row) in table.rows.iter().enumerate() {
        if i > 0 {
            out.push_str(",\n");
        }
        out.push_str("  ");
        out.push_str(&json_line(columns, row));
    }
    out.push_str("\n]\n");
    out
}

fn jsonl(columns: &[Column], table: &Table) -> String {
    let mut out = String::new();
    for row in &table.rows {
        out.push_str(&json_line(columns, row));
        out.push('\n');
    }
    out
}

fn markdown(columns: &[Column], table: &Table) -> String {
    let mut out = String::from("|");
    for c in columns {
        out.push(' ');
        out.push_str(c.label);
        out.push_str(" |");
    }
    out.push_str("\n|");
    for c in columns {
        out.push_str(match c.kind {
            CellKind::Integer | CellKind::Decimal | CellKind::Money => " ---: |",
            _ => " --- |",
        });
    }
    out.push('\n');
    for row in &table.rows {
        out.push('|');
        for cell in row {
            out.push(' ');
            out.push_str(&plain(cell).replace('|', "\\|").replace(['\n', '\r'], " "));
            out.push_str(" |");
        }
        out.push('\n');
    }
    out
}
