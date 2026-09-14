//! The governance KPI strip.
//!
//! Six tiles in one band, spanning both enforcement planes — the whole reason
//! the three tabs share a page is that the chain and the scanners are
//! configured apart and have to be read together.
//!
//! The four chain stages used to be four tiles of their own, which wrapped the
//! band onto a second row and cost the table four visible entries. They are the
//! one-line strip under the band instead — still four numbers visible without a
//! click, still one click each to the rows behind them, at a quarter of the
//! height.

use serde::Serialize;

use super::GovernanceQuery;
use super::data::GovernanceData;
use super::urls::filter_url;
use crate::repositories::governance::decision_log::DecisionStats;
use crate::repositories::governance::findings::SafetyStats;

// Why: One KPI tile, as `components/kpi` reads it. The supporting line is
// `note` on the partial; the field keeps the name `sub` and the template reads
// it as `this.sub`, because a bare `sub` mustache resolves to the registered
// helper of that name rather than to a field.
#[derive(Debug, Serialize)]
pub(super) struct GovernanceKpiView {
    label: &'static str,
    value: String,
    sub: String,
    tone: &'static str,
    href: String,
    active: bool,
    hint: &'static str,
}

pub(super) fn kpis(query: &GovernanceQuery, data: &GovernanceData) -> Vec<GovernanceKpiView> {
    decision_tiles(data.stats)
        .into_iter()
        .chain(safety_tiles(data.safety))
        .map(|t| tile(query, t))
        .collect()
}

fn decision_tiles(s: DecisionStats) -> Vec<Tile> {
    vec![
        Tile {
            label: "Calls",
            value: s.calls,
            sub: format!("{} evaluations · {} callers", s.evaluated, s.distinct_users),
            tone: "",
            filter: None,
            hint: "Governed calls in this window. Each is one row in the log, folded \
     from every policy evaluation that shared its trace",
        },
        Tile {
            label: "Review groups",
            value: s.attention_calls,
            sub: "Repeated warnings grouped by caller and session".to_owned(),
            tone: "err",
            filter: Some(("attention", "1")),
            hint: "Denials and actionable warnings grouped by session and evidence. Entropy-only observations remain in the audit log",
        },
        Tile {
            label: "Denied",
            value: s.denied,
            sub: format!("{} of evaluated", percent(s.denied, s.evaluated)),
            tone: "err",
            filter: Some(("decision", "deny")),
            hint: "Refused outright by one of the four chain stages",
        },
        Tile {
            label: "Warned",
            value: s.warned,
            sub: format!("{} of evaluated", percent(s.warned, s.evaluated)),
            tone: "warn",
            filter: Some(("decision", "warn")),
            hint: "Warnings recorded by policies; inspect the evidence to distinguish observation, sanitization and refusal",
        },
        Tile {
            label: "Allowed",
            value: s.allowed,
            sub: format!("{} of evaluated", percent(s.allowed, s.evaluated)),
            tone: "ok",
            filter: Some(("decision", "allow")),
            hint: "No rule matched, or every rule that matched permitted the call",
        },
    ]
}

fn safety_tiles(f: SafetyStats) -> Vec<Tile> {
    vec![
        Tile {
            label: "Safety findings",
            value: f.findings,
            sub: format!("{} categories, both directions", f.categories),
            tone: "",
            filter: Some(("tab", "safety")),
            hint: "Gateway scanner findings on requests and on responses",
        },
        Tile {
            label: "Findings blocked",
            value: f.blocked,
            sub: format!("{} audited only", f.audited),
            tone: "err",
            filter: Some(("tab", "safety")),
            hint: "Actual recorded blocks. Informational findings and warn-mode findings do not imply a refusal",
        },
    ]
}

// Why: One policy's showing in the window, as a filter link.
#[derive(Debug, Serialize)]
pub(super) struct StageFilterView {
    label: String,
    count: i64,
    href: String,
    active: bool,
    tone: &'static str,
}

// Why: the strip used to name the four synchronous chain stages and count only
// those. On an instance whose traffic is `authz`, `default_allow` and
// `authentication` that is four zeroes sitting above a log of twenty thousand
// rows, which reads as "nothing is happening" — the exact opposite of the
// truth.
//
// It is built from what the window actually holds instead, ordered so the
// policies that denied or warned come first. A chip exists because a policy
// fired; its count is the attention it drew, and a policy that only ever allows
// is muted rather than absent, because "this ran and objected to nothing" is
// also worth reading.
pub(super) fn stage_filters(
    query: &GovernanceQuery,
    data: &GovernanceData,
) -> Vec<StageFilterView> {
    data.policy_counts
        .iter()
        .map(|p| {
            let attention = p.denied + p.warned;
            StageFilterView {
                label: p.policy.clone(),
                count: if attention > 0 { attention } else { p.total },
                href: filter_url(query, &[("tab", "decisions"), ("policy", &p.policy)]),
                active: query.policy.as_deref() == Some(p.policy.as_str()),
                tone: if p.denied > 0 {
                    "err"
                } else if p.warned > 0 {
                    "warn"
                } else {
                    "muted"
                },
            }
        })
        .collect()
}

struct Tile {
    label: &'static str,
    value: i64,
    sub: String,
    tone: &'static str,
    filter: Option<(&'static str, &'static str)>,
    hint: &'static str,
}

fn tile(query: &GovernanceQuery, tile: Tile) -> GovernanceKpiView {
    let Tile {
        label,
        value,
        sub,
        tone,
        filter,
        hint,
    } = tile;
    let (href, active) = filter.map_or_else(
        || (filter_url(query, &[]), false),
        |(name, value)| {
            let current = match name {
                "decision" => query.decision.as_deref(),
                "policy" => query.policy.as_deref(),
                _ => query.tab.as_deref(),
            };
            (filter_url(query, &[(name, value)]), current == Some(value))
        },
    );
    GovernanceKpiView {
        label,
        value: value.to_string(),
        sub,
        tone,
        href,
        active,
        hint,
    }
}

fn percent(part: i64, whole: i64) -> String {
    if whole <= 0 {
        return "0%".to_owned();
    }
    format!("{:.1}%", (part as f64 / whole as f64) * 100.0f64)
}
