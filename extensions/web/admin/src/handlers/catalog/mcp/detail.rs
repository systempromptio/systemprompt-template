//! Assembling `/admin/mcp/{id}`: the tools, the call log, the attached
//! sessions, the grants, and the declaration.
//!
//! The page answers four questions in that order — what it serves, what it did,
//! who is on it, and who may reach it — because that is the order an operator
//! asks them when a server misbehaves. The declaration comes last: it is the
//! thing they will go and edit once the first four have told them what is
//! wrong.

use crate::handlers::ssr::format::{format_duration_ms, local_time, short_id};
use crate::repositories::mcp::runtime::{McpExecutionRow, McpSessionRow, McpToolStat};
use crate::repositories::overview::liveness::{HEARTBEAT_INTERVAL_SECS, liveness_state};
use crate::types::McpServerDetail;
use crate::types::access_control::AccessControlRule;

use super::view::{
    ConfigFactView, McpExecutionRowView, McpGrantRow, McpSessionRowView, McpToolRow,
};

#[expect(
    clippy::cast_precision_loss,
    reason = "call counts are far below the f64 mantissa; this value is only displayed"
)]
fn percentage(part: i64, whole: i64) -> f64 {
    if whole == 0 {
        return 0.0;
    }
    (part as f64 / whole as f64) * 100.0
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "an averaged millisecond figure is rendered to whole milliseconds"
)]
const fn whole_ms(value: f64) -> i64 {
    value.round() as i64
}

fn error_rate(calls: i64, failures: i64) -> (String, &'static str) {
    if calls == 0 {
        return ("\u{2014}".to_owned(), "muted");
    }
    let pct = percentage(failures, calls);
    let tone = if pct >= 10.0 {
        "err"
    } else if pct > 0.0 {
        "warn"
    } else {
        "ok"
    };
    (format!("{pct:.1}%"), tone)
}

fn ms(value: Option<f64>) -> String {
    value.map_or_else(
        || "\u{2014}".to_owned(),
        |v| format_duration_ms(whole_ms(v)),
    )
}

pub(super) fn tool_rows(stats: Vec<McpToolStat>) -> Vec<McpToolRow> {
    stats
        .into_iter()
        .map(|s| {
            let (error_rate_display, error_tone) = error_rate(s.calls, s.failures);
            McpToolRow {
                tool_name: s.tool_name,
                calls: s.calls,
                failures: s.failures,
                error_rate_display,
                error_tone,
                distinct_users: s.distinct_users,
                avg_display: ms(s.avg_ms),
                max_display: s.max_ms.map_or_else(
                    || "\u{2014}".to_owned(),
                    |v| format_duration_ms(i64::from(v)),
                ),
                last_call_display: s
                    .last_call_at
                    .map_or_else(|| "\u{2014}".to_owned(), local_time),
            }
        })
        .collect()
}

fn execution_tone(status: &str) -> &'static str {
    match status {
        "success" => "ok",
        "failed" => "err",
        "timeout" => "warn",
        _ => "muted",
    }
}

pub(super) fn execution_rows(rows: Vec<McpExecutionRow>) -> Vec<McpExecutionRowView> {
    rows.into_iter()
        .map(|r| {
            let session = r
                .session_id
                .as_ref()
                .map(|s| s.as_str().to_owned())
                .unwrap_or_default();
            McpExecutionRowView {
                short_id: short_id(&r.execution_id),
                status_tone: execution_tone(&r.status),
                started_display: local_time(r.started_at),
                duration_display: r.execution_time_ms.map_or_else(
                    || "\u{2014}".to_owned(),
                    |v| format_duration_ms(i64::from(v)),
                ),
                user_url: format!("/admin/users/{}", r.user_id),
                session_url: (!session.is_empty()).then(|| format!("/admin/sessions/{session}")),
                trace_url: r.trace_id.as_ref().map(|t| format!("/admin/traces/{t}")),
                session,
                caller: r.user_id.as_str().to_owned(),
                error_message: r.error_message.unwrap_or_default(),
                execution_id: r.execution_id,
                tool_name: r.tool_name,
                status: r.status,
            }
        })
        .collect()
}

pub(super) fn session_rows(rows: Vec<McpSessionRow>) -> Vec<McpSessionRowView> {
    let now = chrono::Utc::now();
    rows.into_iter()
        .map(|r| {
            // Why: a closed session is never alive however recently it spoke —
            // the heartbeat rule answers "has this beaten lately", and only a
            // session still open can beat again.
            let alive = r.status == "active"
                && liveness_state(now, Some(r.last_activity_at), HEARTBEAT_INTERVAL_SECS)
                    .is_alive();
            let caller = r
                .user_id
                .as_ref()
                .map(|u| u.as_str().to_owned())
                .unwrap_or_default();
            McpSessionRowView {
                short_id: short_id(r.session_id.as_str()),
                status_tone: if alive {
                    "ok"
                } else if r.status == "active" {
                    "warn"
                } else {
                    "muted"
                },
                alive,
                started_display: local_time(r.created_at),
                last_activity_display: local_time(r.last_activity_at),
                expires_display: local_time(r.expires_at),
                identity_label: if r.has_proxy_identity {
                    r.proxy_user_type.unwrap_or_else(|| "proxy".to_owned())
                } else {
                    "\u{2014}".to_owned()
                },
                user_url: if caller.is_empty() {
                    String::new()
                } else {
                    format!("/admin/users/{caller}")
                },
                caller,
                session: r.session_id.as_str().to_owned(),
                status: r.status,
            }
        })
        .collect()
}

pub(super) fn grant_rows(rules: Vec<AccessControlRule>) -> Vec<McpGrantRow> {
    rules
        .into_iter()
        .map(|r| {
            let access = r.access.to_string();
            McpGrantRow {
                subject_kind: r.rule_type.as_str().to_owned(),
                subject: r.rule_value,
                is_allow: access == "allow",
                access,
                updated_display: local_time(r.updated_at),
            }
        })
        .collect()
}

// Why: The declaration, flattened into the lines the summary renders.
pub(super) fn config_facts(server: Option<&McpServerDetail>) -> Vec<ConfigFactView> {
    let Some(s) = server else {
        return vec![ConfigFactView {
            label: "Declaration",
            value: "None. This server is known only from its runtime traffic.".to_owned(),
            mono: false,
        }];
    };
    let fact =
        |label: &'static str, value: String, mono: bool| ConfigFactView { label, value, mono };
    let mut out = vec![
        fact("Type", s.server_type.clone(), false),
        fact(
            "Enabled",
            if s.enabled { "yes" } else { "no" }.to_owned(),
            false,
        ),
        fact("Port", s.port.to_string(), true),
    ];
    if !s.endpoint.is_empty() {
        out.push(fact("Endpoint", s.endpoint.clone(), true));
    }
    if !s.binary.is_empty() {
        out.push(fact("Binary", s.binary.clone(), true));
    }
    if !s.package_name.is_empty() {
        out.push(fact("Package", s.package_name.clone(), true));
    }
    out.push(fact(
        "OAuth",
        if s.oauth_required {
            "required"
        } else {
            "not required"
        }
        .to_owned(),
        false,
    ));
    out.push(fact(
        "Audience",
        if s.oauth_audience.is_empty() {
            "\u{2014}".to_owned()
        } else {
            s.oauth_audience.clone()
        },
        true,
    ));
    out.push(fact("Source", s.source_path.clone(), true));
    out
}
