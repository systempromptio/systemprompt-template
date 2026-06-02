//! View-model assembly for the Trace Explorer list page.
//!
//! Pure functions that turn repository rows + the parsed query into the serde
//! JSON the `perf-traces` template renders: filter ribbon, chips, pagination,
//! per-trace rows, and the display formatting they depend on.

use serde_json::json;
use urlencoding::encode as urlencode;

use crate::repositories::governance_grp::filter_options::{FilterOption, FilterOptions};
use crate::repositories::governance_grp::time_range::TimeRange;
use crate::repositories::perf_grp::traces::{
    TraceFilter, TraceSortColumn, TraceSortDir, TraceStats,
};

use super::{empty_to_none, TraceListQuery, BASE_URL};

pub(super) const fn sort_col_to_str(c: TraceSortColumn) -> &'static str {
    match c {
        TraceSortColumn::StartedAt => "started_at",
        TraceSortColumn::Duration => "duration",
        TraceSortColumn::SpanCount => "spans",
        TraceSortColumn::Cost => "cost",
        TraceSortColumn::Tokens => "tokens",
    }
}

pub(super) const fn sort_dir_to_str(d: TraceSortDir) -> &'static str {
    match d {
        TraceSortDir::Asc => "asc",
        TraceSortDir::Desc => "desc",
    }
}

pub(super) fn view_tabs_qs(range: TimeRange, preset: &str) -> String {
    format!(
        "preset={}&from={}&to={}",
        urlencode(preset),
        urlencode(&range.from.to_rfc3339()),
        urlencode(&range.to.to_rfc3339()),
    )
}

pub(super) fn entity_view_tabs(active: &str, qs: &str) -> serde_json::Value {
    const TABS: &[(&str, &str, &str)] = &[
        ("sessions", "Sessions", "/admin/entities/sessions"),
        ("traces", "Traces", "/admin/entities/traces"),
        ("requests", "Requests", "/admin/entities/requests"),
        ("contexts", "Contexts", "/admin/entities/contexts"),
    ];
    let items: Vec<_> = TABS
        .iter()
        .map(|(key, label, url)| {
            json!({
                "key": key,
                "label": label,
                "url": format!("{url}?{qs}"),
                "active": *key == active,
            })
        })
        .collect();
    serde_json::Value::Array(items)
}

pub(super) fn time_range_context(range: TimeRange, preset: &str) -> serde_json::Value {
    json!({
        "preset": preset,
        "from": range.from.to_rfc3339(),
        "to": range.to.to_rfc3339(),
        "base_url": BASE_URL,
        "query": "",
    })
}

pub(super) fn build_preserved(
    query: &TraceListQuery,
    range: TimeRange,
    preset: &str,
) -> Vec<serde_json::Value> {
    let mut out = vec![
        json!({ "name": "preset", "value": preset }),
        json!({ "name": "from",   "value": range.from.to_rfc3339() }),
        json!({ "name": "to",     "value": range.to.to_rfc3339() }),
    ];
    if query.error_only.as_deref() == Some("true") {
        out.push(json!({ "name": "error_only", "value": "true" }));
    }
    if query.deny_only.as_deref() == Some("true") {
        out.push(json!({ "name": "deny_only", "value": "true" }));
    }
    out
}

pub(super) fn build_chips(query: &TraceListQuery) -> Vec<serde_json::Value> {
    const GROUPS: &[(&str, &str)] = &[
        ("user_id", "User"),
        ("agent_id", "Agent"),
        ("agent_scope", "Scope"),
        ("policy", "Policy"),
        ("decision", "Decision"),
    ];
    let mut chips = Vec::new();
    for (param, label) in GROUPS {
        let val = match *param {
            "user_id" => query.user_id.as_deref(),
            "agent_id" => query.agent_id.as_deref(),
            "agent_scope" => query.agent_scope.as_deref(),
            "policy" => query.policy.as_deref(),
            "decision" => query.decision.as_deref(),
            _ => None,
        };
        let Some(v) = empty_to_none(val) else {
            continue;
        };
        chips.push(json!({
            "group_label": label,
            "label": v,
            "value": v,
            "remove_url": chip_remove_url(query, param),
        }));
    }
    chips
}

