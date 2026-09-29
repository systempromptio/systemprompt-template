//! One tool call as the Tools and Artifacts tables show it: the kind glyph
//! and label an artifact carries, and its payload size.
//!
//! Only the three formatters the analysis conversation detail shares are
//! here; the row view itself lands with the Tools page (Stage-3 phase 9).

use crate::handlers::ssr::format::short_num;

pub(crate) fn kind_icon(kind: Option<&str>) -> &'static str {
    match kind {
        Some("file") => "file",
        Some("card") => "card",
        Some("ui") => "ui",
        Some("body") => "body",
        _ => "wrench",
    }
}

pub(crate) fn kind_label(kind: Option<&str>) -> &'static str {
    match kind {
        Some("file") => "File",
        Some("card") => "Card",
        Some("ui") => "UI",
        Some("body") => "Body",
        _ => "Tool call",
    }
}

pub(crate) fn format_bytes(bytes: i64) -> String {
    if bytes <= 0 {
        return "\u{2014}".to_owned();
    }
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let kib = bytes / 1024;
    if kib < 1024 {
        return format!("{kib} KiB");
    }
    format!("{} MiB", short_num(kib / 1024))
}
