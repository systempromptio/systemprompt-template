//! `/admin/analysis/reports` — on-demand AI reports over the deterministic
//! record. The list is the history of every request with its status,
//! headline and assessment and the form that queues a new one; a report page
//! shows the model's headline, assessment, grounded themes with evidence
//! links into Conversations, Skills and Tools, its recommendations, and the
//! digest it was written from. A request writes the row and starts the one
//! model call on a background task (`services::analysis_report`); the page
//! shows a loading widget that polls `/status` until the verdict is there.

pub(crate) mod banner;
mod form;
mod view;

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Form, Path, State};
use axum::http::HeaderMap;
use axum::response::{Redirect, Response};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::ai::AiService;

use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::handlers::ssr::analysis::help::{HelpView, item};
use crate::handlers::ssr::analysis_urls::ANALYSIS_CONVERSATIONS_URL;
use crate::handlers::ssr::format::{format_cost, format_token_total, local_time};
use crate::handlers::ssr::list_view::SelectOptionView;
use crate::handlers::ssr::page::Page;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::analysis::inventory_index::InventoryIndex;
use crate::repositories::analysis::reports::{
    AnalysisReportRow, find_report, list_reports, render_digest_text,
};
use crate::services::analysis_report::generate_report;

use form::{ReportRequestForm, queue, regenerate_form, resolve};
use view::{
    RecommendationView, ReportPageContext, ReportsPageContext, ThemeView, assessment_icon,
    report_row, scope_label, severity_icon, status_tone, tiles,
};

pub(crate) const REPORTS_URL: &str = "/admin/analysis/reports";
const HISTORY_LIMIT: i64 = 100;

fn reports_help() -> HelpView {
    HelpView::new(
        "AI reports — what they are and how they are grounded",
        "A report is the model's reading of the deterministic record for one scope and window. It sees figures only — never a transcript — and every theme cites the ids it rests on, which become the links on the page.",
    )
    .section(
        "The report",
        vec![
            item("sparkle", "Headline · assessment", "One sentence and one verdict: healthy, watch or degraded."),
            item("layers", "Themes", "Findings grounded in the digest, most important first, each with a severity and evidence chips that open the skill, conversation, model, tool or person cited."),
            item("check", "Recommendations", "Concrete actions on this console, ordered by priority."),
            item("report", "Digest", "The exact figures the model was given — totals, top skills, models and people, best and worst judged conversations, cost outliers, denied tools, intent and client mix — kept with the report for audit."),
        ],
    )
    .section(
        "Generating",
        vec![
            item("calendar", "Scope and window", "Whole instance, one marketplace, one skill, or the filters of the page you came from (Report on this view). The window is anchored to now."),
            item("clock", "On demand only", "A request queues a row; the analysis_report job generates it when run — `systemprompt infra jobs run analysis_report`. Nothing is scheduled, so no report costs anything unless someone asked for it."),
            item("coins", "Cost", "One structured call to the configured judge model per report, audited under the job's identity and shown on the row."),
        ],
    )
}

fn require_console(shell: &Page) -> Result<(), AdminError> {
    if shell.user.is_console {
        Ok(())
    } else {
        Err(AdminError::Forbidden("Console access required".into()))
    }
}

pub(crate) async fn page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    require_console(&shell)?;
    let rows = list_reports(&pool, HISTORY_LIMIT).await?;
    let index = InventoryIndex::build(&crate::handlers::shared::get_services_path()?);
    let context = ReportsPageContext {
        page: "analysis-reports",
        title: "Reports",
        has_rows: !rows.is_empty(),
        rows: rows.iter().map(report_row).collect(),
        marketplace_options: index
            .marketplaces
            .iter()
            .map(|m| SelectOptionView {
                value: m.id.as_str().to_owned(),
                label: m.name.clone(),
                selected: false,
            })
            .collect(),
        help: reports_help(),
    };
    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        "analysis-reports",
        &context,
        &shell.user,
        &shell.marketplace,
    ))
}

fn digest_summary(row: &AnalysisReportRow) -> String {
    let totals = &row.inputs.digest.totals;
    format!(
        "Reading {} conversations by {} people — {} skills, {} models, the best and worst judged, cost outliers and denials",
        totals.conversations,
        totals.people,
        row.inputs.digest.skills.len(),
        row.inputs.digest.models.len()
    )
}

