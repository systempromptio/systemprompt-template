//! The KPI strip and the sortable headers for the sessions list.
//!
//! Split from `view`, which keeps the filter ribbon, the scope form and the
//! pagination. Both halves build their links from the same
//! `preserved_query_string`, so a sort and a page turn carry identical state.

use crate::handlers::ssr::format::{format_cost, format_token_total};
use crate::handlers::ssr::types::SortHeaderView;
use crate::repositories::analytics::conversation_rows::{ConversationSort, ConversationTotals};

use super::context::{SessionsSortHeaders, StatsView};
use super::view::preserved_query_string;
use super::{BASE_URL, SessionListQuery};

pub(super) fn build_sort_headers(
    query: &SessionListQuery,
    active_col: ConversationSort,
    is_desc: bool,
) -> SessionsSortHeaders {
    let qs = preserved_query_string(query, &["sort", "dir", "page"]);
    let prefix = if qs.is_empty() {
        format!("{BASE_URL}?")
    } else {
        format!("{BASE_URL}?{qs}&")
    };
    let header = |col: ConversationSort, label: &'static str, class: &'static str, hint| {
        let active = col == active_col;
        let next_dir = if active && is_desc { "asc" } else { "desc" };
        SortHeaderView {
            label,
            class,
            hint,
            url: format!("{prefix}sort={}&dir={next_dir}", col.as_str()),
            active,
            aria_sort: if active {
                if is_desc { "descending" } else { "ascending" }
            } else {
                "none"
            },
            indicator: if active {
                if is_desc { "▼" } else { "▲" }
            } else {
                "↕"
            },
        }
    };
    SessionsSortHeaders {
        started: header(
            ConversationSort::Activity,
            "Started",
            "sp-col-date",
            "First turn of the conversation; sorts by most recent activity",
        ),
        turns: header(
            ConversationSort::Turns,
            "Turns",
            "sp-table__cell--num",
            "Human prompts answered in this conversation",
        ),
        tokens: header(
            ConversationSort::Tokens,
            "Tokens",
            "sp-table__cell--num",
            "Input plus output tokens across the conversation",
        ),
        cost: header(
            ConversationSort::Cost,
            "Cost",
            "sp-table__cell--num",
            "Billed cost across the conversation, side calls included",
        ),
    }
}

pub(super) fn stats_view(current: &ConversationTotals, previous: &ConversationTotals) -> StatsView {
    let (conversations_delta, conversations_delta_dir) = delta(
        current.conversations,
        previous.conversations,
        Rising::Neutral,
    );
    let (turns_delta, turns_delta_dir) = delta(current.turns, previous.turns, Rising::Neutral);
    let (tokens_delta, tokens_delta_dir) =
        delta(current.total_tokens, previous.total_tokens, Rising::Bad);
    let (cost_delta, cost_delta_dir) = delta(
        current.total_cost_microdollars,
        previous.total_cost_microdollars,
        Rising::Bad,
    );
    StatsView {
        conversations: current.conversations,
        error_conversations: current.error_conversations,
        turns: current.turns,
        side_calls: current.side_calls,
        side_call_cost_display: format_cost(current.side_call_cost_microdollars),
        tokens_display: format_token_total(current.total_tokens),
        cost_display: format_cost(current.total_cost_microdollars),
        conversations_delta,
        conversations_delta_dir,
        turns_delta,
        turns_delta_dir,
        tokens_delta,
        tokens_delta_dir,
        cost_delta,
        cost_delta_dir,
    }
}

// Why: more conversations is neither good nor bad, but more spend is. The
// design system carries both readings, and picking the wrong one paints a
// rising bill in the colour of a healthy trend.
#[derive(Clone, Copy)]
enum Rising {
    Neutral,
    Bad,
}

// Why: a previous window of zero has no percentage to report, so the KPI shows
// no delta at all rather than an infinite rise.
fn delta(current: i64, previous: i64, rising: Rising) -> (Option<String>, Option<&'static str>) {
    if previous <= 0 {
        return (None, None);
    }
    #[expect(
        clippy::cast_precision_loss,
        reason = "a display percentage; the magnitudes here are far below f64's exact range"
    )]
    let pct = ((current - previous) as f64 / previous as f64) * 100.0;
    let rounded = pct.round();
    if rounded == 0.0 {
        return (Some("0%".to_owned()), None);
    }
    let dir = match (rounded > 0.0, rising) {
        (true, Rising::Neutral) => "up",
        (true, Rising::Bad) => "up-bad",
        (false, Rising::Neutral) => "down",
        (false, Rising::Bad) => "down-good",
    };
    (Some(format!("{rounded:+.0}%")), Some(dir))
}
