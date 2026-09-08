//! Row shaping for the approvals queue.
//!
//! The arguments are the part that matters: the digest binds an approval to
//! exactly the payload shown, so a retry that changes the payload is re-held
//! rather than silently executing. They are rendered compactly with the full
//! JSON on the row's title, because an approver who cannot see the argument is
//! not approving anything.

use chrono::Utc;
use serde::Serialize;

use crate::handlers::ssr::format::local_time;
use crate::repositories::governance::approvals::ApprovalRow;

const ARGS_CHARS: usize = 70;

#[derive(Debug, Serialize)]
pub(super) struct ApprovalRowView {
    pub(super) call_id: String,
    pub(super) created_at: String,
    pub(super) age: String,
    pub(super) tool_name: String,
    pub(super) server_name: String,
    pub(super) rule: String,
    pub(super) requested_by: String,
    pub(super) user_url: String,
    pub(super) arguments: String,
    pub(super) arguments_full: String,
    pub(super) status: String,
    pub(super) tone: &'static str,
    pub(super) actionable: bool,
    pub(super) decided_by: String,
    pub(super) trace_url: Option<String>,
}

pub(super) fn rows(rows: &[ApprovalRow]) -> Vec<ApprovalRowView> {
    rows.iter()
        .map(|r| {
            let status = r.effective_status();
            let arguments = r.arguments.to_string();
            ApprovalRowView {
                call_id: r.call_id.clone(),
                created_at: local_time(r.created_at),
                age: age_of(r),
                tool_name: r.tool_name.clone(),
                server_name: r.server_name.clone(),
                rule: r.rule.clone(),
                requested_by: r.requested_by.as_str().to_owned(),
                user_url: format!(
                    "/admin/users/{}",
                    urlencoding::encode(r.requested_by.as_str())
                ),
                arguments: truncate(&arguments),
                arguments_full: format!(
                    "{arguments}\n\ndigest {}",
                    r.args_digest.chars().take(12).collect::<String>()
                ),
                status: title_case(status),
                tone: tone_of(status),
                actionable: r.is_actionable(),
                decided_by: r
                    .approver_username
                    .clone()
                    .or_else(|| r.approver_id.clone())
                    .unwrap_or_else(|| "\u{2014}".to_owned()),
                trace_url: r
                    .trace_id
                    .as_deref()
                    .filter(|t| !t.is_empty())
                    .map(|t| format!("/admin/traces/{}", urlencoding::encode(t))),
            }
        })
        .collect()
}

// Why: the age of a pending row counts from when it was parked; a decided row's
// age is how long the caller actually waited. Two different questions, one
// column, because a queue that showed only one of them would be misleading in
// whichever state the reader happened to be looking at.
fn age_of(row: &ApprovalRow) -> String {
    let end = row.decided_at.unwrap_or_else(Utc::now);
    let minutes = (end - row.created_at).num_minutes().max(0);
    if minutes < 60 {
        return format!("{minutes}m");
    }
    format!("{}h {}m", minutes / 60, minutes % 60)
}

const fn tone_of(status: &str) -> &'static str {
    match status.as_bytes() {
        b"pending" => "warn",
        b"approved" => "ok",
        b"denied" => "err",
        _ => "muted",
    }
}

fn truncate(text: &str) -> String {
    if text.chars().count() <= ARGS_CHARS {
        return text.to_owned();
    }
    let head: String = text.chars().take(ARGS_CHARS).collect();
    format!("{head}\u{2026}")
}

pub(super) fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().collect::<String>() + chars.as_str()
    })
}
