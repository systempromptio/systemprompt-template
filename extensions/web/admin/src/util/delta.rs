//! Period-over-period delta math for the KPI cards.
//!
//! Pure so it is unit-testable from the crate's `tests/` directory; the view
//! layer wraps the result in a serializable struct. Polarity is the caller's
//! call — a rising request count is good, a rising cost is not — so the tone
//! comes out of `up_is_good`, not the sign alone.

/// A formatted delta: the display string, an arrow direction, and a tone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Delta {
    /// `"+12.4%"`, `"−8.0%"`, `"new"` (nothing before, something now), or
    /// `"—"` (nothing in either window).
    pub display_kind: DeltaKind,
    /// `up` | `down` | `flat` — which arrow the template draws.
    pub direction: &'static str,
    /// `good` | `bad` | `neutral` — how the template tints it.
    pub tone: &'static str,
}

/// The display split from the numbers so formatting stays in one place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeltaKind {
    /// Percentage change in tenths of a percent (so 124 renders "+12.4%").
    Pct(i64),
    /// Previous was zero, current is not.
    New,
    /// Both windows are zero.
    None,
}

impl Delta {
    /// Render the display string the KPI card prints.
    #[must_use]
    pub fn display(&self) -> String {
        match self.display_kind {
            DeltaKind::Pct(tenths) => {
                let sign = if tenths >= 0 { "+" } else { "\u{2212}" };
                format!("{sign}{}.{}%", tenths.abs() / 10, tenths.abs() % 10)
            },
            DeltaKind::New => "new".to_owned(),
            DeltaKind::None => "\u{2014}".to_owned(),
        }
    }
}

/// Compare two windows. `up_is_good` decides which direction tints `good`.
#[must_use]
pub fn delta(current: i64, previous: i64, up_is_good: bool) -> Delta {
    if previous == 0 && current == 0 {
        return Delta {
            display_kind: DeltaKind::None,
            direction: "flat",
            tone: "neutral",
        };
    }
    if previous == 0 {
        return Delta {
            display_kind: DeltaKind::New,
            direction: "up",
            tone: if up_is_good { "good" } else { "bad" },
        };
    }
    let tenths = ((current as f64 - previous as f64) / previous as f64 * 1000.0).round() as i64;
    let (direction, tone) = match tenths.cmp(&0) {
        std::cmp::Ordering::Greater => ("up", if up_is_good { "good" } else { "bad" }),
        std::cmp::Ordering::Less => ("down", if up_is_good { "bad" } else { "good" }),
        std::cmp::Ordering::Equal => ("flat", "neutral"),
    };
    Delta {
        display_kind: DeltaKind::Pct(tenths),
        direction,
        tone,
    }
}
