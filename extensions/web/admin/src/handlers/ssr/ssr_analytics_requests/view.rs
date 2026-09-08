//! Row and query mapping for the Inference Requests page.
//!
//! Turns the parsed query into a `RequestFilter` and a sort spec, and each
//! repository row into the shape the table renders. The tiles, headers and
//! pickers above the table live in `summary`.

use crate::handlers::ssr::format::{format_cost, format_duration_ms};
use crate::handlers::ssr::list_view::{ScopeFilterView, scope_filter_view};
use crate::repositories::analytics::requests::{
    RequestFilter, RequestRow, RequestSortColumn, RequestSortSpec, SortDir,
};
use crate::repositories::scope::{ScopeRequest, SubjectScope};
use crate::types::UserContext;
use crate::util::time_range::TimeRange;

use super::context::{RequestListRowView, TimeRangeView};
use super::summary::group_digits;
use super::urls::preserved_query_string;
use super::{BASE_URL, RequestsQuery};

const EM_DASH: &str = "\u{2014}";

// Why: `scope` is the caller's resolved user set — a console role's pick or
// everything, everyone else's own groups whatever the query said. The group
// and project on the filter are the narrower, exclusive attribution key, so a
// person counts against one container rather than every one they belong to.
pub(super) fn filter_from_query(query: &RequestsQuery, scope: SubjectScope) -> RequestFilter {
    RequestFilter {
        scope,
        user_id: query.user_id.clone().filter(|u| !u.as_str().is_empty()),
        agent_id: query.agent_id.clone().filter(|a| !a.as_str().is_empty()),
        model: empty_to_none(query.model.as_ref()),
        provider: empty_to_none(query.provider.as_ref()),
        status: empty_to_none(query.status.as_ref()),
        search: empty_to_none(query.q.as_ref()),
        tool: empty_to_none(query.tool.as_ref()),
        group: empty_to_none(query.group.as_ref()),
        project: empty_to_none(query.project.as_ref()),
    }
}

pub(super) async fn scope_filter(
    pool: &sqlx::PgPool,
    user_ctx: &UserContext,
    request: &ScopeRequest,
    query: &RequestsQuery,
) -> ScopeFilterView {
    let hidden = [
        ("tab", query.tab.as_deref()),
        ("preset", query.preset.as_deref()),
        ("from", query.from.as_deref()),
        ("to", query.to.as_deref()),
        ("sort", query.sort.as_deref()),
        ("dir", query.dir.as_deref()),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_owned(), v.unwrap_or_default().to_owned()))
    .collect();
    scope_filter_view(pool, user_ctx, request, BASE_URL, hidden).await
}

