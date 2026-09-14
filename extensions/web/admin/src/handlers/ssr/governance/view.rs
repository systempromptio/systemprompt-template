//! Row and tile shaping for the governance page.
//!
//! The repository answers in domain rows; the template reads strings and URLs.
//! Everything in between — badge tones, the stage a policy belongs to, the
//! reason truncation — happens here, so the template holds no logic and the
//! handler holds no formatting.

use serde::Serialize;
use systemprompt::identifiers::{PluginId, UserId};

use crate::handlers::ssr::format::local_time;
use crate::repositories::governance::decision_calls::{ChainEvaluation, DecisionCallRow};
use crate::repositories::governance::findings::SafetyFindingLogRow;
use crate::repositories::governance::hook_events::RecentHookEvent;

// Why: the reason column is free text a policy wrote, and one long reason used
// to widen the table until the row count fell from 20 to 11. Truncating to a
// readable clause and carrying the full text in `title` keeps the density and
// loses nothing.
const REASON_CHARS: usize = 90;

// Why: one policy evaluation, as a pill in the row's chain. The pill is what
// makes the folded row lossless — every evaluation the group swallowed is still
// on the page, named, toned and one click from its own audit detail.
#[derive(Debug, Serialize)]
pub(super) struct ChainPill {
    pub(super) policy: String,
    pub(super) tone: &'static str,
    pub(super) title: String,
    pub(super) href: String,
}

#[derive(Debug, Serialize)]
pub(super) struct DecisionRow {
    pub(super) created_at: String,
    pub(super) span: String,
    pub(super) decision: String,
    pub(super) tone: &'static str,
    pub(super) attention: bool,
    pub(super) target: String,
    pub(super) target_title: String,
    pub(super) user_label: String,
    pub(super) user_id: UserId,
    pub(super) user_url: String,
    pub(super) reason: String,
    pub(super) reason_full: String,
    pub(super) chain: Vec<ChainPill>,
    pub(super) chain_overflow: String,
    pub(super) eval_count: i64,
    pub(super) detail_url: String,
    pub(super) trace_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct FindingRow {
    pub(super) created_at: String,
    pub(super) category: String,
    pub(super) scanner: String,
    pub(super) severity: String,
    pub(super) severity_tone: &'static str,
    pub(super) direction: &'static str,
    pub(super) outcome: &'static str,
    pub(super) outcome_tone: &'static str,
    pub(super) excerpt: String,
    pub(super) model: String,
    pub(super) request_url: String,
    pub(super) request_label: String,
}

#[derive(Debug, Serialize)]
pub(super) struct HookRow {
    pub(super) created_at: String,
    pub(super) kind: String,
    pub(super) tool_name: String,
    pub(super) plugin: String,
    pub(super) user_id: UserId,
    pub(super) status: String,
    pub(super) tone: &'static str,
}

// Why: re-exported rather than defined here. Both were private to this module
// and both had failed silently on the live console for want of a test that
// could reach them; they live in `types::governance_labels` now, where the unit
// suite pins them, and the page reads them from their old names.
pub(super) use crate::types::governance_labels::{plane_of, target_label};

pub(super) const fn decision_tone(decision: &str) -> &'static str {
    match decision.as_bytes() {
        b"deny" => "err",
        b"warn" => "warn",
        b"allow" => "ok",
        _ => "muted",
    }
}

const fn severity_tone(severity: &str) -> &'static str {
    match severity.as_bytes() {
        b"critical" | b"high" => "err",
        b"medium" => "warn",
        _ => "muted",
    }
}

fn truncate(text: &str) -> String {
    if text.chars().count() <= REASON_CHARS {
        return text.to_owned();
    }
    let head: String = text.chars().take(REASON_CHARS).collect();
    format!("{head}\u{2026}")
}

fn dash(value: Option<&str>) -> String {
    value
        .filter(|v| !v.is_empty())
        .map_or_else(|| "\u{2014}".to_owned(), ToOwned::to_owned)
}

// Why: a chain longer than this stops being readable as a strip and starts
// being a paragraph. Six covers every ordinary call — the common shape is three
// — and the overflow marker plus `eval_count` say what was left off.
const CHAIN_PILLS: usize = 6;

// Why: adjacent evaluations that agree on both policy and decision are the same
// fact recorded twice, and two identical pills side by side read as two events.
fn pills(chain: &[ChainEvaluation]) -> Vec<ChainPill> {
    let mut out: Vec<ChainPill> = Vec::new();
    let mut last: Option<(&str, &str)> = None;
    for step in chain {
        if last == Some((step.policy.as_str(), step.decision.as_str())) {
            continue;
        }
        last = Some((step.policy.as_str(), step.decision.as_str()));
        let plane = plane_of(&step.policy);
        let title = if step.reason.is_empty() {
            format!("{} · {} · {plane}", step.policy, step.decision)
        } else {
            format!(
                "{} · {} · {plane} — {}",
                step.policy, step.decision, step.reason
            )
        };
        out.push(ChainPill {
            policy: step.policy.clone(),
            tone: decision_tone(&step.decision),
            title,
            href: format!(
                "/admin/governance/decisions/{}",
                urlencoding::encode(&step.id)
            ),
        });
    }
    out
}

