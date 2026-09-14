//! Pure-geometry tests for the SVG chart utilities and delta math.

use systemprompt_web_admin::util::delta::{DeltaKind, delta};
use systemprompt_web_admin::util::svg::{
    PLOT_H, PLOT_W, area_path, bar_slots, cumulative, line_path, nice_max, ref_line_y,
    scale_points, stack_segments,
};

#[test]
fn scale_points_inverts_y_and_centers_x() {
    let pts = scale_points(&[0, 5, 10], 10);
    assert_eq!(pts.len(), 3);
    // X strictly increasing, at bucket centers.
    assert!(pts[0].0 < pts[1].0 && pts[1].0 < pts[2].0);
    assert!((pts[0].0 - PLOT_W / 6.0).abs() < 1e-9);
    // Bigger value -> smaller y (SVG y grows downward).
    assert!((pts[0].1 - PLOT_H).abs() < 1e-9);
    assert!((pts[2].1 - 0.0).abs() < 1e-9);
    assert!(pts[1].1 < pts[0].1 && pts[2].1 < pts[1].1);
}

#[test]
fn scale_points_clamps_overflow_to_plot() {
    let pts = scale_points(&[20], 10);
    assert!((pts[0].1 - 0.0).abs() < 1e-9, "over-max clamps to the top");
}

#[test]
fn scale_points_empty_and_zero_max_yield_nothing() {
    assert!(scale_points(&[], 10).is_empty());
    assert!(scale_points(&[1, 2], 0).is_empty());
}

#[test]
fn line_path_shape() {
    let d = line_path(&scale_points(&[1, 2, 3], 3));
    assert!(d.starts_with('M'));
    assert_eq!(d.matches('L').count(), 2);
    assert_eq!(line_path(&[]), "");
}

#[test]
fn area_path_closes_to_baseline() {
    let d = area_path(&scale_points(&[1, 2], 2));
    assert!(d.ends_with('Z'));
    assert!(d.contains(&format!(",{PLOT_H}")), "touches the baseline");
    assert_eq!(area_path(&[]), "");
}

#[test]
fn cumulative_is_monotone_for_non_negative_input() {
    assert_eq!(cumulative(&[1, 2, 3]), vec![1, 3, 6]);
    assert_eq!(cumulative(&[]), Vec::<i64>::new());
}

#[test]
fn ref_line_y_bounds() {
    assert_eq!(ref_line_y(0, 10), Some(PLOT_H));
    assert_eq!(ref_line_y(10, 10), Some(0.0));
    assert_eq!(ref_line_y(11, 10), None);
    assert_eq!(ref_line_y(5, 0), None);
}

#[test]
fn stack_segments_heights_sum_to_scaled_total() {
    let segs = stack_segments(&[2, 3, 5], 10);
    let total_h: f64 = segs.iter().map(|(_, h)| h).sum();
    assert!((total_h - PLOT_H).abs() < 1e-9);
    // Bottom-up: first segment sits on the baseline.
    assert!((segs[0].0 + segs[0].1 - PLOT_H).abs() < 1e-9);
    // Order preserved: each segment's top is the next one's base.
    assert!((segs[1].0 + segs[1].1 - segs[0].0).abs() < 1e-9);
}

#[test]
fn stack_segments_zero_max_yields_flat_zeros() {
    let segs = stack_segments(&[1, 2], 0);
    assert!(
        segs.iter()
            .all(|&(y, h)| (y - PLOT_H).abs() < 1e-9 && h == 0.0)
    );
}

#[test]
fn bar_slots_partition_the_plot() {
    let slots = bar_slots(4, 0.2);
    assert_eq!(slots.len(), 4);
    let (last_x, last_w) = slots[3];
    assert!(last_x + last_w <= PLOT_W + 1e-9);
    assert!(bar_slots(0, 0.2).is_empty());
}

#[test]
fn nice_max_ladder() {
    assert_eq!(nice_max(0), 0);
    assert_eq!(nice_max(-3), 0);
    assert_eq!(nice_max(1), 1);
    assert_eq!(nice_max(7), 10);
    assert_eq!(nice_max(10), 10);
    assert_eq!(nice_max(11), 20);
    assert_eq!(nice_max(130), 200);
    assert_eq!(nice_max(201), 250);
    assert_eq!(nice_max(2600), 5000);
}

#[test]
fn delta_branches() {
    let d = delta(0, 0, true);
    assert_eq!(d.display_kind, DeltaKind::None);
    assert_eq!(d.display(), "\u{2014}");
    assert_eq!(d.tone, "neutral");

    let d = delta(5, 0, true);
    assert_eq!(d.display_kind, DeltaKind::New);
    assert_eq!(d.display(), "new");
    assert_eq!((d.direction, d.tone), ("up", "good"));

    let d = delta(112, 100, true);
    assert_eq!(d.display(), "+12.0%");
    assert_eq!((d.direction, d.tone), ("up", "good"));

    // Cost going up is bad when up_is_good = false.
    let d = delta(112, 100, false);
    assert_eq!((d.direction, d.tone), ("up", "bad"));

    let d = delta(88, 100, false);
    assert_eq!(d.display(), "\u{2212}12.0%");
    assert_eq!((d.direction, d.tone), ("down", "good"));

    let d = delta(100, 100, true);
    assert_eq!((d.direction, d.tone), ("flat", "neutral"));
}