fn empty_to_none(v: Option<&String>) -> Option<String> {
    v.map(String::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

pub(super) fn sort_from_query(query: &RequestsQuery) -> RequestSortSpec {
    let column = match query.sort.as_deref() {
        Some("cost") => RequestSortColumn::Cost,
        Some("latency") => RequestSortColumn::Latency,
        Some("tokens") => RequestSortColumn::Tokens,
        _ => RequestSortColumn::CreatedAt,
    };
    let dir = match query.dir.as_deref() {
        Some("asc") => SortDir::Asc,
        _ => SortDir::Desc,
    };
    RequestSortSpec { column, dir }
}

pub(super) fn row_count_label(total: i64) -> String {
    if total == 1 {
        "1 request".to_owned()
    } else {
        format!("{} requests", group_digits(total))
    }
}

// Why: a request rejected before route resolution carries no provider or
// model; the cell reads as an em dash rather than a blank the eye skips.
fn dash_if_empty(value: &str) -> String {
    if value.is_empty() {
        EM_DASH.to_owned()
    } else {
        value.to_owned()
    }
}

fn short(value: &str, keep: usize) -> String {
    if value.chars().count() > keep {
        format!("{}\u{2026}", value.chars().take(keep).collect::<String>())
    } else {
        value.to_owned()
    }
}

pub(super) fn request_row_to_json(r: &RequestRow) -> RequestListRowView {
    RequestListRowView {
        detail_url: format!("{BASE_URL}/{}", r.id),
        id: r.id.clone(),
        request_id: r.request_id.clone(),
        trace_id_short: r.trace_id.as_ref().map(|t| short(t.as_str(), 8)),
        trace_id: r.trace_id.clone(),
        session_id: r.session_id.clone(),
        user_url: format!("/admin/users/{}", r.user_id.as_str()),
        user_id: r.user_id.clone(),
        user_label: r
            .user_label
            .clone()
            .unwrap_or_else(|| r.user_id.as_str().to_owned()),
        project_label: r
            .project_name
            .clone()
            .or_else(|| r.project_id.clone())
            .unwrap_or_else(|| "Unattributed".to_owned()),
        project_url: r
            .project_id
            .as_ref()
            .map(|id| format!("/admin/projects/{id}")),
        project_id: r.project_id.clone(),
        group_label: r
            .group_name
            .clone()
            .or_else(|| r.group_id.clone())
            .unwrap_or_else(|| "Unattributed".to_owned()),
        group_url: r.group_id.as_ref().map(|id| format!("/admin/groups/{id}")),
        group_id: r.group_id.clone(),
        is_unattributed: r.group_id.is_none() && r.project_id.is_none(),
        provider: dash_if_empty(&r.provider),
        model: dash_if_empty(&r.model),
        has_model: !r.model.is_empty(),
        is_error: is_error_status(&r.status),
        is_rejected: r.status == "rejected",
        status: r.status.clone(),
        input_tokens: r.input_tokens,
        output_tokens: r.output_tokens,
        tokens_display: match (r.input_tokens, r.output_tokens) {
            (None, None) => EM_DASH.to_owned(),
            (i, o) => format!("{} / {}", i.unwrap_or(0), o.unwrap_or(0)),
        },
        cost_microdollars: r.cost_microdollars,
        cost_display: format_cost(r.cost_microdollars),
        latency_display: r.latency_ms.map_or_else(
            || EM_DASH.to_owned(),
            |ms| format_duration_ms(i64::from(ms)),
        ),
        latency_ms: r.latency_ms,
        error_message: r.error_message.clone(),
        governance_display: if r.deny_count > 0 {
            format!("{} deny", r.deny_count)
        } else if r.decision_count > 0 {
            format!("{} allow", r.decision_count)
        } else {
            EM_DASH.to_owned()
        },
        decision_count: r.decision_count,
        deny_count: r.deny_count,
        is_denied_preflight: r.deny_count > 0,
        tool_call_count: r.tool_call_count,
        created_at: r.created_at.to_rfc3339(),
        created_at_time: r
            .created_at
            .with_timezone(&chrono::Local)
            .format("%H:%M:%S")
            .to_string(),
        created_at_day: r
            .created_at
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d")
            .to_string(),
    }
}

fn is_error_status(status: &str) -> bool {
    !matches!(status, "completed" | "pending" | "streaming")
}

pub(super) fn time_range_context(
    query: &RequestsQuery,
    range: &TimeRange,
    auto_widened: Option<&'static str>,
) -> TimeRangeView {
    let preset = query.preset.clone().unwrap_or_else(|| {
        if query.from.is_some() && query.to.is_some() {
            "custom".to_owned()
        } else {
            auto_widened.unwrap_or("24h").to_owned()
        }
    });
    let qs = preserved_query_string(query, &["preset", "from", "to"]);
    let q_suffix = if qs.is_empty() {
        String::new()
    } else {
        format!("&{qs}")
    };
    TimeRangeView {
        preset,
        from: range.from.to_rfc3339(),
        to: range.to.to_rfc3339(),
        base_url: BASE_URL,
        query: q_suffix,
        auto_widened,
        rejected: range.rejected_bounds,
    }
}
