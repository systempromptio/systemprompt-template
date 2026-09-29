//! Versions: every marketplace by the content hash of what it serves.
//!
//! The landing page lists each marketplace with its current hash and the
//! window's figures across all its versions. A marketplace's page has four
//! views over one window — History (every version, what changed between
//! consecutive ones, how each performed), Compare (two versions side by side,
//! skill by skill), Evaluation (every conversation scored by fixed rules, by
//! version and skill: the rows the `analysis-plugin-eval` export returns) and
//! Distribution (what the publication pipeline delivered to devices for this
//! marketplace's skills). A version is its hash; no counter is shown anywhere.

mod access;
mod compare;
pub(crate) mod diff;
mod distribution;
mod evaluation;
mod history;
mod landing;
mod views;

use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::response::Response;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::MarketplaceId;

use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::handlers::ssr::analysis_urls::ANALYSIS_VERSIONS_URL;
use crate::handlers::ssr::page::Page;
use crate::handlers::ssr::render_typed_page;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::analysis::marketplace_versions::{
    VersionWindow, list_marketplace_completion_rollups, list_marketplace_rollups,
    list_version_completion,
};
use crate::routes::managed_state::ManagedState;

use access::readable_versions;
pub(crate) use access::{may_read_marketplace, require_versions_reader};


const WINDOWS: [u32; 4] = [7, 30, 90, 365];

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VersionsQuery {
    pub days: Option<u32>,
    pub tab: Option<String>,
    pub a: Option<String>,
    pub b: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Tab {
    History,
    Compare,
    Evaluation,
    Distribution,
}

impl Tab {
    // Why: the Evaluation tab's figures are its own export's rows, so its
    // button downloads them; every other view exports the version history.
    const fn dataset(self) -> &'static str {
        match self {
            Self::Evaluation => "analysis-plugin-eval",
            Self::History | Self::Compare | Self::Distribution => "analysis-versions",
        }
    }

    fn parse(value: Option<&str>) -> AdminResult<Self> {
        Ok(match value.unwrap_or("history") {
            "history" => Self::History,
            "compare" => Self::Compare,
            "evaluation" => Self::Evaluation,
            "distribution" => Self::Distribution,
            _ => return Err(AdminError::BadRequest("Unknown versions tab".to_owned())),
        })
    }
}

fn window(days: Option<u32>) -> AdminResult<(u32, VersionWindow)> {
    let days = days.unwrap_or(30);
    if !WINDOWS.contains(&days) {
        return Err(AdminError::BadRequest("Unsupported window".to_owned()));
    }
    let end = (Utc::now() + Duration::days(1))
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap_or_default()
        .and_utc();
    Ok((
        days,
        VersionWindow {
            start: end - Duration::days(i64::from(days)),
            end,
        },
    ))
}

fn clean_hash(value: Option<String>) -> AdminResult<Option<String>> {
    match value {
        None => Ok(None),
        Some(v) if v.is_empty() => Ok(None),
        Some(v) if v.len() <= 64 && v.chars().all(|c| c.is_ascii_hexdigit()) => Ok(Some(v)),
        Some(_) => Err(AdminError::BadRequest("Invalid version hash".to_owned())),
    }
}

#[derive(Serialize)]
struct LandingPage {
    page: &'static str,
    title: &'static str,
    days: u32,
    marketplaces: Vec<landing::MarketplaceCard>,
    current: usize,
    retired: usize,
    versions: i64,
    export: crate::export::ExportView,
}

pub(crate) async fn landing_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<VersionsQuery>,
) -> AdminHtmlResult<Response> {
    require_versions_reader(&shell.user)?;
    let (days, window) = window(query.days)?;
    let rows: Vec<_> = list_marketplace_rollups(&pool, window)
        .await?
        .into_iter()
        .filter(|r| may_read_marketplace(&shell.user, &r.marketplace_id))
        .collect();
    let scores = list_marketplace_completion_rollups(&pool).await?;
    let current = rows.iter().filter(|r| r.content_hash.is_some()).count();
    let data = LandingPage {
        page: "analysis-versions",
        title: "Versions",
        days,
        retired: rows.len() - current,
        current,
        versions: rows.iter().map(|r| r.versions).sum(),
        marketplaces: rows
            .into_iter()
            .map(|row| {
                let rollup = scores
                    .iter()
                    .find(|s| s.marketplace_id == row.marketplace_id.as_str());
                landing::card(row, rollup)
            })
            .collect(),
        export: crate::export::ExportView::single("analysis-marketplaces", &format!("days={days}")),
    };
    Ok(render_typed_page(
        &shell.engine,
        "analysis-marketplaces",
        &data,
        &shell.user,
        &shell.marketplace,
    ))
}

