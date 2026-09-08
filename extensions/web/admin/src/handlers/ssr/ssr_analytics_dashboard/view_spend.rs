//! Spend-tab view builders: the usage-anomaly rows, the fast/slow latency
//! split, and the client-reported session-cost cards.
//!
//! Split from `view.rs` at the 300-line ceiling; the shared label helpers
//! stay there and are imported here.

use crate::handlers::ssr::format::{format_cost, format_duration_ms};
use crate::repositories::analytics::site::anomalies::UsageAnomalyRow;
use crate::repositories::analytics::site::kpis::SiteKpis;
use crate::repositories::analytics::site::latency::LatencySplit;
use crate::repositories::analytics::site::session_costs::SessionCostStats;

use super::context::{AnomalyRowView, FastSlowView, SessionCostsView, ThinkingView};
use super::view::compact;

// Why: cost renders as dollars and the counting metrics as counts; the rows
// come from one table, so the discrimination lives here rather than in SQL.
pub(super) fn anomaly_rows(rows: &[UsageAnomalyRow]) -> Vec<AnomalyRowView> {
    rows.iter()
        .map(|r| {
            let (observed, baseline) = if r.metric == "cost" {
                (format_cost(r.observed), format_cost(r.baseline))
            } else {
                (r.observed.to_string(), r.baseline.to_string())
            };
            AnomalyRowView {
                metric: r.metric.clone(),
                window_display: r.window_start.format("%Y-%m-%d %H:%M UTC").to_string(),
                observed_display: observed,
                baseline_display: baseline,
            }
        })
        .collect()
}

// Why: this platform has no fast/slow request pools, so the split is stated
// as what it actually is — a latency bucket at the caller's SLO threshold —
// with the percentiles and breach share beside it and untimed requests shown
// rather than folded away. The displays derive from the threshold the query
// actually bound, so the caption can never contradict the split.
pub(super) fn fast_slow(split: &LatencySplit) -> FastSlowView {
    let timed = split.fast + split.slow;
    let threshold = format_duration_ms(i64::from(split.threshold_ms));
    FastSlowView {
        within_label: format!("Within SLO (<{threshold})"),
        breach_label: format!("Breaching SLO (>={threshold})"),
        fast: split.fast,
        slow: split.slow,
        untimed: split.untimed,
        threshold_display: threshold,
        breach_pct_display: if timed > 0 {
            let permille = split.slow.saturating_mul(1000) / timed;
            format!("{}.{}%", permille / 10, permille % 10)
        } else {
            "–".to_owned()
        },
        p50_display: format_duration_ms(split.p50_ms.round() as i64),
        p95_display: format_duration_ms(split.p95_ms.round() as i64),
        has_data: timed + split.untimed > 0,
    }
}

pub(super) fn session_costs(stats: &SessionCostStats) -> SessionCostsView {
    SessionCostsView {
        has_data: stats.sessions > 0,
        sessions: stats.sessions,
        cache_hit_display: format!("{:.0}%", stats.cache_hit_pct),
        cache_read_display: compact(stats.cache_read_tokens),
        avg_context_display: compact(stats.avg_context_window),
        max_context_display: compact(stats.max_context_window),
    }
}

// Why: reasoning tokens are billed inside the output count, so the card states
// the share of output they took rather than a figure that looks additive to
// the total beside it.
pub(super) fn thinking(kpis: &SiteKpis) -> ThinkingView {
    ThinkingView {
        has_data: kpis.reasoning_tokens > 0,
        reasoning_display: compact(kpis.reasoning_tokens),
        output_display: compact(kpis.output_tokens),
        share_display: if kpis.output_tokens > 0 {
            let permille = kpis.reasoning_tokens.saturating_mul(1000) / kpis.output_tokens;
            format!("{}.{}%", permille / 10, permille % 10)
        } else {
            "–".to_owned()
        },
    }
}
