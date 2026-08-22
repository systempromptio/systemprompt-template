//! `/admin/evals` — traffic distribution and evaluation results.
//!
//! The page is five tabs, selected by `?tab=` and rendered server-side, split
//! by the kind of eval rather than by the table the rows came from. `overview`
//! is the window's health, `traffic` is what actually went through the gateway
//! (models, users, prompt shapes) straight from `ai_requests`, and the last
//! three are one per [`EvalRunKind`](crate::repositories::evals::EvalRunKind):
//! `judge` scores live traffic, `head-to-head` compares two models, and
//! `golden-set` holds the cases replay exercises. Each of those three owns the
//! form that launches its own run, so the button and the table it fills sit
//! together.
//!
//! Runs are launched from here by POST and execute inline, so the redirect
//! back to the page already reflects the finished run. That is deliberate for
//! sample sizes in the tens; the `sample_size` ceiling in
//! [`crate::services::evals::MAX_SAMPLE_SIZE`] is what keeps it honest.

use std::sync::Arc;

use axum::extract::{Extension, Path, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::types as charts;
use crate::repositories::evals::results::ResultFilter;
use crate::repositories::evals::{EvalRunKind, results, runs};
use crate::services::evals::MAX_SAMPLE_SIZE;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

mod actions;
mod context;
mod context_runs;
mod data;
mod format;
mod urls;
mod view;
mod view_runs;

use context::{EvalsPageContext, EvalsTab, NoticeView, RunDetailContext};

use actions::require_admin;
pub(crate) use actions::{eval_promote_case_action, eval_run_action};

const BASE_URL: &str = "/admin/evals";
const DEFAULT_SAMPLE_SIZE: i64 = 20;
const RUN_DETAIL_RESULT_LIMIT: i64 = 200;

#[derive(Debug, Deserialize)]
pub(crate) struct EvalsQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub tab: Option<String>,
    pub verdict: Option<String>,
    pub model: Option<String>,
    pub notice: Option<String>,
    pub notice_error: Option<String>,
}

impl EvalsQuery {
    // Why: An empty select submits `""`, which means "any" — not a model named
    // empty string. Blanks are dropped here so the repository never sees one.
    fn result_filter(&self) -> ResultFilter {
        ResultFilter {
            verdict: non_blank(self.verdict.as_deref()),
            model: non_blank(self.model.as_deref()),
        }
    }
}

fn non_blank(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

pub(crate) async fn evals_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<EvalsQuery>,
) -> AdminHtmlResult<Response> {
    require_admin(&user_ctx)?;

    let tab = EvalsTab::from_query(query.tab.as_deref());
    let filter = query.result_filter();
    let (range, auto_widened) = data::resolve_range(&pool, &query).await;
    let fetched = data::fetch_evals_data(&pool, range, tab, &filter).await;

    let traffic = view::traffic_stats(&fetched.stats, &fetched.models, &fetched.users);
    let total = fetched.stats.total;

    // Why: The Golden set tab lists only the runs that exercise it; every other run
    // kind belongs to the tab that launched it.
    let run_views = match tab {
        EvalsTab::GoldenSet => view_runs::run_rows_of_kind(&fetched.runs, EvalRunKind::Replay),
        _ => view_runs::run_rows(&fetched.runs),
    };
    let model_options = view::model_options(&fetched.models);

    let ctx = EvalsPageContext {
        page: "evals",
        title: "Evals",
        tab: tab.as_str(),
        is_overview: tab == EvalsTab::Overview,
        is_traffic: tab == EvalsTab::Traffic,
        is_judge: tab == EvalsTab::Judge,
        is_head_to_head: tab == EvalsTab::HeadToHead,
        is_golden_set: tab == EvalsTab::GoldenSet,
        show_traffic_kpis: matches!(tab, EvalsTab::Overview | EvalsTab::Traffic),
        show_quality_kpis: matches!(tab, EvalsTab::Judge | EvalsTab::HeadToHead),
        tabs: urls::tab_links(tab, &range, &query),
        time_range: urls::time_range_context(&query, &range, auto_widened, tab),
        traffic,
        scores: view::score_summary(&fetched.scores, total),
        histogram: charts::histogram_view(&fetched.hist, &fetched.stats),
        cost_chart: charts::cost_chart(&fetched.series, &range),
        models: view::model_rows(&fetched.models, &fetched.model_scores, total),
        users: view::user_rows(&fetched.users, total),
        topics: view::topic_rows(&fetched.topics, total),
        win_rates: view::win_rate_rows(&fetched.win_rates),
        pairs: view::pair_rows(&fetched.pairs),
        runs: run_views,
        results: view_runs::result_rows(&fetched.results),
        cases: view_runs::case_rows(&fetched.cases),
        filter: view::result_filter_view(&filter, &model_options),
        model_options,
        judge_model: default_judge_label(&fetched.models),
        default_sample_size: DEFAULT_SAMPLE_SIZE,
        max_sample_size: MAX_SAMPLE_SIZE,
        base_url: BASE_URL,
        notice: notice_from_query(&query),
    };

    Ok(super::render_typed_page(
        &engine, "evals", &ctx, &user_ctx, &mkt_ctx,
    ))
}

pub(crate) async fn eval_run_detail_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(run_id): Path<String>,
) -> AdminHtmlResult<Response> {
    require_admin(&user_ctx)?;

    let Some(run) = runs::find_run(&pool, &run_id)
        .await
        .map_err(AdminError::from)?
    else {
        return Err(AdminError::NotFound("No eval run with that id.".to_owned()).into());
    };

    let rows = results::list_results_for_run(&pool, &run_id, RUN_DETAIL_RESULT_LIMIT)
        .await
        .map_err(AdminError::from)?;
    let result_views = view_runs::result_rows(&rows);

    let ctx = RunDetailContext {
        page: "eval-run-detail",
        title: format!("Eval run · {}", run_id.chars().take(14).collect::<String>()),
        run: view_runs::run_row(&run),
        results: result_views,
        back_url: BASE_URL,
    };

    Ok(super::render_typed_page(
        &engine,
        "eval-run-detail",
        &ctx,
        &user_ctx,
        &mkt_ctx,
    ))
}


fn notice_from_query(query: &EvalsQuery) -> Option<NoticeView> {
    let message = query.notice.clone().filter(|n| !n.is_empty())?;
    Some(NoticeView {
        is_error: query.notice_error.as_deref() == Some("1"),
        message,
    })
}

fn default_judge_label(
    models: &[crate::repositories::evals::distribution::ModelDistributionRow],
) -> String {
    models
        .iter()
        .min_by_key(|m| m.request_count)
        .map_or_else(|| "none available".to_owned(), |m| m.model.clone())
}
