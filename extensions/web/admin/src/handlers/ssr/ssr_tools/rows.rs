//! One tool call as the Tools and Artifacts tables show it: every figure
//! pre-formatted and toned, the kind glyph chosen, every link built.

use serde::Serialize;

use crate::handlers::ssr::analysis_urls::analysis_skill_url;
use crate::handlers::ssr::format::{format_duration_ms, local_time, relative_time, short_num};
use crate::repositories::analysis::tools::ToolActivityRow;

const INPUT_CLIP: usize = 72;

#[derive(Debug, Serialize)]
pub(crate) struct ToolActivityRowView {
    pub row_key: String,
    pub at: String,
    pub at_full: String,
    pub tool_name: String,
    pub tool_icon: &'static str,
    pub is_builtin: bool,
    pub server_name: String,
    pub input_summary: String,
    pub input_full: String,
    pub is_path: bool,
    pub state: String,
    pub state_tone: &'static str,
    pub status: String,
    pub duration_display: String,
    pub decision: String,
    pub decision_tone: &'static str,
    pub decision_href: Option<String>,
    pub user_key: String,
    pub user_label: String,
    pub user_href: Option<String>,
    pub session_href: Option<String>,
    pub conversation_href: Option<String>,
    pub request_href: Option<String>,
    pub trace_href: Option<String>,
    pub client_kind: String,
    pub source: String,
    pub artifact_kind: Option<String>,
    pub artifact_icon: &'static str,
    pub artifact_label: &'static str,
    pub artifact_href: Option<String>,
    pub preview_href: Option<String>,
    pub artifact_title: String,
    pub bytes_display: String,
    pub redactions: i32,
    pub error_message: Option<String>,
    pub skill: Option<String>,
    pub skill_href: Option<String>,
}

pub(crate) fn kind_icon(kind: Option<&str>) -> &'static str {
    match kind {
        Some("file") => "file",
        Some("card") => "card",
        Some("ui") => "ui",
        Some("body") => "body",
        _ => "wrench",
    }
}

pub(crate) fn kind_label(kind: Option<&str>) -> &'static str {
    match kind {
        Some("file") => "File",
        Some("card") => "Card",
        Some("ui") => "UI",
        Some("body") => "Body",
        _ => "Tool call",
    }
}

pub(crate) fn format_bytes(bytes: i64) -> String {
    if bytes <= 0 {
        return "\u{2014}".to_owned();
    }
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let kib = bytes / 1024;
    if kib < 1024 {
        return format!("{kib} KiB");
    }
    format!("{} MiB", short_num(kib / 1024))
}

fn clip(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let head: String = text.chars().take(max - 1).collect();
    format!("{head}…")
}

fn state_tone(r: &ToolActivityRow) -> &'static str {
    if r.failed {
        "err"
    } else {
        match r.state.as_str() {
            "executed" => "ok",
            "intended" => "warn",
            _ => "muted",
        }
    }
}

fn decision_tone(decision: Option<&str>) -> &'static str {
    match decision {
        Some("allow") => "ok",
        Some("warn") => "warn",
        Some("deny") => "err",
        _ => "muted",
    }
}

// Why: the five places a call links out to — person, harness session,
// conversation, request and trace — resolved together so the row stays
// readable. The session page only answers to a session id, never a trace
// id; the trace page resolves an execution by its trace id or by its own id,
// so a call with no gateway trace still links by execution id.
struct EntityLinks {
    user: Option<String>,
    session: Option<String>,
    conversation: Option<String>,
    request: Option<String>,
    trace: Option<String>,
}

fn entity_links(r: &ToolActivityRow, context_key: Option<&str>) -> EntityLinks {
    EntityLinks {
        user: r
            .user_id
            .as_ref()
            .map(|u| format!("/admin/users/{}", urlencoding::encode(u.as_str()))),
        session: r
            .session_key
            .as_ref()
            .map(|s| format!("/admin/sessions/{}", urlencoding::encode(s))),
        conversation: context_key
            .map(|c| format!("/admin/analysis/conversations/{}", urlencoding::encode(c))),
        request: r
            .request_id
            .as_ref()
            .map(|id| format!("/admin/requests/{}", urlencoding::encode(id))),
        trace: r
            .execution_trace_id
            .as_ref()
            .or(r.mcp_execution_id.as_ref())
            .map(|id| format!("/admin/traces/{}", urlencoding::encode(id))),
    }
}

