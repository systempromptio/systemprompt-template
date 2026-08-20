//! Spend-limit proximity meter view-model.
//!
//! Split from `charts.rs` at the 300-line ceiling.

use serde::Serialize;

use crate::handlers::ssr::format::format_cost;

// Why: `state` turns "warn" exactly when the gateway records a soft-cap
// crossing (spent >= warn threshold), so the meter and org_budget_warnings
// describe the same moment; with no soft cap configured it stays "ok" until
// the hard cap that produces the 429.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct MeterView {
    pub label: String,
    pub spent_display: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hard_cap_display: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub soft_cap_display: Option<String>,
    /// Percent of the hard cap, clamped to 0..=100 for the track width.
    pub pct: i64,
    /// Soft-cap tick position as a percent of the hard cap.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub soft_pct: Option<i64>,
    /// ok | warn | over | none — none is an uncapped plan.
    pub state: &'static str,
    pub caption: String,
}

pub(crate) struct MeterInput {
    pub label: String,
    pub spent_microdollars: i64,
    pub cap_microdollars: Option<i64>,
    pub warn_microdollars: Option<i64>,
}

pub(crate) fn meter_view(input: MeterInput) -> MeterView {
    let spent_display = format_cost(input.spent_microdollars);
    let Some(cap) = input.cap_microdollars.filter(|c| *c > 0) else {
        return MeterView {
            label: input.label,
            caption: format!("{spent_display} this month · uncapped plan"),
            spent_display,
            hard_cap_display: None,
            soft_cap_display: None,
            pct: 0,
            soft_pct: None,
            state: "none",
        };
    };

    let raw_pct = input.spent_microdollars.saturating_mul(100) / cap;
    let warn = input.warn_microdollars.filter(|w| *w > 0);
    let state = if input.spent_microdollars >= cap {
        "over"
    } else if warn.is_some_and(|w| input.spent_microdollars >= w) {
        "warn"
    } else {
        "ok"
    };
    let cap_display = format_cost(cap);
    let soft_cap_display = warn.map(format_cost);
    let caption = soft_cap_display.as_ref().map_or_else(
        || format!("{spent_display} of {cap_display} ({raw_pct}%)"),
        |soft| format!("{spent_display} of {cap_display} ({raw_pct}%) · soft limit {soft}"),
    );

    MeterView {
        label: input.label,
        spent_display,
        hard_cap_display: Some(cap_display),
        soft_cap_display,
        pct: raw_pct.clamp(0, 100),
        soft_pct: warn.map(|w| (w.saturating_mul(100) / cap).clamp(0, 100)),
        state,
        caption,
    }
}
