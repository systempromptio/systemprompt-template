//! Waterfall geometry for the trace-detail page.
//!
//! The bars are an inline SVG rather than a stack of positioned divs: one
//! element carries the whole chart, the time axis is drawn once instead of
//! implied per row, and a zero-duration span still gets a visible mark. The
//! viewBox is a fixed 1000 units wide, so every x is a permille of the trace's
//! own first-to-last window and no CSS custom property has to carry a number.

use serde::Serialize;

use crate::handlers::ssr::format::format_duration_ms;
use crate::repositories::traces::Span;

// Why: the viewBox is unitless; the SVG is scaled to the card by CSS. 1000
// across gives sub-pixel-free integers for a percentage.
const VIEW_WIDTH: f64 = 1000.0;
const ROW_HEIGHT: f64 = 22.0;
const BAR_HEIGHT: f64 = 12.0;
const BAR_INSET: f64 = 5.0;
// Why: an instantaneous span (a governance decision) has zero width and would
// otherwise be invisible; it is drawn as a tick, not widened into a lie.
const MIN_BAR_WIDTH: f64 = 3.0;

#[derive(Debug, Serialize)]
pub(super) struct WaterfallView {
    pub(super) view_width: f64,
    pub(super) view_height: f64,
    pub(super) rows: Vec<WaterfallRow>,
    pub(super) ticks: Vec<WaterfallTick>,
    pub(super) has_rows: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct WaterfallRow {
    pub(super) id: String,
    pub(super) kind: &'static str,
    pub(super) status: &'static str,
    pub(super) name: String,
    pub(super) label: String,
    pub(super) x: f64,
    pub(super) y: f64,
    pub(super) width: f64,
    pub(super) height: f64,
    pub(super) row_y: f64,
}

#[derive(Debug, Serialize)]
pub(super) struct WaterfallTick {
    pub(super) x: f64,
    pub(super) label: String,
}

pub(super) fn build_waterfall(spans: &[Span], total_ms: i64) -> WaterfallView {
    let start = spans.iter().map(|s| s.started_at).min();
    let rows: Vec<WaterfallRow> = spans
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let offset_ms = start.map_or(0, |a| (s.started_at - a).num_milliseconds().max(0));
            let (x, width) = if total_ms > 0 {
                let x = round2(offset_ms as f64 / total_ms as f64 * VIEW_WIDTH);
                let w = round2(s.duration_ms as f64 / total_ms as f64 * VIEW_WIDTH);
                (x.min(VIEW_WIDTH - MIN_BAR_WIDTH), w.max(MIN_BAR_WIDTH))
            } else {
                (0.0, MIN_BAR_WIDTH)
            };
            let row_y = i as f64 * ROW_HEIGHT;
            WaterfallRow {
                id: s.id.clone(),
                kind: s.kind.as_str(),
                status: s.status.as_str(),
                name: s.name.clone(),
                label: format!(
                    "{} · +{} · {}",
                    s.name,
                    format_duration_ms(offset_ms),
                    format_duration_ms(s.duration_ms)
                ),
                x,
                y: row_y + BAR_INSET,
                width: width.min(VIEW_WIDTH - x),
                height: BAR_HEIGHT,
                row_y,
            }
        })
        .collect();

    WaterfallView {
        view_width: VIEW_WIDTH,
        view_height: (rows.len() as f64 * ROW_HEIGHT).max(ROW_HEIGHT),
        ticks: build_ticks(total_ms),
        has_rows: !rows.is_empty(),
        rows,
    }
}

// Why: five gridlines, labelled in the trace's own elapsed time. A wall-clock
// axis would be unreadable on a trace that lasted 40 ms and useless on one
// that lasted an hour.
fn build_ticks(total_ms: i64) -> Vec<WaterfallTick> {
    (0..=4)
        .map(|i| {
            let fraction = f64::from(i) / 4.0;
            WaterfallTick {
                x: round2(fraction * VIEW_WIDTH),
                label: format_duration_ms((total_ms as f64 * fraction).round() as i64),
            }
        })
        .collect()
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}
