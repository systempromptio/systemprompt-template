//! View-model builders for the contexts list page: map repository rows
//! (`contexts_list::*`) into the typed template-context structs in `context`.

use std::collections::HashMap;

use crate::repositories::analytics::contexts_list;

use super::context::{ContextItemView, UserSummaryView};

pub(super) fn microdollars_to_usd(micro: i64) -> f64 {
    (micro as f64) / 1_000_000.0
}

pub(super) fn group_contexts_by_user(
    contexts: &[contexts_list::ContextListItem],
) -> HashMap<String, Vec<ContextItemView>> {
    let mut out: HashMap<String, Vec<ContextItemView>> = HashMap::new();
    for c in contexts {
        let key = c
            .user_id
            .as_ref()
            .map_or_else(|| "unknown".to_owned(), |u| u.as_str().to_owned());
        out.entry(key).or_default().push(ContextItemView {
            context_id: c.context_id.clone(),
            name: c.name.clone(),
            is_cli_session: c.kind.as_deref() == Some("cli_session"),
            user_id: None,
            display_name: None,
            session_id: None,
            model: c.model.clone(),
            request_count: c.request_count,
            message_count: c.message_count,
            error_count: c.error_count,
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: c.total_input_tokens + c.total_output_tokens,
            cost_usd: microdollars_to_usd(c.total_cost_microdollars),
            first_request_at: None,
            last_request_at: None,
            last_activity: c.last_activity_at.map(|t| t.to_rfc3339()),
        });
    }
    out
}

pub(super) fn build_contexts_json(
    contexts: &[contexts_list::ContextListItem],
) -> Vec<ContextItemView> {
    contexts
        .iter()
        .map(|c| ContextItemView {
            context_id: c.context_id.clone(),
            name: c.name.clone(),
            is_cli_session: c.kind.as_deref() == Some("cli_session"),
            user_id: c.user_id.clone(),
            display_name: c.display_name.clone(),
            session_id: c.session_id.clone(),
            model: c.model.clone(),
            request_count: c.request_count,
            message_count: c.message_count,
            error_count: c.error_count,
            input_tokens: c.total_input_tokens,
            output_tokens: c.total_output_tokens,
            total_tokens: c.total_input_tokens + c.total_output_tokens,
            cost_usd: microdollars_to_usd(c.total_cost_microdollars),
            first_request_at: c.first_request_at.map(|t| t.to_rfc3339()),
            last_request_at: c.last_request_at.map(|t| t.to_rfc3339()),
            last_activity: c.last_activity_at.map(|t| t.to_rfc3339()),
        })
        .collect()
}

pub(super) fn build_user_summaries_json(
    summaries: &[contexts_list::ContextUserSummary],
    by_user: &HashMap<String, Vec<ContextItemView>>,
) -> Vec<UserSummaryView> {
    summaries
        .iter()
        .map(|s| {
            let nested = by_user.get(s.user_id.as_str()).cloned().unwrap_or_default();
            UserSummaryView {
                user_id: s.user_id.clone(),
                display_name: s.display_name.clone(),
                context_count: s.context_count,
                request_count: s.request_count,
                message_count: s.message_count,
                input_tokens: s.total_input_tokens,
                output_tokens: s.total_output_tokens,
                total_tokens: s.total_input_tokens + s.total_output_tokens,
                cost_usd: microdollars_to_usd(s.total_cost_microdollars),
                error_count: s.error_count,
                last_activity: s.last_activity_at.map(|t| t.to_rfc3339()),
                models: s.distinct_models.clone(),
                contexts: nested,
            }
        })
        .collect()
}
