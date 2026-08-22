//! View shapes for the demo trace, and the mapping from repository rows.
//!
//! Two transformations carry the page. The first decodes `evaluated_rules`
//! into the four policy stages the governance pipeline actually ran, so an
//! allowed call shows *why* it was allowed instead of an empty cell. The
//! second folds the flat timeline into turns: a prompt decision opens a turn
//! and everything until the next prompt belongs to it, which is what makes a
//! denial and the model call it prevented legible as one story.

use serde::Serialize;
use systemprompt::identifiers::SessionId;

use crate::repositories::governance::demo_trace::{DemoSessionRow, DemoTraceRow};

use super::super::entity_urls::session_detail_url;

#[derive(Debug, Serialize)]
pub(super) struct SessionView {
    pub session_id: SessionId,
    pub label: String,
    pub allowed: i64,
    pub denied: i64,
    pub requests: i64,
    pub model: String,
    pub has_model: bool,
    pub started_at: String,
    pub last_at: String,
    pub url: String,
    pub detail_url: String,
    pub is_active: bool,
}

// Why: one policy the governance pipeline evaluated for a single decision;
// `result` is `pass` | `fail` | `skip`, as written by the audit payload.
#[derive(Debug, Serialize)]
pub(super) struct StageView {
    pub policy: String,
    pub result: String,
    pub detail: String,
    pub is_fail: bool,
    pub is_skip: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct TraceRowView {
    pub chain_id: String,
    pub at: String,
    pub at_full: String,
    pub offset: String,
    pub kind: String,
    pub kind_label: String,
    pub subject: String,
    pub outcome: String,
    pub policy: String,
    pub has_policy: bool,
    pub detail: String,
    pub has_detail: bool,
    pub stages: Vec<StageView>,
    pub has_stages: bool,
    pub is_deny: bool,
    pub is_request: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct TurnView {
    pub ordinal: usize,
    pub prompt: String,
    pub rows: Vec<TraceRowView>,
    pub row_count: usize,
    pub denied: usize,
    pub model_calls: usize,
    pub blocked: bool,
}

fn kind_label(kind: &str) -> &'static str {
    match kind {
        "prompt" => "Prompt gate",
        "tool" => "Tool gate",
        "route" => "Model route gate",
        "request" => "Model call",
        _ => "Tool fire",
    }
}

fn to_stage_views(rules: Option<&serde_json::Value>) -> Vec<StageView> {
    let Some(chain) = rules
        .and_then(|r| r.get("chain"))
        .and_then(|c| c.as_array())
    else {
        return Vec::new();
    };
    chain
        .iter()
        .map(|stage| {
            let result = stage
                .get("result")
                .and_then(|v| v.as_str())
                .unwrap_or("pass")
                .to_owned();
            StageView {
                policy: stage
                    .get("policy_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("policy")
                    .to_owned(),
                detail: stage
                    .get("detail")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_owned(),
                is_fail: result == "fail",
                is_skip: result == "skip",
                result,
            }
        })
        .collect()
}

fn decisive_detail(reason: &str, stages: &[StageView]) -> String {
    if !reason.is_empty() {
        return reason.to_owned();
    }
    stages
        .iter()
        .find(|s| s.is_fail)
        .or_else(|| stages.iter().rev().find(|s| !s.is_skip))
        .map(|s| s.detail.clone())
        .unwrap_or_default()
}

pub(super) fn to_session_views(
    sessions: Vec<DemoSessionRow>,
    selected: Option<&SessionId>,
) -> Vec<SessionView> {
    sessions
        .into_iter()
        .map(|s| {
            let model = s.model.unwrap_or_default();
            SessionView {
                is_active: selected == Some(&s.session_id),
                url: format!("/admin/demo/trace?session={}", s.session_id),
                detail_url: session_detail_url(&s.session_id),
                label: s.started_at.format("%b %-d, %H:%M").to_string(),
                started_at: s.started_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                last_at: s.last_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                session_id: s.session_id,
                allowed: s.allowed,
                denied: s.denied,
                requests: s.requests,
                has_model: !model.is_empty(),
                model,
            }
        })
        .collect()
}

fn to_row_view(r: DemoTraceRow, origin: chrono::DateTime<chrono::Utc>) -> TraceRowView {
    let stages = to_stage_views(r.evaluated_rules.as_ref());
    let detail = decisive_detail(&r.detail, &stages);
    let elapsed = (r.at - origin).num_milliseconds().max(0);
    TraceRowView {
        chain_id: r.id,
        at: r.at.format("%H:%M:%S").to_string(),
        at_full: r.at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        offset: format!("+{:.2}s", elapsed as f64 / 1000.0),
        kind_label: kind_label(&r.kind).to_owned(),
        is_deny: r.outcome == "deny",
        is_request: r.kind == "request",
        has_policy: !r.policy.is_empty(),
        kind: r.kind,
        subject: r.subject,
        outcome: r.outcome,
        policy: r.policy,
        has_detail: !detail.is_empty(),
        detail,
        has_stages: !stages.is_empty(),
        stages,
    }
}

fn close_turn(turn: &mut TurnView) {
    turn.row_count = turn.rows.len();
    turn.denied = turn.rows.iter().filter(|r| r.is_deny).count();
    turn.model_calls = turn.rows.iter().filter(|r| r.is_request).count();
    turn.blocked = turn.model_calls == 0 && turn.denied > 0;
}

fn new_turn(ordinal: usize, prompt: &str) -> TurnView {
    TurnView {
        ordinal,
        prompt: prompt.to_owned(),
        rows: Vec::new(),
        row_count: 0,
        denied: 0,
        model_calls: 0,
        blocked: false,
    }
}

// Why: rows that arrive before any prompt (a gateway-only run has no hook
// spine) open a leading turn of their own rather than being dropped.
pub(super) fn to_turn_views(rows: Vec<DemoTraceRow>) -> Vec<TurnView> {
    let Some(origin) = rows.first().map(|r| r.at) else {
        return Vec::new();
    };

    let mut turns: Vec<TurnView> = Vec::new();
    let mut current: Option<TurnView> = None;

    for row in rows {
        if row.kind == "prompt"
            && current.as_ref().is_some_and(|t| !t.rows.is_empty())
            && let Some(mut done) = current.take()
        {
            close_turn(&mut done);
            turns.push(done);
        }
        let ordinal = turns.len() + 1;
        let turn = current.get_or_insert_with(|| {
            new_turn(
                ordinal,
                if row.kind == "prompt" {
                    "user prompt"
                } else {
                    "before the first prompt"
                },
            )
        });
        turn.rows.push(to_row_view(row, origin));
    }

    if let Some(mut done) = current.take() {
        close_turn(&mut done);
        turns.push(done);
    }
    turns
}
