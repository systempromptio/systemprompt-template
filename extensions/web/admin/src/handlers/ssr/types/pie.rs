//! Distribution donut view-model: server-precomputed SVG arcs.
//!
//! Split from `charts.rs` at the 300-line ceiling; same design rule — the
//! view carries every derived value (each slice's path, its tooltip, the
//! centre figure) so the template does no arithmetic.

use serde::Serialize;

use crate::handlers::ssr::format::short_num;

use super::svg_line::CHART_COLOR_TOKENS;

// Why: the ring is drawn as one annular path per slice in a 100×100 space,
// so it scales to whatever the panel gives it and every slice can carry its
// own tooltip and hover state — a conic-gradient disc could do neither. The
// legend stays the accessible representation.
#[derive(Debug, Serialize)]
pub(crate) struct PieView {
    // Why: serialized as chart_title — the layout partial's `title=` hash
    // param shadows a context field named `title` inside nested partials.
    #[serde(rename = "chart_title")]
    pub title: &'static str,
    pub subtitle: String,
    pub has_data: bool,
    pub aria_label: String,
    pub arcs: Vec<PieArcView>,
    pub legend: Vec<PieSliceView>,
    pub center_value: String,
    pub center_label: &'static str,
    pub empty_message: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct PieArcView {
    pub path_d: String,
    pub color_token: &'static str,
    pub tooltip: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct PieSliceView {
    pub label: String,
    pub color_index: usize,
    pub share_display: String,
    pub share_pct: String,
    pub value_display: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_url: Option<String>,
}

// Why: one slice's inputs before percentage math — a label, its magnitude,
// the legend detail line, and an optional drill-down link.
pub(crate) struct PieSliceInput {
    pub label: String,
    pub value: i64,
    pub value_display: String,
    pub filter_url: Option<String>,
}

const PIE_COLORS: usize = CHART_COLOR_TOKENS.len();
const OUTER_R: f64 = 48.0;
const INNER_R: f64 = 31.0;
// Why: a full-circle arc is undefined in SVG (start equals end), so a lone
// slice stops a hair short of 360° and reads as a ring with a seam.
const FULL_TURN: f64 = 0.9999;

// Why: slices arrive sorted descending; everything past the sixth folds into
// "Other" so the ring stays readable and the seven colour tokens always
// suffice. The last slice's end is pinned to the full turn so rounding can
// never leave a sliver.
pub(crate) fn pie_view(
    title: &'static str,
    subtitle: String,
    slices: Vec<PieSliceInput>,
    center_label: &'static str,
    empty_message: &'static str,
) -> PieView {
    let total: i64 = slices.iter().map(|s| s.value).sum();
    if total <= 0 {
        return PieView {
            title,
            subtitle,
            has_data: false,
            aria_label: String::new(),
            arcs: Vec::new(),
            legend: Vec::new(),
            center_value: String::new(),
            center_label,
            empty_message,
        };
    }

    let kept = fold_tail(slices);
    let count = kept.len();
    let mut arcs = Vec::with_capacity(count);
    let mut legend = Vec::with_capacity(count);
    let mut cumulative = 0.0f64;
    for (i, slice) in kept.into_iter().enumerate() {
        let share = slice.value as f64 / total as f64;
        let from = cumulative;
        cumulative += share;
        let to = if i + 1 == count { 1.0 } else { cumulative };
        let color_token = CHART_COLOR_TOKENS[i.min(PIE_COLORS - 1)];
        let share_display = format!("{:.1}%", share * 100.0);
        arcs.push(PieArcView {
            path_d: arc_path(from, to),
            color_token,
            tooltip: format!(
                "{} — {} · {}",
                slice.label, share_display, slice.value_display
            ),
        });
        legend.push(PieSliceView {
            label: slice.label,
            color_index: i + 1,
            share_pct: format!("{:.1}", share * 100.0),
            share_display,
            value_display: slice.value_display,
            filter_url: slice.filter_url,
        });
    }

    let aria_label = format!(
        "{title}: {}",
        legend
            .iter()
            .map(|s| format!("{} {}", s.label, s.share_display))
            .collect::<Vec<_>>()
            .join(", ")
    );

    PieView {
        title,
        subtitle,
        has_data: true,
        aria_label,
        arcs,
        legend,
        center_value: short_num(total),
        center_label,
        empty_message,
    }
}

fn fold_tail(slices: Vec<PieSliceInput>) -> Vec<PieSliceInput> {
    let mut kept: Vec<PieSliceInput> = Vec::new();
    for (i, slice) in slices.into_iter().enumerate() {
        if i < PIE_COLORS - 1 {
            kept.push(slice);
        } else if let Some(other) = kept.get_mut(PIE_COLORS - 1) {
            other.value += slice.value;
        } else {
            kept.push(PieSliceInput {
                label: "Other".to_owned(),
                value: slice.value,
                value_display: String::new(),
                filter_url: None,
            });
        }
    }
    kept
}

// Why: one annular sector from `from` to `to` (fractions of a turn, clockwise
// from twelve o'clock): outer arc forward, inner arc back, closed.
fn arc_path(from: f64, to: f64) -> String {
    let to = to.min(from + FULL_TURN);
    let angle = |f: f64| f.mul_add(std::f64::consts::TAU, -std::f64::consts::FRAC_PI_2);
    let point = |r: f64, f: f64| {
        (
            r.mul_add(angle(f).cos(), 50.0),
            r.mul_add(angle(f).sin(), 50.0),
        )
    };
    let large = u8::from(to - from > 0.5);
    let (ox1, oy1) = point(OUTER_R, from);
    let (ox2, oy2) = point(OUTER_R, to);
    let (ix1, iy1) = point(INNER_R, to);
    let (ix2, iy2) = point(INNER_R, from);
    format!(
        "M{ox1:.2},{oy1:.2} A{OUTER_R},{OUTER_R} 0 {large} 1 {ox2:.2},{oy2:.2} L{ix1:.2},{iy1:.2} A{INNER_R},{INNER_R} 0 {large} 0 {ix2:.2},{iy2:.2} Z"
    )
}
