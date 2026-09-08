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
    let s = data.stats;
    let f = data.safety;
    vec![
        tile(
            "Evaluated",
            s.evaluated,
            format!("{} callers in window", s.distinct_users),
            "",
            query,
            None,
            "Tool calls the governance chain decided in this window",
        ),
        tile(
            "Denied",
            s.denied,
            format!("{} of evaluated", percent(s.denied, s.evaluated)),
            "err",
            query,
            Some(("decision", "deny")),
            "Refused outright by one of the four chain stages",
        ),
        tile(
            "Warned",
            s.warned,
            format!("{} of evaluated", percent(s.warned, s.evaluated)),
            "warn",
            query,
            Some(("decision", "warn")),
            "Warn mode absorbed these; enforcement would have refused them",
        ),
        tile(
            "Allowed",
            s.allowed,
            format!("{} of evaluated", percent(s.allowed, s.evaluated)),
            "ok",
            query,
            Some(("decision", "allow")),
            "No rule matched, or every rule that matched permitted the call",
        ),
        tile(
            "Safety findings",
            f.findings,
            format!("{} categories, both directions", f.categories),
            "",
            query,
            Some(("tab", "safety")),
            "Gateway scanner findings on requests and on responses",
        ),
        tile(
            "Findings blocked",
            f.blocked,
            format!("{} audited only", f.audited),
            "err",
            query,
            Some(("tab", "safety")),
            "Findings with no blocks under them means the scanners are in warn mode",
        ),
    ]
}

// Why: One stage of the chain, as a filter link. A separate strip rather than
// four more tiles: the four numbers still read without a click and still lead
// to their rows, and the KPI band stays one row deep.
#[derive(Debug, Serialize)]
pub(super) struct StageFilterView {
    label: &'static str,
    count: i64,
    href: String,
    active: bool,
}

// Why: the four synchronous stages in evaluation order, named as the policies
// write them. Reading the strip left to right is reading the chain.
pub(super) fn stage_filters(
    query: &GovernanceQuery,
    data: &GovernanceData,
) -> Vec<StageFilterView> {
    let s = data.stats;
    [
        ("Scope", "agent_scope", s.scope_denied),
        ("Secret", "secret_scan", s.secret_denied),
        ("Blocklist", "tool_blocklist", s.blocklist_denied),
        ("Rate", "rate_limit", s.rate_denied),
    ]
    .into_iter()
    .map(|(label, policy, count)| StageFilterView {
        label,
        count,
        href: filter_url(
            query,
            &[
                ("tab", "decisions"),
                ("policy", policy),
                ("decision", "deny"),
            ],
        ),
        active: query.policy.as_deref() == Some(policy),
    })
    .collect()
}

#[expect(
    clippy::too_many_arguments,
    reason = "page query plumbing; splitting the parameters is tracked in docs/tech-debt.md"
)]
fn tile(
    label: &'static str,
    value: i64,
    sub: String,
    tone: &'static str,
    query: &GovernanceQuery,
    filter: Option<(&'static str, &'static str)>,
    hint: &'static str,
) -> GovernanceKpiView {
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
