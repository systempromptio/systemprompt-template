//! View-model assembly for the Trace Explorer list page.
//!
//! Pure functions that turn repository rows + the parsed query into the typed
//! context the `perf-traces` template renders: filter ribbon, chips,
//! pagination, per-trace rows, and the display formatting they depend on.

use urlencoding::encode as urlencode;

use crate::repositories::governance_grp::filter_options::{FilterOption, FilterOptions};
use crate::repositories::governance_grp::time_range::TimeRange;
use crate::repositories::perf_grp::traces::{
    TraceFilter, TraceSortColumn, TraceSortDir, TraceStats,
};

use super::context::{
    AnnotatedOption, Chip, EntityViewTab, FilterOptionsView, Pagination, Preserved, StatsView,
    TimeRangeContext,
};
use super::{BASE_URL, TraceListQuery, empty_to_none};

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

pub(super) fn entity_view_tabs(active: &str, qs: &str) -> Vec<EntityViewTab> {
    const TABS: &[(&str, &str, &str)] = &[
        ("sessions", "Sessions", "/admin/entities/sessions"),
        ("traces", "Traces", "/admin/entities/traces"),
        ("requests", "Requests", "/admin/entities/requests"),
        ("contexts", "Contexts", "/admin/entities/contexts"),
    ];
    TABS.iter()
        .map(|(key, label, url)| EntityViewTab {
            key,
            label,
            url: format!("{url}?{qs}"),
            active: *key == active,
        })
        .collect()
}

pub(super) fn time_range_context(range: TimeRange, preset: &str) -> TimeRangeContext {
    TimeRangeContext {
        preset: preset.to_owned(),
        from: range.from.to_rfc3339(),
        to: range.to.to_rfc3339(),
        base_url: BASE_URL,
        query: "",
    }
}

pub(super) fn build_preserved(
    query: &TraceListQuery,
    range: TimeRange,
    preset: &str,
) -> Vec<Preserved> {
    let mut out = vec![
        Preserved {
            name: "preset",
            value: preset.to_owned(),
        },
        Preserved {
            name: "from",
            value: range.from.to_rfc3339(),
        },
        Preserved {
            name: "to",
            value: range.to.to_rfc3339(),
        },
    ];
    if query.error_only.as_deref() == Some("true") {
        out.push(Preserved {
            name: "error_only",
            value: "true".to_owned(),
        });
    }
    if query.deny_only.as_deref() == Some("true") {
        out.push(Preserved {
            name: "deny_only",
            value: "true".to_owned(),
        });
    }
    out
}

pub(super) fn build_chips(query: &TraceListQuery) -> Vec<Chip> {
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
            "user_id" => query
                .user_id
                .as_ref()
                .map(systemprompt::identifiers::UserId::as_str),
            "agent_id" => query
                .agent_id
                .as_ref()
                .map(systemprompt::identifiers::AgentId::as_str),
            "agent_scope" => query.agent_scope.as_deref(),
            "policy" => query.policy.as_deref(),
            "decision" => query.decision.as_deref(),
            _ => None,
        };
        let Some(v) = empty_to_none(val) else {
            continue;
        };
        chips.push(Chip {
            group_label: label,
            label: v.to_owned(),
            value: v.to_owned(),
            remove_url: chip_remove_url(query, param),
        });
    }
    chips
}

fn chip_remove_url(query: &TraceListQuery, drop: &str) -> String {
    let qs = preserved_query_string(query, &[drop]);
    if qs.is_empty() {
        BASE_URL.to_owned()
    } else {
        format!("{BASE_URL}?{qs}")
    }
}

fn preserved_query_string(query: &TraceListQuery, drop: &[&str]) -> String {
    let pairs: [(&str, Option<&str>); 12] = [
        ("preset", query.preset.as_deref()),
        ("from", query.from.as_deref()),
        ("to", query.to.as_deref()),
        (
            "user_id",
            query
                .user_id
                .as_ref()
                .map(systemprompt::identifiers::UserId::as_str),
        ),
        (
            "agent_id",
            query
                .agent_id
                .as_ref()
                .map(systemprompt::identifiers::AgentId::as_str),
        ),
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
) -> FilterOptionsView {
    FilterOptionsView {
        users: annotate_group(&options.users, filter.user_id),
        agents: annotate_group(&options.agents, filter.agent_id),
        agent_scopes: annotate_group(&options.agent_scopes, filter.agent_scope),
        policies: annotate_group(&options.policies, filter.policy),
        decisions: annotate_group(&options.decisions, filter.decision),
    }
}

fn annotate_group(items: &[FilterOption], selected: Option<&str>) -> Vec<AnnotatedOption> {
    items
        .iter()
        .map(|o| AnnotatedOption {
            id: o.id.clone(),
            label: o.label.clone(),
            count: o.count,
            selected: selected.is_some_and(|s| s == o.id),
        })
        .collect()
}

pub(super) fn build_pagination(query: &TraceListQuery, page: i64, total_pages: i64) -> Pagination {
    let qs = preserved_query_string(query, &["page"]);
    let prefix = if qs.is_empty() {
        format!("{BASE_URL}?")
    } else {
        format!("{BASE_URL}?{qs}&")
    };
    let prev_url = (page > 0).then(|| format!("{prefix}page={}", page - 1));
    let next_url = (page + 1 < total_pages).then(|| format!("{prefix}page={}", page + 1));
    Pagination {
        current_page: page + 1,
        total_pages,
        has_prev: prev_url.is_some(),
        has_next: next_url.is_some(),
        prev_url,
        next_url,
    }
}

pub(super) const fn serde_stats(s: &TraceStats) -> StatsView {
    StatsView {
        total_traces: s.total_traces,
        error_count: s.error_count,
        deny_count: s.deny_count,
        p50_ms: s.p50_duration_ms,
        p95_ms: s.p95_duration_ms,
        p99_ms: s.p99_duration_ms,
    }
}
