//! Assembling the conversations page context from the query, the scope and
//! the repository result: the filter ribbon, breakdown rows with their
//! export links, sort headers, selection and the help modal.

use crate::handlers::ssr::analysis::help::conversations_help;
use crate::handlers::ssr::analysis::reports::banner::ReportBannerView;
use crate::handlers::ssr::analysis::ribbon::{RibbonGroupView, RibbonView};
use crate::handlers::ssr::list_view::{PageWindow, ScopeFilterView};
use crate::repositories::analysis::conversations::{
    BreakdownBy, ConversationAnalysisResult, FlagFilter,
};
use crate::repositories::scope::ScopeRequest;
use systemprompt::identifiers::UserId;

use super::charts::series_charts;
use super::exports::{DATASET, export_view};
use super::filters::{filter_view, sort_headers};
use super::kpis::KpisView;
use super::page_context::ConversationsPageContext;
use super::query::{BASE_URL, ConversationAnalysisQuery};
use super::summary::{CATEGORIES, ConversationBucketView, OUTCOMES, category_label};
use super::view::ConversationRowView;
use crate::handlers::ssr::list_view::DEFAULT_PAGE_SIZE;


// Why: the breakdown dimension's query parameter — the key a bucket row's
// link and export narrow by; groups and projects narrow through the scope
// bar instead.
const fn bucket_param(by: BreakdownBy) -> Option<&'static str> {
    match by {
        BreakdownBy::Category => Some("category"),
        BreakdownBy::Outcome => Some("outcome"),
        BreakdownBy::Model => Some("model"),
        BreakdownBy::Client => Some("client"),
        BreakdownBy::Skill => Some("skill"),
        BreakdownBy::User => Some("user_id"),
        BreakdownBy::Group | BreakdownBy::Project => None,
    }
}

fn breakdown_rows(
    params: &ConversationAnalysisQuery,
    data: &ConversationAnalysisResult,
) -> Vec<ConversationBucketView> {
    let by = params.breakdown();
    data.breakdown
        .iter()
        .map(|row| {
            let value = if by == BreakdownBy::User {
                row.user_id.as_ref().map(|u| u.as_str().to_owned())
            } else {
                Some(row.label.clone())
            };
            let key = bucket_param(by).zip(value);
            let href = key
                .as_ref()
                .map(|(name, value)| params.narrowed(name, value));
            let export = key.as_ref().map(|(name, value)| {
                (
                    params.export_href(DATASET, name, value),
                    params.export_query(name, value),
                )
            });
            let label = (by == BreakdownBy::Category).then(|| category_label(&row.label));
            ConversationBucketView::new(
                row,
                href,
                export,
                label,
                data.totals.total_cost_microdollars,
            )
        })
        .collect()
}

// Why: a chip per active filter, each removable and each downloadable as
// exactly the rows it selects.
fn chips(params: &ConversationAnalysisQuery, mut ribbon: RibbonView) -> RibbonView {
    let active: [(&'static str, &'static str, Option<String>); 9] = [
        ("Model", "model", params.model()),
        ("Client", "client", params.client()),
        (
            "Record",
            "flag",
            params.flag().map(|f| f.as_str().to_owned()),
        ),
        ("Skill", "skill", params.skill()),
        (
            "Person",
            "user_id",
            params.user_id().map(|u| u.as_str().to_owned()),
        ),
        (
            "Intent",
            "category",
            params.category().map(|c| category_label(&c).to_owned()),
        ),
        ("Outcome", "outcome", params.outcome()),
        (
            "Judged",
            "judged",
            params.judged().map(|j| j.as_str().to_owned()),
        ),
        ("Search", "q", params.free_text()),
    ];
    for (group, name, value) in active {
        let Some(value) = value else { continue };
        let remove = params
            .link_prefix(&[name, "page"])
            .trim_end_matches(['?', '&'])
            .to_owned();
        let raw = params.pair_value(name).unwrap_or_default();
        let export = (name != "q").then(|| params.export_href(DATASET, name, &raw));
        ribbon = ribbon.chip(group, value, remove, export);
    }
    ribbon
}

const RIBBON_PARAMS: [&str; 10] = [
    "model", "client", "flag", "skill", "user_id", "category", "outcome", "judged", "q", "page",
];