fn theme_views(
    findings: Option<&crate::repositories::analysis::reports::ReportFindings>,
) -> Vec<ThemeView> {
    findings
        .map(|f| {
            f.themes
                .iter()
                .map(|t| ThemeView {
                    kind: t.kind.clone(),
                    severity: t.severity.as_str(),
                    icon: severity_icon(t.severity.as_str()),
                    theme_title: t.title.clone(),
                    detail: t.detail.clone(),
                    evidence: t.evidence.clone(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn report_context(row: &AnalysisReportRow) -> ReportPageContext {
    let findings = row.findings.as_ref();
    let assessment = findings.map_or("", |f| f.assessment.as_str());
    ReportPageContext {
        page: "analysis-report",
        title: findings.map_or_else(|| scope_label(row), |f| f.headline.clone()),
        breadcrumbs: vec![
            BreadcrumbView::link("Analysis", ANALYSIS_CONVERSATIONS_URL),
            BreadcrumbView::link("Reports", REPORTS_URL),
            BreadcrumbView::current(scope_label(row)),
        ],
        report_id: row.id.clone(),
        digest_summary: digest_summary(row),
        status_url: format!("{REPORTS_URL}/{}/status", urlencoding::encode(&row.id)),
        scope_label: scope_label(row),
        scope_kind: row.scope_kind.clone(),
        window_display: format!(
            "{} → {}",
            row.window_start.format("%b %-d, %Y"),
            row.window_end.format("%b %-d, %Y")
        ),
        status: row.status.clone(),
        status_tone: status_tone(&row.status),
        pending: row.status == "pending",
        failed: row.status == "failed",
        headline: findings.map(|f| f.headline.clone()).unwrap_or_default(),
        assessment: assessment.to_owned(),
        assessment_label: findings.map_or("", |f| f.assessment.label()).to_owned(),
        assessment_tone: findings.map_or("muted", |f| f.assessment.tone()),
        assessment_icon: assessment_icon(assessment),
        generated_display: row.generated_at.map(local_time).unwrap_or_default(),
        model: row.model.clone().unwrap_or_default(),
        tokens_display: format_token_total(
            i64::from(row.input_tokens.unwrap_or(0)) + i64::from(row.output_tokens.unwrap_or(0)),
        ),
        cost_display: row.cost_microdollars.map_or_else(String::new, format_cost),
        requested_by: row.requested_by.clone(),
        error: row.last_error.clone().unwrap_or_default(),
        filter_query: row.inputs.filter_query.clone(),
        tiles: tiles(&row.inputs.digest.totals),
        themes: theme_views(findings),
        recommendations: findings
            .map(|f| {
                f.recommendations
                    .iter()
                    .map(|r| RecommendationView {
                        action: r.action.clone(),
                        rationale: r.rationale.clone(),
                        priority: r.priority.as_str(),
                        priority_tone: r.priority.tone(),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        digest_text: render_digest_text(&row.inputs.digest),
        regenerate_url: format!("{REPORTS_URL}/{}/regenerate", urlencoding::encode(&row.id)),
        back_url: REPORTS_URL,
        help: reports_help(),
    }
}

async fn load_report(pool: &PgPool, id: &str) -> Result<AnalysisReportRow, AdminError> {
    let id = id.trim();
    if id.is_empty() || id.len() > 64 {
        return Err(AdminError::BadRequest(
            "A report id is required.".to_owned(),
        ));
    }
    find_report(pool, id)
        .await?
        .ok_or_else(|| AdminError::NotFound("No report matches that id.".to_owned()))
}

pub(crate) async fn report_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminHtmlResult<Response> {
    require_console(&shell)?;
    let row = load_report(&pool, &id).await?;
    let context = report_context(&row);
    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        "analysis-report",
        &context,
        &shell.user,
        &shell.marketplace,
    ))
}

// Why: the row is written with its digest and a lease, generation starts on
// its own task, and the reader is sent to the report page, whose loading
// widget polls `/status` until the model has answered.
async fn start(
    pool: &Arc<PgPool>,
    ai: Option<Arc<AiService>>,
    shell: &Page,
    form: &ReportRequestForm,
) -> AdminResult<String> {
    let resolved = resolve(pool, &shell.user, form).await?;
    let queued = queue(pool, &shell.user, resolved).await?;
    let row = load_report(pool, &queued.id).await?;
    tokio::spawn(generate_report(
        Arc::clone(pool),
        ai,
        shell.user.user_id.clone(),
        row,
        queued.lease_token,
    ));
    tracing::info!(report_id = %queued.id, requested_by = %shell.user.user_id, "analysis report generation started from the console");
    Ok(queued.id)
}

fn report_redirect(id: &str) -> Redirect {
    Redirect::to(&format!("{REPORTS_URL}/{}", urlencoding::encode(id)))
}

// Why: `POST /admin/analysis/reports` — write one report now. The digest is
// computed at this moment, so the report answers the record as it stood.
pub(crate) async fn create(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Extension(ai): Extension<Option<Arc<AiService>>>,
    headers: HeaderMap,
    Form(form): Form<ReportRequestForm>,
) -> AdminHtmlResult<Redirect> {
    require_console(&shell)?;
    crate::handlers::shared::require_write_origin(&headers)?;
    let id = start(&pool, ai, &shell, &form).await?;
    Ok(report_redirect(&id))
}

pub(crate) async fn regenerate(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Extension(ai): Extension<Option<Arc<AiService>>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> AdminHtmlResult<Redirect> {
    require_console(&shell)?;
    crate::handlers::shared::require_write_origin(&headers)?;
    let row = load_report(&pool, &id).await?;
    let new_id = start(&pool, ai, &shell, &regenerate_form(&row)).await?;
    Ok(report_redirect(&new_id))
}

// Why: what the loading widget polls: the row's state and, when it failed,
// why — as JSON, so the page reloads itself once the verdict exists.
#[derive(Debug, Serialize)]
pub(crate) struct ReportStatus {
    status: String,
    error: Option<String>,
    elapsed_ms: i64,
}

pub(crate) async fn status(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> AdminResult<Json<ReportStatus>> {
    require_console(&shell)?;
    let row = load_report(&pool, &id).await?;
    Ok(Json(ReportStatus {
        status: row.status.clone(),
        error: row.last_error.clone(),
        elapsed_ms: (chrono::Utc::now() - row.created_at).num_milliseconds(),
    }))
}
