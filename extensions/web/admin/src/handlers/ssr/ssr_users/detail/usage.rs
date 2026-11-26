//! The Usage tab: gateway totals, the latest conversation, the ten most
//! the latest one, the model split and the commits the bridge recorded.
//!
//! The conversation listing itself lives on its own tab; this one keeps the
//! numbers and points at it.

use systemprompt::identifiers::UserId;

#[cfg(feature = "agents")]
use crate::repositories::analytics::conversation_rows::ConversationRow;

use super::context::{CommitRowView, UsageTabView, UserModelRowView};
#[cfg(feature = "agents")]
use super::context::{LatestConversationView, UsageConversationsView};
#[cfg(feature = "agents")]
use super::load::ConversationUsage;
use super::load::UserUsageData;
use super::view::stamp;
#[cfg(feature = "agents")]
use crate::handlers::ssr::entity_urls::context_detail_url;
#[cfg(feature = "agents")]
use crate::handlers::ssr::format::relative_time;
use crate::handlers::ssr::format::{format_cost, format_token_total};
use crate::repositories::users::enrolment::UserCommitRow;

pub(super) fn usage_tab(data: UserUsageData, user_id: &UserId) -> UsageTabView {
    let encoded = urlencoding::encode(user_id.as_str()).into_owned();
    UsageTabView {
        window_label: "Gateway traffic through /v1/messages, all time.".to_owned(),
        requests: data.summary.requests,
        #[cfg(feature = "agents")]
        conversations: conversations_view(&data.conversations, &encoded),
        failed: data.summary.failed,
        tokens_display: format_token_total(data.summary.tokens),
        cost_display: format_cost(data.summary.cost_microdollars),
        first_request_at: stamp(data.summary.first_request_at),
        last_request_at: stamp(data.summary.last_request_at),
        has_models: !data.models.is_empty(),
        models: data
            .models
            .into_iter()
            .map(|share| UserModelRowView {
                model: share.model,
                requests: share.requests,
                tokens_display: format_token_total(share.tokens),
                cost_display: format_cost(share.cost_microdollars),
            })
            .collect(),
        log_url: format!("/admin/requests?user_id={encoded}"),
        has_commits: !data.commits.is_empty(),
        commits: data.commits.into_iter().map(commit_row).collect(),
    }
}

#[cfg(feature = "agents")]
fn conversations_view(c: &ConversationUsage, encoded: &str) -> UsageConversationsView {
    UsageConversationsView {
        conversations: c.totals.conversations,
        turns: c.totals.turns,
        tool_calls: c.totals.tool_calls,
        side_calls: c.totals.side_calls,
        side_call_cost_display: format_cost(c.totals.side_call_cost_microdollars),
        latest: c.latest.as_ref().map(latest_conversation),
        conversations_url: format!("/admin/users/{encoded}?tab=conversations"),
    }
}

#[cfg(feature = "agents")]
fn latest_conversation(c: &ConversationRow) -> LatestConversationView {
    LatestConversationView {
        conversation_title: c.title.clone(),
        url: context_detail_url(&c.context_id),
        context_id: c.context_id.clone(),
        last_relative: c.last_at.map_or_else(|| "—".to_owned(), relative_time),
        last_at: c.last_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
        model: c.model.clone().unwrap_or_else(|| "—".to_owned()),
        turns: c.turn_count,
        tool_calls: c.tool_call_count,
        cost_display: format_cost(c.total_cost_microdollars),
    }
}


fn commit_row(row: UserCommitRow) -> CommitRowView {
    CommitRowView {
        short_hash: row.commit_hash.chars().take(8).collect(),
        message: row.message.lines().next().unwrap_or_default().to_owned(),
        branch: row.branch.unwrap_or_default(),
        files_changed: row.files_changed,
        insertions: row.insertions,
        deletions: row.deletions,
        committed_at: stamp(Some(row.committed_at)),
    }
}