// Why: the pills fed by the filtered set's own facets, each listing only the
// values present, with counts.
fn facet_groups(
    params: &ConversationAnalysisQuery,
    data: &ConversationAnalysisResult,
) -> Vec<RibbonGroupView> {
    let user_id = params.user_id();
    vec![
        RibbonGroupView::single(
            "model",
            "Model",
            "model",
            params.model().as_deref(),
            data.models
                .iter()
                .map(|m| (m.model.as_str(), m.model.clone(), m.conversations)),
        ),
        RibbonGroupView::single(
            "client",
            "Client",
            "plug",
            params.client().as_deref(),
            data.clients.iter().map(|c| {
                (
                    c.client_kind.as_str(),
                    c.client_kind.clone(),
                    c.conversations,
                )
            }),
        ),
        RibbonGroupView::single(
            "user_id",
            "Person",
            "user",
            user_id.as_ref().map(UserId::as_str),
            data.users.iter().map(|u| {
                (
                    u.user_id.as_str(),
                    u.display_name
                        .clone()
                        .unwrap_or_else(|| u.user_id.as_str().to_owned()),
                    u.conversations,
                )
            }),
        ),
        RibbonGroupView::single(
            "skill",
            "Skill",
            "skill",
            params.skill().as_deref(),
            data.skills
                .iter()
                .map(|s| (s.skill.as_str(), s.skill.clone(), s.conversations)),
        ),
    ]
}

// Why: the pills over closed vocabularies — the record flags and the
// judge's own words.
fn fixed_groups(params: &ConversationAnalysisQuery) -> Vec<RibbonGroupView> {
    vec![
        RibbonGroupView::fixed(
            "flag",
            "Record",
            "layers",
            params.flag().map(FlagFilter::as_str),
            &FlagFilter::ALL.map(|(v, l)| (v.as_str(), l)),
        ),
        RibbonGroupView::fixed(
            "category",
            "Intent",
            "sparkle",
            params.category().as_deref(),
            &CATEGORIES,
        ),
        RibbonGroupView::fixed(
            "outcome",
            "Outcome",
            "check",
            params.outcome().as_deref(),
            &OUTCOMES,
        ),
    ]
}

fn ribbon(params: &ConversationAnalysisQuery, data: &ConversationAnalysisResult) -> RibbonView {
    let clear = params
        .link_prefix(&RIBBON_PARAMS)
        .trim_end_matches(['?', '&'])
        .to_owned();
    let mut ribbon = RibbonView::new(BASE_URL, clear).preserve(&params.preserved(&RIBBON_PARAMS));
    for group in facet_groups(params, data)
        .into_iter()
        .chain(fixed_groups(params))
    {
        ribbon = ribbon.group(group);
    }
    let ribbon = ribbon.search(
        "q",
        params.free_text().as_deref(),
        "title, summary, tag, model, skill or person",
    );
    chips(params, ribbon)
}

// Why: what the handler read besides the repository result.
pub(super) struct ConversationsPageInputs {
    pub(super) scope_filter: ScopeFilterView,
    pub(super) report_banner: ReportBannerView,
    pub(super) can_judge: bool,
    pub(super) manual_judge: bool,
}

pub(super) fn build_context(
    params: &ConversationAnalysisQuery,
    request: &ScopeRequest,
    data: &ConversationAnalysisResult,
    inputs: ConversationsPageInputs,
) -> ConversationsPageContext {
    let current_url = params.current_url();
    let rows: Vec<ConversationRowView> = data
        .rows
        .iter()
        .map(|r| {
            ConversationRowView::from_fact(r)
                .with_viewer(inputs.can_judge && inputs.manual_judge, &current_url)
        })
        .collect();
    let shown = i64::try_from(rows.len()).unwrap_or(0);
    let window = PageWindow::new(
        params.page(),
        DEFAULT_PAGE_SIZE,
        data.totals.conversations,
        shown,
        "conversations",
    );
    let hourly = params.since_label() == "24h";
    ConversationsPageContext {
        page: "analysis-conversations",
        title: "Conversations",
        kpis: KpisView::new(&data.totals, &data.series),
        charts: series_charts(&data.series, hourly),
        range_links: params.range_links(),
        filter: filter_view(params, request),
        ribbon: ribbon(params, data),
        scope_filter: inputs.scope_filter,
        breakdown_tabs: params.breakdown_tabs(),
        breakdown_label: params.breakdown().label(),
        breakdown: breakdown_rows(params, data),
        has_rows: !rows.is_empty(),
        count_label: format!("{} conversations", data.totals.conversations),
        turns_toggle: params.turns_toggle(data.totals.without_turns),
        rows,
        pagination: params.pagination(window),
        sort_headers: sort_headers(params),
        export: export_view(params),
        help: conversations_help(),
        report_banner: inputs.report_banner,
        current_url,
        judge_all_url: super::judge_all::JUDGE_ALL_URL,
        can_judge: inputs.can_judge,
        manual_judge: inputs.can_judge && inputs.manual_judge,
        unjudged_in_view: data.totals.conversations - data.totals.judged,
    }
}
