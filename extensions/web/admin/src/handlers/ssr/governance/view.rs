//! Row and tile shaping for the governance page.
//!
//! The repository answers in domain rows; the template reads strings and URLs.
//! Everything in between — badge tones, the stage a policy belongs to, the
//! reason truncation — happens here, so the template holds no logic and the
//! handler holds no formatting.

use serde::Serialize;
use systemprompt::identifiers::UserId;

use crate::handlers::ssr::format::local_time;
use crate::repositories::governance::decision_log::DecisionLogRow;
use crate::repositories::governance::findings::SafetyFindingLogRow;
use crate::repositories::governance::hook_events::RecentHookEvent;

// Why: the reason column is free text a policy wrote, and one long reason used
// to widen the table until the row count fell from 20 to 11. Truncating to a
// readable clause and carrying the full text in `title` keeps the density and
// loses nothing.
const REASON_CHARS: usize = 90;

#[derive(Debug, Serialize)]
pub(super) struct DecisionRow {
    pub(super) created_at: String,
    pub(super) decision: String,
    pub(super) tone: &'static str,
    pub(super) policy: String,
    pub(super) stage: &'static str,
    pub(super) tool_name: String,
    pub(super) user_id: UserId,
    pub(super) user_url: String,
    pub(super) scope: String,
    pub(super) reason: String,
    pub(super) reason_full: String,
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
    pub(super) plugin_id: String,
    pub(super) user_id: UserId,
    pub(super) status: String,
    pub(super) tone: &'static str,
}

// Why: The four synchronous chain stages, named as the policy writes them.
pub(super) const fn stage_of(policy: &str) -> &'static str {
    match policy.as_bytes() {
        b"agent_scope" => "1 scope",
        b"secret_scan" => "2 secret",
        b"tool_blocklist" => "3 blocklist",
        b"rate_limit" => "4 rate",
        _ => "\u{2014}",
    }
}

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

pub(super) fn decision_rows(rows: &[DecisionLogRow]) -> Vec<DecisionRow> {
    rows.iter()
        .map(|r| DecisionRow {
            created_at: local_time(r.created_at),
            decision: r.decision.clone(),
            tone: decision_tone(&r.decision),
            policy: r.policy.clone(),
            stage: stage_of(&r.policy),
            tool_name: r.tool_name.clone(),
            user_id: r.user_id.clone(),
            user_url: format!("/admin/users/{}", urlencoding::encode(r.user_id.as_str())),
            scope: dash(r.agent_scope.as_deref()),
            reason: truncate(&r.reason),
            reason_full: r.reason.clone(),
            detail_url: format!("/admin/governance/decisions/{}", urlencoding::encode(&r.id)),
            trace_url: r
                .trace_id
                .as_deref()
                .filter(|t| !t.is_empty())
                .map(|t| format!("/admin/traces/{}", urlencoding::encode(t))),
        })
        .collect()
}

pub(super) fn finding_rows(rows: &[SafetyFindingLogRow]) -> Vec<FindingRow> {
    rows.iter()
        .map(|r| FindingRow {
            created_at: local_time(r.created_at),
            category: r.category.clone(),
            scanner: r.scanner.clone(),
            severity: r.severity.clone(),
            severity_tone: severity_tone(&r.severity),
            direction: if r.phase == "request" {
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
                plugin_id: dash(
                    e.plugin_id
                        .as_ref()
                        .map(systemprompt::identifiers::PluginId::as_str),
                ),
                user_id: e.user_id.clone(),
                tone: decision_tone(&status),
                status,
            }
        })
        .collect()
}
