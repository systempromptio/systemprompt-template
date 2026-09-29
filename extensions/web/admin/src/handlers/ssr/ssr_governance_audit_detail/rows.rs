//! The "AI requests" rows on the audit page: every call on the session,
//! each linking to its own audit page, with cost in dollars and latency in
//! the units the rest of the console uses.

use serde::Serialize;
use systemprompt::identifiers::AiRequestId;

use crate::handlers::ssr::entity_urls::request_detail_url;
use crate::handlers::ssr::format::{format_cost, format_duration_ms, local_time, short_num};
use crate::repositories::governance::chain::AiRequestSummary;

#[derive(Debug, Serialize)]
pub(super) struct RequestRowView {
    id: String,
    request_id: AiRequestId,
    url: String,
    is_primary: bool,
    provider: String,
    model: String,
    status: String,
    is_ok: bool,
    tokens_display: String,
    cost_display: String,
    latency_display: String,
    created_at_local: String,
}

pub(super) fn build_request_row(r: &AiRequestSummary, is_primary: bool) -> RequestRowView {
    RequestRowView {
        id: r.id.clone(),
        request_id: r.request_id.clone(),
        url: request_detail_url(&r.request_id),
        is_primary,
        provider: r.provider.clone().unwrap_or_else(|| "—".to_owned()),
        model: r.model.clone().unwrap_or_else(|| "—".to_owned()),
        status: r.status.clone(),
        is_ok: !super::is_failed_status(&r.status),
        tokens_display: match (r.input_tokens, r.output_tokens) {
            (Some(i), Some(o)) => {
                format!("{} → {}", short_num(i64::from(i)), short_num(i64::from(o)))
            },
            _ => "—".to_owned(),
        },
        cost_display: format_cost(r.cost_microdollars),
        latency_display: r
            .latency_ms
            .map_or_else(|| "—".to_owned(), |ms| format_duration_ms(i64::from(ms))),
        created_at_local: local_time(r.created_at),
    }
}
