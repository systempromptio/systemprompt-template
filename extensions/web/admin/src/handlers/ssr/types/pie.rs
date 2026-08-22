//! Distribution pie view-model: server-precomputed conic-gradient stops.
//!
//! Split from `charts.rs` at the 300-line ceiling; same design rule — the
//! view carries every derived value so the template does no arithmetic.

use serde::Serialize;

// Why: the disc is decoration over a precomputed conic-gradient stop list; the
// legend is the accessible representation. The view carries the whole stop
// string so the template does no arithmetic, per this module's design rule.
#[derive(Debug, Serialize)]
pub(crate) struct PieView {
    // Why: serialized as chart_title — the layout partial's `title=` hash
    // param shadows a context field named `title` inside nested partials.
    #[serde(rename = "chart_title")]
    pub title: &'static str,
    pub subtitle: String,
    pub has_data: bool,
    pub stops: String,
    pub aria_label: String,
    pub legend: Vec<PieSliceView>,
    pub empty_message: &'static str,
}

#[derive(Debug, Serialize)]
pub(crate) struct PieSliceView {
    pub label: String,
    pub color_index: usize,
    pub share_display: String,
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

const PIE_COLORS: usize = 7;

// Why: slices arrive sorted descending; everything past the sixth folds into
// "Other" so the disc stays readable and the seven swatch tokens always
// suffice. Stop percentages are cumulative and rounded to one decimal, with
// the final stop pinned to 100% so rounding can never leave a sliver.
pub(crate) fn pie_view(
    title: &'static str,
    subtitle: String,
    slices: Vec<PieSliceInput>,
    empty_message: &'static str,
) -> PieView {
    let total: i64 = slices.iter().map(|s| s.value).sum();
    if total <= 0 {
        return PieView {
            title,
            subtitle,
            has_data: false,
            stops: String::new(),
            aria_label: String::new(),
            legend: Vec::new(),
            empty_message,
        };
    }

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

    let mut stops: Vec<String> = Vec::with_capacity(kept.len());
    let mut legend: Vec<PieSliceView> = Vec::with_capacity(kept.len());
    let mut cumulative = 0.0f64;
    let count = kept.len();
    for (i, slice) in kept.into_iter().enumerate() {
        let share = slice.value as f64 / total as f64 * 100.0;
        let from = cumulative;
        cumulative += share;
        let to = if i + 1 == count { 100.0 } else { cumulative };
        let color = PIE_COLOR_TOKENS[i.min(PIE_COLORS - 1)];
        stops.push(format!("var({color}) {from:.1}% {to:.1}%"));
        legend.push(PieSliceView {
            label: slice.label,
            color_index: i + 1,
            share_display: format!("{share:.1}%"),
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
        stops: stops.join(", "),
        aria_label,
        legend,
        empty_message,
    }
}

const PIE_COLOR_TOKENS: [&str; PIE_COLORS] = [
    "--sp-chart-purple",
    "--sp-chart-blue",
    "--sp-chart-green",
    "--sp-chart-amber",
    "--sp-chart-red",
    "--sp-chart-cyan",
    "--sp-chart-indigo",
];
