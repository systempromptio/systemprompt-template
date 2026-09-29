//! `/admin/projects/{id}/report`: one project, one printable page.
//!
//! The report is the detail page's three tabs laid out end to end with the
//! agents and artifacts the tabs do not show, rendered for paper: no chrome,
//! tables that do not scroll, and a "Save as PDF" control that hands the page
//! to the browser's print engine. Every read is the same repository call the
//! console makes, so the PDF a manager keeps says what the screen said.

use std::collections::HashMap;

use chrono::Utc;
use sqlx::PgPool;
use systemprompt_web_shared::ProjectId;

use crate::repositories::people_usage::DEFAULT_WINDOW_DAYS;
use crate::repositories::projects::output::{
    ProjectAgentRow, ProjectArtifactRow, list_project_agents, list_project_artifacts,
};
use crate::repositories::scope::{Attribution, ScopeQuery, ScopeTarget};
use crate::types::UserContext;
use crate::types::projects::ProjectRow;

use super::super::format::short_num;
use super::super::people_view::{chips, format_usd, model_rows, or_default};
use super::super::types::{
    BreadcrumbView, ProjectAgentRowView, ProjectArtifactRowView, ProjectReportFactView,
    ProjectReportPageData,
};
use super::detail::{DetailData, ProjectUsageData, member_views};
use super::detail_view::{daily_charts, kpis, session_row, skill_rows, tool_rows};
use super::{BASE_URL, WINDOW_LABEL};

// Why: the artifact table is bounded like every other section; a project
// that produced more kinds than this has outgrown a one-page report.
const ARTIFACT_LIMIT: i64 = 25;

pub(super) struct ReportData {
    pub(super) agents: Vec<ProjectAgentRow>,
    pub(super) artifacts: Vec<ProjectArtifactRow>,
}

pub(super) async fn load(pool: &PgPool, project_id: &ProjectId) -> ReportData {
    let q = ScopeQuery::new(
        ScopeTarget::Project(project_id),
        Attribution::Exclusive,
        DEFAULT_WINDOW_DAYS,
    );
    let (agents, artifacts) = tokio::join!(
        list_project_agents(pool, &q),
        list_project_artifacts(pool, &q, ARTIFACT_LIMIT),
    );
    ReportData {
        agents: or_default("project agents", agents),
        artifacts: or_default("project artifacts", artifacts),
    }
}

#[derive(Clone, Copy)]
pub(super) struct ReportInputs<'a> {
    pub(super) project: &'a ProjectRow,
    pub(super) detail: &'a DetailData,
    pub(super) usage: &'a ProjectUsageData,
    pub(super) report: &'a ReportData,
    pub(super) print_on_load: bool,
}

pub(super) fn page_data(inputs: ReportInputs<'_>, user_ctx: &UserContext) -> ProjectReportPageData {
    let ReportInputs {
        project,
        detail,
        usage,
        report,
        print_on_load,
    } = inputs;
    let members = member_views(detail, user_ctx);
    let member_count = members.len() as i64;
    let names: HashMap<&str, &str> = members
        .iter()
        .map(|m| (m.user_id.as_str(), m.display_name.as_str()))
        .collect();
    let sessions = usage
        .sessions
        .iter()
        .map(|s| {
            let mut row = session_row(s);
            if let Some(name) = names.get(s.user_id.as_str()) {
                (*name).clone_into(&mut row.person);
            }
            row
        })
        .collect();
    let (daily, daily_cost) = daily_charts(&usage.daily);
    let artifact_total = report.artifacts.iter().map(|a| a.artifacts).sum();

    ProjectReportPageData {
        page: "project-report",
        title: format!("{} · report", project.name),
        breadcrumbs: vec![
            BreadcrumbView::link("Projects", BASE_URL),
            BreadcrumbView::link(&project.name, format!("{BASE_URL}/{}", project.id)),
            BreadcrumbView::current("Report"),
        ],
        detail_href: format!("{BASE_URL}/{}", project.id),
        window_label: WINDOW_LABEL.to_owned(),
        generated_at: Utc::now().format("%Y-%m-%d %H:%M UTC").to_string(),
        generated_by: user_ctx.email.to_string(),
        print_on_load,
        facts: facts(project, detail, member_count, report),
        kpis: kpis(detail, member_count),
        group_count: detail.groups.len() as i64,
        groups_represented: chips(&detail.groups),
        members,
        member_count,
        daily,
        daily_cost,
        models: model_rows(&usage.models),
        agents: agent_rows(&report.agents),
        skills: skill_rows(&detail.skills),
        tools: tool_rows(&usage.tools),
        artifacts: report.artifacts.iter().map(artifact_row).collect(),
        artifact_total,
        sessions,
        export: super::report_export_view(&project.id),
        project_id: project.id.clone(),
        project_name: project.name.clone(),
        description: project.description.clone(),
    }
}

fn facts(
    project: &ProjectRow,
    detail: &DetailData,
    member_count: i64,
    report: &ReportData,
) -> Vec<ProjectReportFactView> {
    let fact = |label: &'static str, value: String| ProjectReportFactView { label, value };
    let agents: Vec<&str> = report
        .agents
        .iter()
        .filter(|a| a.client_kind != "unknown" && a.client_kind != "internal")
        .map(|a| a.client_kind.as_str())
        .collect();
    vec![
        fact("Project id", project.id.to_string()),
        fact("Defined by", project.source.clone()),
        fact("Members", member_count.to_string()),
        fact("Groups represented", detail.groups.len().to_string()),
        fact(
            "Tokens",
            format!(
                "{} in / {} out / {} total",
                short_num(detail.usage.tokens_in),
                short_num(detail.usage.tokens_out),
                short_num(detail.usage.tokens)
            ),
        ),
        fact("Cost", format_usd(detail.usage.cost_microdollars)),
        fact(
            "Agents",
            if agents.is_empty() {
                "none in the window".to_owned()
            } else {
                agents.join(", ")
            },
        ),
        fact("Skills invoked", detail.skills.len().to_string()),
        fact(
            "Artifacts",
            report
                .artifacts
                .iter()
                .map(|a| a.artifacts)
                .sum::<i64>()
                .to_string(),
        ),
    ]
}

fn agent_rows(rows: &[ProjectAgentRow]) -> Vec<ProjectAgentRowView> {
    let total: i64 = rows.iter().map(|r| r.requests).sum();
    rows.iter()
        .map(|r| ProjectAgentRowView {
            label: agent_label(&r.client_kind).to_owned(),
            client_kind: r.client_kind.clone(),
            requests: r.requests,
            users: r.users,
            tokens_display: short_num(r.tokens),
            cost_display: format_usd(r.cost_microdollars),
            models: r.models,
            share_pct: super::pct(r.requests, total),
        })
        .collect()
}

// Why: the wire value is the check constraint's spelling; the report is read
// by people who know the product names.
fn agent_label(client_kind: &str) -> &'static str {
    match client_kind {
        "claude-code" => "Claude Code",
        "claude-desktop" => "Claude Desktop",
        "codex" => "Codex",
        "opencode" => "OpenCode",
        "hermes" => "Hermes",
        "pi" => "Pi",
        "internal" => "Internal job",
        "other" => "Other client",
        _ => "Unknown client",
    }
}

fn artifact_row(a: &ProjectArtifactRow) -> ProjectArtifactRowView {
    ProjectArtifactRowView {
        server_name: a.server_name.clone(),
        artifact_type: a.artifact_type.clone(),
        artifacts: a.artifacts,
        errors: a.errors,
        users: a.users,
        last_created_at: a.last_created_at.to_rfc3339(),
    }
}
