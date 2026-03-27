use serde::Serialize;

pub(super) const SVG_WIDTH: f64 = 960.0;
pub(super) const SVG_HEIGHT: f64 = 280.0;

#[derive(Serialize)]
pub(super) struct AxisLabel {
    pub label: String,
    pub y: String,
}

#[derive(Serialize)]
pub(super) struct XAxisLabel {
    pub label: String,
    pub x: String,
}

pub(super) fn svg_x(j: usize, n: usize, svg_w: f64) -> f64 {
    if n > 1 {
        f64::from(u32::try_from(j).unwrap_or(0)) / f64::from(u32::try_from(n - 1).unwrap_or(1))
            * svg_w
    } else {
        svg_w / 2.0
    }
}

pub(super) fn build_svg_line(base: &[f64], n: usize, svg_w: f64, svg_h: f64, y_max: f64) -> String {
    use std::fmt::Write as _;
    let mut line = String::new();
    for (j, &total) in base.iter().enumerate() {
        let x = svg_x(j, n, svg_w);
        let y = svg_h - (total / y_max * svg_h);
        if j == 0 {
            let _ = write!(line, "M{x:.1},{y:.1}");
        } else {
            let _ = write!(line, " L{x:.1},{y:.1}");
        }
    }
    line
}

pub(super) fn build_stacked_area(
    top: &[f64],
    base: &[f64],
    n: usize,
    svg_w: f64,
    svg_h: f64,
    y_max: f64,
) -> String {
    use std::fmt::Write as _;
    let mut d = String::new();
    for (j, &y_val) in top.iter().enumerate() {
        let x = svg_x(j, n, svg_w);
        let y = svg_h - (y_val / y_max * svg_h);
        if j == 0 {
            let _ = write!(d, "M{x:.1},{y:.1}");
        } else {
            let _ = write!(d, " L{x:.1},{y:.1}");
        }
    }
    for j in (0..n).rev() {
        let x = svg_x(j, n, svg_w);
        let y = svg_h - (base[j] / y_max * svg_h);
        let _ = write!(d, " L{x:.1},{y:.1}");
    }
    d.push('Z');
    d
}

pub(super) fn build_y_labels(peak: i64, svg_h: f64, y_max: f64) -> Vec<AxisLabel> {
    let y_step = (peak / 4).max(1);
    (0..=4i64)
        .map(|i| {
            let label = i * y_step;
            let val = f64::from(i32::try_from(label).unwrap_or(0));
            let y = svg_h - (val / y_max * svg_h);
            AxisLabel {
                label: label.to_string(),
                y: format!("{y:.1}"),
            }
        })
        .collect()
}
