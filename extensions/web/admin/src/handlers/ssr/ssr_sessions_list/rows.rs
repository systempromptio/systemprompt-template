//! One repository row → one table row.

use crate::handlers::ssr::entity_urls::session_detail_url;
use crate::handlers::ssr::format::{
    format_cost, format_span, format_token_total, local_time, short_id,
};
use crate::repositories::analytics::sessions_list::SessionListItem;

use super::context::SessionRowView;

pub(super) fn session_row(s: &SessionListItem) -> SessionRowView {
    let (source_label, source_variant) = match (s.has_gateway, s.has_hooks) {
        (true, true) => ("Both", "success"),
        (true, false) => ("Gateway", "info"),
        (false, true) => ("Hooks", "secondary"),
        // Why: Unreachable: a row exists only because one side of the join matched.
        (false, false) => ("—", "secondary"),
    };

    let user_label = s
        .display_name
        .clone()
        .or_else(|| s.user_id.as_ref().map(|u| short_id(u.as_str())))
        .unwrap_or_else(|| "—".to_owned());

    SessionRowView {
        session_id: s.session_id.clone(),
        session_id_short: short_id(s.session_id.as_str()),
        detail_url: session_detail_url(&s.session_id),
        ai_title: s.ai_title.clone(),
        user_id: s.user_id.clone(),
        user_label,
        user_url: s
            .user_id
            .as_ref()
            .map(|u| format!("/admin/user?id={}", urlencoding::encode(u.as_str()))),
        department: s.department.clone(),
        source_label,
        source_variant,
        model: s.model.clone(),
        client_source: s.client_source.clone(),
        request_count: s.request_count,
        context_count: s.context_count,
        trace_count: s.trace_count,
        tool_uses: s.tool_uses,
        tokens_display: format_token_total(s.total_input_tokens + s.total_output_tokens),
        cost_display: format_cost(s.total_cost_microdollars),
        duration_display: format_span(s.started_at, s.last_activity_at),
        started_at: s.started_at.map(|t| t.to_rfc3339()),
        started_at_local: s.started_at.map(local_time),
        error_count: s.error_count,
        has_error: s.error_count > 0,
        status_label: status_label(s),
    }
}

fn status_label(s: &SessionListItem) -> String {
    if s.error_count > 0 {
        let noun = if s.error_count == 1 {
            "error"
        } else {
            "errors"
        };
        return format!("{} {noun}", s.error_count);
    }
    s.status
        .clone()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "OK".to_owned())
}