#[derive(Serialize)]
struct TabLink {
    label: &'static str,
    href: String,
    is_active: bool,
}

fn tab_links(base: &str, active: Tab) -> Vec<TabLink> {
    [
        (Tab::History, "History", "history"),
        (Tab::Compare, "Compare", "compare"),
        (Tab::Evaluation, "Evaluation", "evaluation"),
        (Tab::Distribution, "Distribution", "distribution"),
    ]
    .into_iter()
    .map(|(tab, label, slug)| TabLink {
        label,
        href: format!("{base}&tab={slug}"),
        is_active: tab == active,
    })
    .collect()
}

#[derive(Serialize)]
struct DetailPage {
    page: &'static str,
    title: String,
    breadcrumbs: Vec<BreadcrumbView>,
    marketplace_id: MarketplaceId,
    name: String,
    days: u32,
    tab: Tab,
    tabs: Vec<TabLink>,
    base_href: String,
    current: Option<history::VersionView>,
    version_count: usize,
    history: Vec<history::VersionView>,
    compare: Option<compare::CompareView>,
    evaluation: Option<evaluation::EvaluationView>,
    distribution: Option<distribution::DistributionPage>,
    can_manage: bool,
    export: crate::export::ExportView,
}

pub(crate) async fn detail_page(
    shell: Page,
    Extension(managed): Extension<Arc<ManagedState>>,
    State(pool): State<Arc<PgPool>>,
    Path(marketplace_id): Path<String>,
    Query(query): Query<VersionsQuery>,
) -> AdminHtmlResult<Response> {
    require_versions_reader(&shell.user)?;
    if marketplace_id.len() > 128 || marketplace_id.chars().any(char::is_control) {
        return Err(AdminError::BadRequest("Invalid marketplace id".to_owned()).into());
    }
    let marketplace_id = MarketplaceId::new(marketplace_id);
    let (days, window) = window(query.days)?;
    let tab = Tab::parse(query.tab.as_deref())?;
    let rows = readable_versions(&pool, window, &marketplace_id, &shell.user).await?;
    let scores = list_version_completion(&pool, Some(&marketplace_id)).await?;
    let history = history::views(&rows, &scores);
    let name = history
        .iter()
        .find_map(|v| v.name.clone())
        .unwrap_or_else(|| marketplace_id.as_str().to_owned());
    let views = views::tab_views(
        &pool,
        &managed,
        views::TabInput {
            tab,
            window,
            marketplace_id: &marketplace_id,
            rows: &rows,
            scores: &scores,
            history: &history,
        },
        query,
    )
    .await?;
    let encoded = urlencoding::encode(marketplace_id.as_str());
    let base_href = format!("/admin/analysis/versions/{encoded}?days={days}");
    let data = DetailPage {
        export: crate::export::ExportView::single(
            tab.dataset(),
            &format!("days={days}&marketplace={encoded}"),
        ),
        page: "analysis-marketplace-versions",
        title: format!("{name} — versions"),
        breadcrumbs: vec![
            BreadcrumbView::link("Versions", ANALYSIS_VERSIONS_URL),
            BreadcrumbView::current(name.clone()),
        ],
        tabs: tab_links(&base_href, tab),
        base_href,
        marketplace_id,
        name,
        days,
        tab,
        current: history.iter().find(|v| v.is_current).cloned(),
        version_count: history.len(),
        history,
        compare: views.compare,
        evaluation: views.evaluation,
        distribution: views.distribution,
        can_manage: shell.user.is_admin,
    };
    Ok(render_typed_page(
        &shell.engine,
        "analysis-marketplace-versions",
        &data,
        &shell.user,
        &shell.marketplace,
    ))
}
