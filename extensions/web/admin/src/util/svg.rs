//! Pure geometry for the server-rendered SVG charts.
//!
//! Every chart on the analytics pages is drawn in a fixed unit space —
//! `viewBox="0 0 100 40"` stretched with `preserveAspectRatio="none"` and
//! `vector-effect="non-scaling-stroke"` — so all coordinate math happens here
//! in Rust and the Handlebars partials only print attributes. Axis labels stay
//! in HTML gutters exactly like the CSS bar charts, which keeps this module
//! free of text layout entirely.

/// Plot-space width: x spans `0..=PLOT_W`.
pub const PLOT_W: f64 = 100.0;
/// Plot-space height: y spans `0..=PLOT_H`, with 0 at the top (SVG-style).
pub const PLOT_H: f64 = 40.0;

/// Map a series onto plot space against a shared `y_max`.
///
/// The scale is never per-series — every series on one chart must share one
/// or the chart lies. X sits at bucket centers; y is inverted and clamped.
/// Empty input or a non-positive max yields no points.
#[must_use]
pub fn scale_points(values: &[i64], y_max: i64) -> Vec<(f64, f64)> {
    if values.is_empty() || y_max <= 0 {
        return Vec::new();
    }
    let n = values.len() as f64;
    let max = y_max as f64;
    values
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            let x = (i as f64 + 0.5) / n * PLOT_W;
            let y = (PLOT_H * (1.0 - v as f64 / max)).clamp(0.0, PLOT_H);
            (x, y)
        })
        .collect()
}

/// `M x0,y0 L x1,y1 …` with two-decimal coordinates. Empty points yield "".
#[must_use]
pub fn line_path(points: &[(f64, f64)]) -> String {
    let mut d = String::new();
    for (i, (x, y)) in points.iter().enumerate() {
        let cmd = if i == 0 { 'M' } else { 'L' };
        d.push_str(&format!("{cmd}{x:.2},{y:.2} "));
    }
    d.trim_end().to_owned()
}

/// The line path closed down to the baseline for an area fill. Empty yields "".
#[must_use]
pub fn area_path(points: &[(f64, f64)]) -> String {
    if points.is_empty() {
        return String::new();
    }
    let line = line_path(points);
    let (first_x, _) = points[0];
    let (last_x, _) = points[points.len() - 1];
    format!("{line} L{last_x:.2},{PLOT_H} L{first_x:.2},{PLOT_H} Z")
}

/// Y for a horizontal reference line at `value` on the shared scale (the cap
/// and warn lines on the burn-up chart). `None` when the value is off-plot.
#[must_use]
pub fn ref_line_y(value: i64, y_max: i64) -> Option<f64> {
    if y_max <= 0 || value < 0 || value > y_max {
        return None;
    }
    Some(PLOT_H * (1.0 - value as f64 / y_max as f64))
}

/// Running cumulative sum, for burn-up series.
#[must_use]
pub fn cumulative(values: &[i64]) -> Vec<i64> {
    let mut total = 0;
    values
        .iter()
        .map(|v| {
            total += v;
            total
        })
        .collect()
}

/// Stacked-bar segments for one bucket: `(y, height)` pairs in plot space,
/// stacked bottom-up in input order against the shared `y_max`. Zero segments
/// yield zero heights (the caller skips them).
#[must_use]
pub fn stack_segments(segment_values: &[i64], y_max: i64) -> Vec<(f64, f64)> {
    if y_max <= 0 {
        return segment_values.iter().map(|_| (PLOT_H, 0.0)).collect();
    }
    let max = y_max as f64;
    let mut base = 0.0; // accumulated height from the baseline, in plot units
    segment_values
        .iter()
        .map(|&v| {
            let h = (v.max(0) as f64 / max * PLOT_H).min(PLOT_H - base);
            base += h;
            (PLOT_H - base, h)
        })
        .collect()
}

/// Bar layout: `(x, width)` for `n` bars across the plot, with `gap_ratio` of
/// each slot left as spacing (0.0 = touching, 0.5 = half the slot is gap).
#[must_use]
pub fn bar_slots(n: usize, gap_ratio: f64) -> Vec<(f64, f64)> {
    if n == 0 {
        return Vec::new();
    }
    let slot = PLOT_W / n as f64;
    let gap = slot * gap_ratio.clamp(0.0, 0.9);
    let width = slot - gap;
    (0..n)
        .map(|i| ((i as f64).mul_add(slot, gap / 2.0), width))
        .collect()
}

/// The smallest of `1|2|2.5|5 × 10^k >= raw_max`, so the top gridline label is
/// a round number. Non-positive input yields 0.
#[must_use]
pub fn nice_max(raw_max: i64) -> i64 {
    if raw_max <= 0 {
        return 0;
    }
    let mut magnitude: i64 = 1;
    loop {
        for ladder in [1, 2, 3, 5] {
            // Why: 2.5 only makes sense once a decade has room for it — at
            // magnitude 1 the ladder is 1/2/3/5 so small counts stay integral.
            let step = if ladder == 3 {
                if magnitude >= 10 {
                    magnitude * 5 / 2
                } else {
                    continue;
                }
            } else {
                ladder * magnitude
            };
            if step >= raw_max {
                return step;
            }
        }
        magnitude = magnitude.saturating_mul(10);
    }
}
