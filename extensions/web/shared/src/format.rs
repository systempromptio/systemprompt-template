//! Pure value formatting shared by the web crates' display layers.
//!
//! One implementation per concern so a cost or duration renders identically
//! on every page that shows it.

// Why: Ids are long and the leading segment is the distinguishing part, so a
// table cell shows the head and puts the full value in a `title`.
pub fn short_id(id: &str) -> String {
    const KEEP: usize = 12;
    if id.chars().count() > KEEP {
        let head: String = id.chars().take(KEEP).collect();
        format!("{head}…")
    } else {
        id.to_owned()
    }
}
// Why: `—` rather than `$0`: a session with no billed traffic has no cost to
// show, which is different from one that cost nothing.
pub fn format_cost(microdollars: i64) -> String {
    if microdollars <= 0 {
        return "—".to_owned();
    }
    let dollars = microdollars as f64 / 1_000_000.0;
    if dollars >= 1.0 {
        format!("${dollars:.2}")
    } else if dollars >= 0.01 {
        format!("${dollars:.4}")
    } else {
        format!("${dollars:.6}")
    }
}
pub fn short_num(n: i64) -> String {
    let abs = n.unsigned_abs();
    if abs >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if abs >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
pub fn format_duration_ms(ms: i64) -> String {
    if ms < 1000 {
        format!("{ms} ms")
    } else if ms < 60_000 {
        format!("{:.2} s", ms as f64 / 1000.0)
    } else if ms < 3_600_000 {
        format!("{:.1} min", ms as f64 / 60_000.0)
    } else {
        format!("{:.1} h", ms as f64 / 3_600_000.0)
    }
}

// Why: `max_chars` is a character budget, so the guard must count characters
// too. Guarding on `s.len()` (bytes) and cutting on `char_indices()` lets any
// string that is longer in bytes than in characters fall through the guard and
// come back with an ellipsis appended to text that was never truncated.
pub fn truncate_chars(s: &str, max_chars: usize) -> &str {
    s.char_indices()
        .nth(max_chars)
        .map_or(s, |(end, _)| &s[..end])
}

// Why: the one truncation the admin console shows a reader, so there is one
// ellipsis spelling (`…`) and it appears only when something was cut.
pub fn truncate_ellipsis(s: &str, max_chars: usize) -> String {
    let head = truncate_chars(s, max_chars);
    if head.len() == s.len() {
        s.to_owned()
    } else {
        format!("{head}\u{2026}")
    }
}

// Why: distinct from `short_num` — this ladder drops the decimal between 10k
// and a million, where the tenth of a thousand is noise rather than precision.
// Two pages had a byte-identical copy of it under two different names.
pub fn compact_num(v: i64) -> String {
    if v >= 1_000_000 {
        format!("{:.1}M", v as f64 / 1_000_000.0)
    } else if v >= 10_000 {
        format!("{}k", v / 1000)
    } else if v >= 1000 {
        format!("{:.1}k", v as f64 / 1000.0)
    } else {
        v.to_string()
    }
}

// Why: "3d ago" answers "is this still in use" at a glance; the exact stamp
// stays on the cell's `title`. Months are thirty days, which is the roster's
// idle threshold, so "1mo ago" and the idle-30d chip agree. One ladder, so
// two pages cannot disagree about what a fresh timestamp reads as.
pub fn relative_time(delta_secs: i64) -> String {
    match delta_secs.max(0) {
        d if d < 60 => "just now".to_owned(),
        d if d < 3_600 => format!("{}m ago", d / 60),
        d if d < 86_400 => format!("{}h ago", d / 3_600),
        d if d < 2_592_000 => format!("{}d ago", d / 86_400),
        d => format!("{}mo ago", d / 2_592_000),
    }
}
