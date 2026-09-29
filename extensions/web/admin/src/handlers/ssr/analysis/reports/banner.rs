//! The slim strip under the Conversations and Skills headers: the latest
//! generated report for the scope — assessment, headline, age, link — or
//! the invitation to generate one, plus the "Report on this view" action
//! that queues a report over exactly the page's current filters.

use serde::Serialize;
use sqlx::PgPool;

use crate::handlers::ssr::format::relative_time;
use crate::repositories::analysis::reports::find_latest_generated_report;

use super::REPORTS_URL;

#[derive(Debug, Serialize)]
pub(crate) struct ReportBannerView {
    pub exists: bool,
    pub pending: bool,
    pub href: String,
    pub headline: String,
    pub assessment: &'static str,
    pub assessment_label: &'static str,
    pub assessment_icon: &'static str,
    pub generated_ago: String,
    pub generate_action: &'static str,
    pub reports_href: &'static str,
    pub scope_kind: String,
    pub scope_id: String,
    pub query: String,
    pub label: String,
}

impl ReportBannerView {
    fn empty(scope_kind: &str, scope_id: Option<&str>, page_query: &str, label: &str) -> Self {
        Self {
            exists: false,
            pending: false,
            href: REPORTS_URL.to_owned(),
            headline: String::new(),
            assessment: "muted",
            assessment_label: "",
            assessment_icon: "sparkle",
            generated_ago: String::new(),
            generate_action: REPORTS_URL,
            reports_href: REPORTS_URL,
            scope_kind: scope_kind.to_owned(),
            scope_id: scope_id.unwrap_or_default().to_owned(),
            query: page_query.trim_start_matches('?').to_owned(),
            label: label.to_owned(),
        }
    }
}

// Why: a failed read degrades to the empty banner — the page's own data is
// what the reader came for, and the report strip must never take it down.
pub(crate) async fn report_banner(
    pool: &PgPool,
    scope_kind: &str,
    scope_id: Option<&str>,
    page_query: &str,
    label: &str,
) -> ReportBannerView {
    let mut view = ReportBannerView::empty(scope_kind, scope_id, page_query, label);
    let latest = match find_latest_generated_report(pool, scope_kind, scope_id).await {
        Ok(latest) => latest,
        Err(error) => {
            tracing::warn!(%error, "latest analysis report unavailable for banner");
            return view;
        },
    };
    let Some(row) = latest else {
        return view;
    };
    view.href = format!("{REPORTS_URL}/{}", urlencoding::encode(&row.id));
    view.pending = row.status == "pending";
    if let Some(findings) = row.findings.as_ref() {
        view.exists = true;
        view.headline.clone_from(&findings.headline);
        view.assessment = findings.assessment.tone();
        view.assessment_label = findings.assessment.label();
        view.assessment_icon = super::view::assessment_icon(findings.assessment.as_str());
        view.generated_ago = row.generated_at.map(relative_time).unwrap_or_default();
    }
    view
}