// Why: the artifact's own title when it has one, else what the call was
// about, else the tool — never an empty cell.
fn artifact_title(r: &ToolActivityRow, tool_name: &str, input: String) -> String {
    r.artifact_title
        .clone()
        .filter(|t| !t.is_empty() && t != tool_name)
        .unwrap_or_else(|| {
            if input.is_empty() {
                tool_name.to_owned()
            } else {
                input
            }
        })
}

fn state_label(r: &ToolActivityRow) -> String {
    if r.failed {
        "failed".to_owned()
    } else {
        r.state.clone()
    }
}

fn duration_display(ms: Option<i32>) -> String {
    ms.map_or_else(
        || "\u{2014}".to_owned(),
        |ms| format_duration_ms(i64::from(ms)),
    )
}

pub(crate) fn tool_row(r: &ToolActivityRow) -> ToolActivityRowView {
    let tool_name = r.tool_name.clone().unwrap_or_else(|| "unknown".to_owned());
    let input = r.input_summary.clone().unwrap_or_default();
    let is_path = r.artifact_kind.as_deref() == Some("file") || input.starts_with('/');
    let context_key = r
        .context_key
        .clone()
        .or_else(|| r.execution_context_key.clone());
    let previewable = matches!(r.artifact_kind.as_deref(), Some("card" | "ui" | "body"))
        && r.is_structured
        && !r.is_error;
    let artifact_href = r
        .artifact_key
        .as_ref()
        .filter(|_| r.artifact_kind.is_some())
        .map(|id| format!("/admin/artifacts/{}", urlencoding::encode(id)));
    let links = entity_links(r, context_key.as_deref());
    ToolActivityRowView {
        row_key: r.row_key.clone(),
        at: r.occurred_at.map(relative_time).unwrap_or_default(),
        at_full: r.occurred_at.map(local_time).unwrap_or_default(),
        tool_icon: if r.is_builtin { "terminal" } else { "plug" },
        is_builtin: r.is_builtin,
        server_name: r.server_name.clone().unwrap_or_default(),
        input_summary: clip(&input, INPUT_CLIP),
        input_full: input.clone(),
        is_path,
        state_tone: state_tone(r),
        state: state_label(r),
        status: r.execution_status.clone().unwrap_or_default(),
        duration_display: duration_display(r.execution_time_ms),
        decision_tone: decision_tone(r.decision.as_deref()),
        decision: r.decision.clone().unwrap_or_else(|| "\u{2014}".to_owned()),
        decision_href: r
            .decision_id
            .as_ref()
            .map(|id| format!("/admin/governance/decisions/{}", urlencoding::encode(id))),
        user_key: r
            .user_id
            .as_ref()
            .map(|u| u.as_str().to_owned())
            .unwrap_or_default(),
        user_label: r
            .display_name
            .clone()
            .or_else(|| r.user_id.as_ref().map(|u| u.as_str().to_owned()))
            .unwrap_or_else(|| "\u{2014}".to_owned()),
        user_href: links.user,
        session_href: links.session,
        conversation_href: links.conversation,
        request_href: links.request,
        trace_href: links.trace,
        client_kind: r.client_kind.clone().unwrap_or_default(),
        source: r.source.clone().unwrap_or_default(),
        artifact_icon: kind_icon(r.artifact_kind.as_deref()),
        artifact_label: kind_label(r.artifact_kind.as_deref()),
        preview_href: r
            .artifact_key
            .as_ref()
            .filter(|_| previewable)
            .map(|id| format!("/admin/artifacts/{}/preview", urlencoding::encode(id))),
        artifact_href,
        artifact_title: artifact_title(r, &tool_name, input),
        artifact_kind: r.artifact_kind.clone(),
        bytes_display: format_bytes(i64::from(r.payload_bytes.unwrap_or(0))),
        redactions: r.secret_redactions.unwrap_or(0),
        error_message: r.error_message.clone().filter(|m| !m.is_empty()),
        skill_href: r.skill.as_deref().map(analysis_skill_url),
        skill: r.skill.clone(),
        tool_name,
    }
}