// Why: every distinct thing the call touched, for the cell's tooltip. The cell
// itself shows the target of the evaluation the row speaks for; a call that
// governed a prompt and a route has both, and hiding the second would make the
// visible one look like the whole story.
fn target_title(chain: &[ChainEvaluation]) -> String {
    let mut seen: Vec<String> = Vec::new();
    for step in chain {
        let label = target_label(&step.tool_name, step.entity_type.as_deref());
        if !seen.contains(&label) {
            seen.push(label);
        }
    }
    seen.join(" · ")
}

pub(super) fn decision_rows(rows: &[DecisionCallRow]) -> Vec<DecisionRow> {
    rows.iter()
        .map(|r| {
            let all = pills(&r.chain.0);
            let shown: Vec<ChainPill> = all.into_iter().take(CHAIN_PILLS).collect();
            let hidden = usize::try_from(r.eval_count)
                .unwrap_or(shown.len())
                .saturating_sub(shown.len());
            DecisionRow {
                created_at: local_time(r.started_at),
                span: span_of(r),
                decision: r.worst_decision.clone(),
                tone: decision_tone(&r.worst_decision),
                attention: r.deny_count + r.warn_count > 0,
                target: target_label(&r.worst_tool, r.worst_entity_type.as_deref()),
                target_title: target_title(&r.chain.0),
                user_label: r.user_label.clone(),
                user_id: r.user_id.clone(),
                user_url: format!("/admin/users/{}", urlencoding::encode(r.user_id.as_str())),
                // Why: an allow has nothing to explain, and the producers write
                // an empty reason for one anyway. Showing the worst evaluation's
                // reason means the column carries the objection or nothing.
                reason: truncate(&r.worst_reason),
                reason_full: r.worst_reason.clone(),
                chain: shown,
                chain_overflow: if hidden > 0 {
                    format!("+{hidden}")
                } else {
                    String::new()
                },
                eval_count: r.eval_count,
                detail_url: format!(
                    "/admin/governance/decisions/{}",
                    urlencoding::encode(&r.worst_id)
                ),
                trace_url: r
                    .trace_id
                    .as_deref()
                    .filter(|t| !t.is_empty())
                    .map(|t| format!("/admin/traces/{}", urlencoding::encode(t))),
            }
        })
        .collect()
}

// Why: how long the call's evaluations were spread over. Ordinarily
// milliseconds and not worth the ink, but it is the tell that a trace covered a
// burst of calls rather than one, which is the failure mode of grouping on it.
fn span_of(row: &DecisionCallRow) -> String {
    let ms = (row.ended_at - row.started_at).num_milliseconds();
    if ms < 1000 {
        return String::new();
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "a display duration; a span this side of f64's exact range"
    )]
    let seconds = ms as f64 / 1000.0;
    format!("+{seconds:.1}s")
}

pub(super) fn finding_rows(rows: &[SafetyFindingLogRow]) -> Vec<FindingRow> {
    rows.iter()
        .map(|r| FindingRow {
            created_at: local_time(r.created_at),
            category: r.category.clone(),
            scanner: r.scanner.clone(),
            severity: r.severity.clone(),
            severity_tone: severity_tone(&r.severity),
            direction: if r.phase == "request_history" {
                "inbound history"
            } else if r.phase == "request" {
                "inbound"
            } else {
                "outbound"
            },
            outcome: if r.blocked { "blocked" } else { "audited" },
            outcome_tone: if r.blocked { "err" } else { "muted" },
            excerpt: truncate(r.excerpt.as_deref().unwrap_or("\u{2014}")),
            model: dash(r.model.as_deref()),
            request_url: format!("/admin/requests/{}", urlencoding::encode(&r.ai_request_id)),
            request_label: short(&r.ai_request_id),
        })
        .collect()
}

// Why: the shared `short_id` rule — a fixed head plus an ellipsis. Taking the
// last dash-separated segment instead rendered `e2e-dreq-000` as "000" beside a
// UUID's twelve-character tail, so two ids in the same column had nothing in
// common to compare.
fn short(id: &str) -> String {
    if id.is_empty() {
        return "\u{2014}".to_owned();
    }
    crate::handlers::ssr::format::short_id(id)
}

pub(super) fn hook_rows(events: &[RecentHookEvent]) -> Vec<HookRow> {
    events
        .iter()
        .map(|e| {
            let status = e.status.clone().unwrap_or_else(|| "recorded".to_owned());
            HookRow {
                created_at: local_time(e.created_at),
                kind: e.kind.clone(),
                tool_name: dash(e.tool_name.as_deref()),
                plugin: dash(e.plugin_id.as_ref().map(PluginId::as_str)),
                user_id: e.user_id.clone(),
                tone: decision_tone(&status),
                status,
            }
        })
        .collect()
}
