//! `/admin/artifacts/{artifact_id}` — one artifact: what it is, where it was
//! seen, what the scanners found, and every row of the same call.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::Response;
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{ArtifactId, SessionId, UserId};

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::analysis::help::HelpView;
use crate::handlers::ssr::entity_urls::{request_detail_url, session_detail_url};
use crate::handlers::ssr::format::{client_label, format_duration_ms, local_time, short_id};
use crate::handlers::ssr::page::Page;
use crate::handlers::ssr::ssr_helpers::render_typed_page;
use crate::handlers::ssr::ssr_tools::help::artifacts_help;
use crate::handlers::ssr::ssr_tools::rows::{format_bytes, kind_icon, kind_label};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::analytics::artifacts::{
    ArtifactDetail, ArtifactFindingRow, find_artifact,
};

fn source_label(source: &str) -> String {
    match source {
        "in_process" => "In-process".to_owned(),
        "proxy" => "Proxy tap".to_owned(),
        "gateway" => "Gateway history".to_owned(),
        "hook_claude_code" => "Claude Code hook".to_owned(),
        "hook_opencode" => "OpenCode hook".to_owned(),
        other => other.to_owned(),
    }
}

#[derive(Debug, Serialize)]
struct ArtifactDetailContext {
    page: &'static str,
    title: String,
    breadcrumbs: Vec<BreadcrumbView>,
    artifact: ArtifactView,
    provenance: ProvenanceView,
    findings: Vec<FindingView>,
    finding_count: usize,
    has_findings: bool,
    body_pretty: Option<String>,
    body_missing: bool,
    preview_url: Option<String>,
    back_url: &'static str,
    help: HelpView,
}

#[derive(Debug, Serialize)]
struct ArtifactView {
    artifact_id: ArtifactId,
    artifact_id_short: String,
    artifact_title: String,
    tool_name: String,
    server_name: String,
    artifact_type: String,
    kind_label: &'static str,
    kind_icon: &'static str,
    input_summary: Option<String>,
    is_builtin: bool,
    is_structured: bool,
    has_ui_resource: bool,
    is_error: bool,
    bytes_display: String,
    payload_sha256: Option<String>,
    secret_redactions: i32,
    created_at: String,
    skill: Option<String>,
    skill_url: Option<String>,
}

// Why: the strip that answers "how do we know this row is this call": the
// vantage point, the join quality, and a link to each of the other rows of
// the same call — intent, execution, decision, session.
#[derive(Debug, Serialize)]
struct ProvenanceView {
    source: String,
    source_label: String,
    last_seen_source: Option<String>,
    last_seen_label: Option<String>,
    correlation: String,
    correlation_tone: &'static str,
    state_label: String,
    state_tone: &'static str,
    client_label: Option<String>,
    ai_tool_call_id: Option<String>,
    mcp_execution_id: String,
    execution_status: Option<String>,
    execution_time_display: Option<String>,
    error_message: Option<String>,
    intended_at: Option<String>,
    request_id: Option<String>,
    request_url: Option<String>,
    session_id: Option<SessionId>,
    session_url: Option<String>,
    trace_id: Option<String>,
    user_id: Option<UserId>,
    user_label: String,
    governance_decision_id: Option<String>,
    governance_decision: Option<String>,
    governance_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct FindingView {
    phase: String,
    severity: String,
    severity_tone: &'static str,
    category: String,
    scanner: String,
    path: Option<String>,
    excerpt: Option<String>,
    redacted: bool,
    created_at: String,
}

pub(crate) async fn artifact_detail_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(artifact_id): Path<ArtifactId>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let detail = find_artifact(&pool, &artifact_id)
        .await
        .map_err(AdminError::from)?
        .ok_or_else(|| AdminError::NotFound(format!("Artifact {artifact_id} not found")))?;

    let ctx = build_context(&detail);
    Ok(render_typed_page(
        &shell.engine,
        "artifact-detail",
        &ctx,
        &shell.user,
        &shell.marketplace,
    ))
}

fn build_context(detail: &ArtifactDetail) -> ArtifactDetailContext {
    let row = detail;
    let body_pretty = detail
        .body
        .as_ref()
        .and_then(|body| serde_json::to_string_pretty(body).ok());
    let preview_url = (row.is_structured && detail.body.is_some())
        .then(|| format!("/admin/artifacts/{}/preview", row.artifact_id));

    ArtifactDetailContext {
        page: "artifact-detail",
        title: format!("Artifact {}", short_id(row.artifact_id.as_str())),
        breadcrumbs: vec![
            BreadcrumbView::link("AI activity", "/admin/analytics"),
            BreadcrumbView::link("Tools & artifacts", "/admin/tools"),
            BreadcrumbView::link("Artifacts", "/admin/artifacts"),
            BreadcrumbView::current(short_id(row.artifact_id.as_str())),
        ],
        artifact: artifact_view(detail),
        provenance: provenance_view(detail),
        findings: detail.findings.iter().map(finding_view).collect(),
        finding_count: detail.findings.len(),
        has_findings: !detail.findings.is_empty(),
        body_missing: detail.body.is_none(),
        body_pretty,
        preview_url,
        back_url: "/admin/artifacts",
        help: artifacts_help(),
    }
}

