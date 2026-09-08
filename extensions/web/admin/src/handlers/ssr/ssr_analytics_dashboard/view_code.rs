//! Code-tab view builders: commit activity, AI line deltas, and the
//! measurement-frame cards. Split from `view.rs` at the 300-line ceiling;
//! the shared label helpers stay in `view.rs` and are imported here.

use crate::handlers::ssr::types::{LineChartSpec, SvgLineChartView, SvgSeriesInput, line_chart};
use crate::repositories::analytics::site::code::{CodeDayBucket, CodeTotals};
use crate::util::time_range::TimeRange;

use super::context::CodeFrameView;
use super::view::{compact, date_label, midpoint};

pub(super) fn commit_chart(buckets: &[CodeDayBucket], range: &TimeRange) -> SvgLineChartView {
    let commits: i64 = buckets.iter().map(|b| b.commits).sum();
    let insertions: i64 = buckets.iter().map(|b| b.commit_insertions).sum();
    line_chart(LineChartSpec {
        title: "Commit activity",
        subtitle: format!(
            "{commits} commits observed via Claude Code sessions · {} lines inserted",
            compact(insertions)
        ),
        empty_message: "No commits observed in this window. Only commits made through \
                        Claude Code sessions are visible.",
        // Why: commits alone. Plotting them beside inserted lines put a
        // ~50x magnitude gap on one shared axis, which flattened the commit
        // series onto the baseline — the line totals have their own chart.
        series: vec![SvgSeriesInput {
            label: "commits".to_owned(),
            values: buckets.iter().map(|b| b.commits).collect(),
            value_display: commits.to_string(),
        }],
        ref_lines: Vec::new(),
        y_max: None,
        y_display: compact,
        x_start_display: date_label(range.from),
        x_mid_display: date_label(midpoint(range)),
        x_end_display: date_label(range.to),
        show_area: true,
    })
}

// Why: the two series are deliberately plotted together and never subtracted
// — AI lines come from hook-observed tool inputs, committed lines from git
// diff stats, and the gap between them is not "manual lines", it is two
// different measurement frames.
pub(super) fn loc_chart(buckets: &[CodeDayBucket], range: &TimeRange) -> SvgLineChartView {
    let ai: i64 = buckets.iter().map(|b| b.loc_added_ai).sum();
    let committed: i64 = buckets.iter().map(|b| b.commit_insertions).sum();
    line_chart(LineChartSpec {
        title: "AI lines vs committed lines",
        subtitle: format!(
            "{} AI lines applied · {} lines committed (different measurement frames)",
            compact(ai),
            compact(committed)
        ),
        empty_message: "No AI line data in this window yet.",
        series: vec![
            SvgSeriesInput {
                label: "AI lines added".to_owned(),
                values: buckets.iter().map(|b| b.loc_added_ai).collect(),
                value_display: compact(ai),
            },
            SvgSeriesInput {
                label: "committed lines".to_owned(),
                values: buckets.iter().map(|b| b.commit_insertions).collect(),
                value_display: compact(committed),
            },
        ],
        ref_lines: Vec::new(),
        y_max: None,
        y_display: compact,
        x_start_display: date_label(range.from),
        x_mid_display: date_label(midpoint(range)),
        x_end_display: date_label(range.to),
        show_area: false,
    })
}

// Why: the two line measurements come from different frames (hook-observed
// tool inputs vs git diff stats) and are captioned as such — the page never
// subtracts one from the other to fabricate a "manual lines" number.
pub(super) fn code_frames(totals: &CodeTotals) -> Vec<CodeFrameView> {
    vec![
        CodeFrameView {
            title: "AI lines added",
            value_display: compact(totals.loc_added_ai),
            caption: "Lines applied through Edit/Write tool calls (hook-observed). \
                      A whole-file Write counts every line as added.",
        },
        CodeFrameView {
            title: "AI lines removed",
            value_display: compact(totals.loc_removed_ai),
            caption: "Lines replaced through Edit tool calls (hook-observed).",
        },
        CodeFrameView {
            title: "Committed lines",
            value_display: format!(
                "+{} −{}",
                compact(totals.commit_insertions),
                compact(totals.commit_deletions)
            ),
            caption: "Git diff totals — AI and manual lines together, only for commits \
                      made inside tracked sessions. A different measurement frame from \
                      AI lines; the two are not comparable line-for-line.",
        },
        CodeFrameView {
            title: "Commits",
            value_display: compact(totals.commits),
            caption: "Commits observed via Claude Code Bash calls. Commits from other \
                      terminals are not visible.",
        },
        CodeFrameView {
            title: "AI edit operations",
            value_display: compact(totals.ai_edit_operations),
            caption: "Applied edits — Claude Code emits no accept/reject signal, so \
                      there is no acceptance rate to report.",
        },
    ]
}
