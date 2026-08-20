//! Code-tab view builders: commit activity, AI line deltas, and the
//! measurement-frame cards. Split from `view.rs` at the 300-line ceiling;
//! the shared label helpers stay in `view.rs` and are imported here.

use crate::handlers::ssr::types::{ChartBarView, ChartView, bar_pct};
use crate::repositories::analytics::site::code::{CodeDayBucket, CodeTotals};
use crate::util::time_range::TimeRange;

use super::context::CodeFrameView;
use super::view::{compact, date_label, midpoint};

pub(super) fn commit_chart(buckets: &[CodeDayBucket], range: &TimeRange) -> ChartView {
    let max = buckets.iter().map(|b| b.commits).max().unwrap_or(0);
    let total: i64 = buckets.iter().map(|b| b.commits).sum();
    ChartView {
        title: "Commit activity",
        subtitle: format!("{total} commits observed via Claude Code sessions"),
        tone: "info",
        series: buckets
            .iter()
            .map(|b| ChartBarView {
                pct: bar_pct(b.commits, max),
                tooltip: format!(
                    "{}: {} commits, +{} −{}",
                    b.date.format("%b %d"),
                    b.commits,
                    b.commit_insertions,
                    b.commit_deletions
                ),
            })
            .collect(),
        has_data: max > 0,
        y_max_display: max.to_string(),
        y_mid_display: (max / 2).to_string(),
        x_start_display: date_label(range.from),
        x_mid_display: date_label(midpoint(range)),
        x_end_display: date_label(range.to),
        empty_message: "No commits observed in this window. Only commits made through \
                        Claude Code sessions are visible.",
    }
}

pub(super) fn loc_chart(buckets: &[CodeDayBucket], range: &TimeRange) -> ChartView {
    let max = buckets.iter().map(|b| b.loc_added_ai).max().unwrap_or(0);
    let total: i64 = buckets.iter().map(|b| b.loc_added_ai).sum();
    ChartView {
        title: "AI lines added",
        subtitle: format!("{} lines applied through Edit/Write tools", compact(total)),
        tone: "accent",
        series: buckets
            .iter()
            .map(|b| ChartBarView {
                pct: bar_pct(b.loc_added_ai, max),
                tooltip: format!(
                    "{}: +{} −{} AI lines",
                    b.date.format("%b %d"),
                    b.loc_added_ai,
                    b.loc_removed_ai
                ),
            })
            .collect(),
        has_data: max > 0,
        y_max_display: compact(max),
        y_mid_display: compact(max / 2),
        x_start_display: date_label(range.from),
        x_mid_display: date_label(midpoint(range)),
        x_end_display: date_label(range.to),
        empty_message: "No AI line data in this window yet.",
    }
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