fn chip_remove_url(query: &TraceListQuery, drop: &str) -> String {
    let qs = preserved_query_string(query, &[drop]);
    if qs.is_empty() {
        BASE_URL.to_string()
    } else {
        format!("{BASE_URL}?{qs}")
    }
}

fn preserved_query_string(query: &TraceListQuery, drop: &[&str]) -> String {
    let pairs: [(&str, Option<&str>); 12] = [
        ("preset", query.preset.as_deref()),
        ("from", query.from.as_deref()),
        ("to", query.to.as_deref()),
        ("user_id", query.user_id.as_deref()),
        ("agent_id", query.agent_id.as_deref()),
        ("agent_scope", query.agent_scope.as_deref()),
        ("policy", query.policy.as_deref()),
        ("decision", query.decision.as_deref()),
        ("error_only", query.error_only.as_deref()),
        ("deny_only", query.deny_only.as_deref()),
        ("sort", query.sort.as_deref()),
        ("dir", query.dir.as_deref()),
    ];
    pairs
        .iter()
        .filter(|(name, _)| !drop.contains(name))
        .filter_map(|(name, val)| {
            val.filter(|s| !s.is_empty())
                .map(|v| format!("{}={}", name, urlencode(v)))
        })
        .collect::<Vec<_>>()
        .join("&")
}

pub(super) fn annotate_options(
    options: &FilterOptions,
    filter: &TraceFilter<'_>,
) -> serde_json::Value {
    let mut out = serde_json::Map::new();
    if !options.users.is_empty() {
        out.insert(
            "users".into(),
            annotate_group(&options.users, filter.user_id).into(),
        );
    }
    if !options.agents.is_empty() {
        out.insert(
            "agents".into(),
            annotate_group(&options.agents, filter.agent_id).into(),
        );
    }
    if !options.agent_scopes.is_empty() {
        out.insert(
            "agent_scopes".into(),
            annotate_group(&options.agent_scopes, filter.agent_scope).into(),
        );
    }
    if !options.policies.is_empty() {
        out.insert(
            "policies".into(),
            annotate_group(&options.policies, filter.policy).into(),
        );
    }
    if !options.decisions.is_empty() {
        out.insert(
            "decisions".into(),
            annotate_group(&options.decisions, filter.decision).into(),
        );
    }
    serde_json::Value::Object(out)
}

fn annotate_group(items: &[FilterOption], selected: Option<&str>) -> Vec<serde_json::Value> {
    items
        .iter()
        .map(|o| {
            json!({
                "id": o.id,
                "label": o.label,
                "count": o.count,
                "selected": selected.is_some_and(|s| s == o.id),
            })
        })
        .collect()
}

pub(super) fn build_pagination(
    query: &TraceListQuery,
    page: i64,
    total_pages: i64,
) -> serde_json::Value {
    let qs = preserved_query_string(query, &["page"]);
    let prefix = if qs.is_empty() {
        format!("{BASE_URL}?")
    } else {
        format!("{BASE_URL}?{qs}&")
    };
    let prev_url = (page > 0).then(|| format!("{prefix}page={}", page - 1));
    let next_url = (page + 1 < total_pages).then(|| format!("{prefix}page={}", page + 1));
    json!({
        "current_page": page + 1,
        "total_pages": total_pages,
        "has_prev": prev_url.is_some(),
        "has_next": next_url.is_some(),
        "prev_url": prev_url,
        "next_url": next_url,
    })
}

pub(super) fn serde_stats(s: &TraceStats) -> serde_json::Value {
    json!({
        "total_traces": s.total_traces,
        "error_count": s.error_count,
        "deny_count": s.deny_count,
        "p50_ms": s.p50_duration_ms,
        "p95_ms": s.p95_duration_ms,
        "p99_ms": s.p99_duration_ms,
    })
}
