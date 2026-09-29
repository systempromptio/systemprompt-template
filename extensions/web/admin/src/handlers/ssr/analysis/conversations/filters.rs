//! The filter state echoed back into the form and the sortable column
//! headers, both derived from the page query.

use super::page_context::SortHeaders;
use super::query::ConversationAnalysisQuery;
use super::summary::ConversationAnalysisFilterView;
use crate::repositories::analysis::conversations::{FactSort, FlagFilter, JudgedFilter};
use crate::repositories::scope::ScopeRequest;

pub(super) fn filter_view(
    params: &ConversationAnalysisQuery,
    request: &ScopeRequest,
) -> ConversationAnalysisFilterView {
    ConversationAnalysisFilterView {
        since: params.since_label(),
        q: params.free_text().unwrap_or_default(),
        category: params.category().unwrap_or_default(),
        outcome: params.outcome().unwrap_or_default(),
        skill: params.skill().unwrap_or_default(),
        judged: params
            .judged()
            .map(JudgedFilter::as_str)
            .unwrap_or_default()
            .to_owned(),
        model: params.model().unwrap_or_default(),
        client: params.client().unwrap_or_default(),
        flag: params
            .flag()
            .map(FlagFilter::as_str)
            .unwrap_or_default()
            .to_owned(),
        user_key: params
            .user_id()
            .map(|u| u.as_str().to_owned())
            .unwrap_or_default(),
        group: request.group.clone().unwrap_or_default(),
        project: request.project.clone().unwrap_or_default(),
        by: params.breakdown().as_str(),
        sort: params.sort().as_str(),
        dir: if params.descending() { "desc" } else { "asc" },
        query: params.query_string(),
    }
}

pub(super) fn sort_headers(params: &ConversationAnalysisQuery) -> SortHeaders {
    let h = |col, label, class, hint| params.sort_header(col, label, class, hint);
    SortHeaders {
        activity: h(
            FactSort::Activity,
            "Last",
            "sp-col-date",
            "Most recent request on the conversation",
        ),
        turns: h(
            FactSort::Turns,
            "Turns",
            "sp-table__cell--num",
            "Human prompts answered",
        ),
        tools: h(
            FactSort::Tools,
            "Tools",
            "sp-table__cell--num",
            "Tool calls the model asked for (executed/asked when they differ)",
        ),
        errors: h(
            FactSort::Errors,
            "Err",
            "sp-table__cell--num",
            "Failed requests plus denied tool calls",
        ),
        tokens: h(
            FactSort::Tokens,
            "Tokens",
            "sp-table__cell--num",
            "Input and output tokens; the bar is the cache-read share",
        ),
        cost: h(
            FactSort::Cost,
            "Cost",
            "sp-table__cell--num",
            "Priced spend on the conversation",
        ),
        active: h(
            FactSort::Active,
            "Active",
            "sp-table__cell--num",
            "Time the model spent answering: the summed latency of every turn",
        ),
        duration: h(
            FactSort::Duration,
            "Length",
            "sp-table__cell--num",
            "Wall clock from first to last activity, idle time included",
        ),
    }
}