fn artifact_view(detail: &ArtifactDetail) -> ArtifactView {
    let row = detail;
    let tool_name = row
        .tool_name
        .clone()
        .unwrap_or_else(|| "unknown".to_owned());
    ArtifactView {
        artifact_id: row.artifact_id.clone(),
        artifact_id_short: short_id(row.artifact_id.as_str()),
        artifact_title: row
            .artifact_title
            .clone()
            .or_else(|| row.input_summary.clone())
            .unwrap_or_else(|| tool_name.clone()),
        tool_name,
        server_name: row.server_name.clone(),
        artifact_type: row.artifact_type.clone(),
        kind_label: kind_label(row.artifact_kind.as_deref()),
        kind_icon: kind_icon(row.artifact_kind.as_deref()),
        input_summary: row.input_summary.clone(),
        is_builtin: row.is_builtin,
        is_structured: row.is_structured,
        has_ui_resource: row.has_ui_resource,
        is_error: row.is_error,
        bytes_display: format_bytes(i64::from(row.payload_bytes.unwrap_or(0))),
        payload_sha256: detail.payload_sha256.clone(),
        secret_redactions: row.secret_redactions,
        created_at: local_time(row.created_at),
        skill: row.skill.clone(),
        skill_url: row
            .skill
            .as_ref()
            .map(|s| format!("/admin/artifacts?skill={}", urlencoding::encode(s))),
    }
}

fn provenance_view(detail: &ArtifactDetail) -> ProvenanceView {
    let row = detail;
    let correlation = row
        .correlation
        .clone()
        .unwrap_or_else(|| "exact".to_owned());
    let (state_label, state_tone) = if row.is_error {
        ("failed".to_owned(), "err")
    } else if row.request_id.is_some() {
        ("executed".to_owned(), "ok")
    } else {
        ("unattested".to_owned(), "warn")
    };
    ProvenanceView {
        source: row.source.clone(),
        source_label: source_label(&row.source),
        last_seen_source: detail.last_seen_source.clone(),
        last_seen_label: detail.last_seen_source.as_deref().map(source_label),
        correlation_tone: if correlation == "inferred" {
            "warn"
        } else {
            "ok"
        },
        correlation,
        state_label,
        state_tone,
        client_label: row.client_kind.as_deref().map(client_label),
        ai_tool_call_id: row.ai_tool_call_id.clone(),
        mcp_execution_id: row.mcp_execution_id.to_string(),
        execution_status: detail.execution_status.clone(),
        execution_time_display: detail
            .execution_time_ms
            .map(|ms| format_duration_ms(i64::from(ms))),
        error_message: detail.error_message.clone(),
        intended_at: detail.intended_at.map(local_time),
        request_id: row.request_id.clone(),
        request_url: row
            .request_id
            .as_deref()
            .map(|id| request_detail_url(&systemprompt::identifiers::AiRequestId::new(id))),
        session_id: row.session_id.clone(),
        session_url: row.session_id.as_ref().map(session_detail_url),
        trace_id: detail.trace_id.clone(),
        user_id: row.user_id.clone(),
        user_label: row
            .user_label
            .clone()
            .or_else(|| row.user_id.as_ref().map(|u| short_id(u.as_str())))
            .unwrap_or_else(|| "\u{2014}".to_owned()),
        governance_decision_id: detail.governance_decision_id.clone(),
        governance_decision: detail.governance_decision.clone(),
        governance_url: detail
            .governance_decision_id
            .as_ref()
            .map(|id| format!("/admin/governance/decisions/{id}")),
    }
}

fn finding_view(f: &ArtifactFindingRow) -> FindingView {
    FindingView {
        phase: f.phase.clone(),
        severity_tone: match f.severity.as_str() {
            "high" => "err",
            "medium" => "warn",
            _ => "info",
        },
        severity: f.severity.clone(),
        category: f.category.clone(),
        scanner: f.scanner.clone(),
        path: f.path.clone(),
        excerpt: f.excerpt.clone(),
        redacted: f.redacted,
        created_at: local_time(f.created_at),
    }
}
