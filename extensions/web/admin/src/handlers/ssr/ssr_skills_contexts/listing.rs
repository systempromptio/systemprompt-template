//! Shapes the rows of the conversations listing for the active view.

use std::collections::HashMap;

use crate::repositories::scope::ScopeRequest;

use super::context::{ConversationItemView, FilterView, UserSummaryView};
use super::load::ContextsPageData;
use super::{ContextsListQuery, ContextsPageInputs, PAGE_SIZE, view};

pub(super) struct ContextsListing {
    pub(super) conversations: Vec<ConversationItemView>,
    pub(super) user_summaries: Vec<UserSummaryView>,
    pub(super) count: i64,
    pub(super) shown: i64,
    pub(super) noun: &'static str,
}

// Why: the users view groups conversations per person and the all view
// lists them flat; only the active view's rows are shaped.
pub(super) fn listing(
    inputs: &ContextsPageInputs,
    data: &ContextsPageData,
    params: &ContextsListQuery,
) -> ContextsListing {
    let by_user = if inputs.view_is_users {
        view::group_by_user(&data.conversations)
    } else {
        HashMap::new()
    };
    let conversations = if inputs.view_is_users {
        Vec::new()
    } else {
        data.conversations
            .iter()
            .map(view::conversation_item)
            .collect()
    };
    let user_summaries = data
        .user_summaries
        .iter()
        .map(|s| view::user_summary(s, &by_user, params))
        .collect();
    let (count, shown, noun) = if inputs.view_is_users {
        (data.totals.users, data.user_summaries.len(), "users")
    } else {
        (
            data.total_conversations,
            data.conversations.len(),
            "conversations",
        )
    };
    ContextsListing {
        conversations,
        user_summaries,
        count,
        shown: i64::try_from(shown).unwrap_or(PAGE_SIZE),
        noun,
    }
}

pub(super) fn filter_view(inputs: &ContextsPageInputs, request: &ScopeRequest) -> FilterView {
    FilterView {
        q: inputs.q.clone().unwrap_or_default(),
        since: inputs.since_label.clone().unwrap_or_default(),
        view: inputs.view.clone(),
        group: request.group.clone().unwrap_or_default(),
        project: request.project.clone().unwrap_or_default(),
        side: if inputs.show_side { "1" } else { "" }.to_owned(),
    }
}
