//! `/admin/governance/decisions` — forensic audit trail.
//!
//! URL-bound surface combining a `time-range` picker, an
//! `identity-filter-ribbon`, and a paged decisions table. Every row is
//! `[data-chain-id]`-clickable; the `chain-drawer` JS opens the full chain
//! envelope on click.
//!
//! Replaces the old summary-card "decisions" page; the four KPI cards now
//! live on Page 1 (the governance overview).

use std::sync::Arc;

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use urlencoding::encode as urlencode;

use crate::repositories::governance_grp::filter_options::{
    fetch_filter_options, FilterOption, FilterOptions,
};
use crate::repositories::governance_grp::paged::{
    fetch_decisions_paged, DecisionFilter, DecisionRow, SortColumn, SortDir, SortSpec,
};
use crate::repositories::governance_grp::time_range::{
    parse_time_range, TimeRange, TimeRangeQuery,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

const BASE_URL: &str = "/admin/governance/decisions";
const DEFAULT_PAGE_SIZE: i64 = 50;

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    // time range
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    // identity filters
    pub user_id: Option<String>,
    pub agent_id: Option<String>,
    pub agent_scope: Option<String>,
    pub policy: Option<String>,
    pub decision: Option<String>,
    pub tool_name: Option<String>,
    pub q: Option<String>,
    // sort + pagination
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub page: Option<i64>,
}

pub async fn governance_audit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AuditQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });
    let filter = filter_from_query(&query);
    let sort = sort_from_query(&query);
    let page = query.page.unwrap_or(0).max(0);
    let offset = page * DEFAULT_PAGE_SIZE;

    let (rows_res, options_res) = tokio::join!(
        fetch_decisions_paged(&pool, &filter, range, sort, DEFAULT_PAGE_SIZE, offset),
        fetch_filter_options(&pool, range),
    );
    let (rows, total_count) = rows_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_decisions_paged failed");
        (Vec::new(), 0)
    });
    let options = options_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_filter_options failed");
        FilterOptions::default()
    });

    let preserved = build_preserved(&query, &range);
    let chips = build_chips(&query);
    let filter_ribbon_options = annotate_options(&options, &filter);

    let total_pages = if total_count == 0 {
        1
    } else {
        (total_count + DEFAULT_PAGE_SIZE - 1) / DEFAULT_PAGE_SIZE
    };
    let pagination = build_pagination(&query, page, total_pages);

    let data = json!({
        "page": "governance",
        "title": "Audit Trail",
        "time_range": time_range_context(&range, &query),
        "filter_ribbon": {
            "base_url": BASE_URL,
            "preserved": preserved,
            "options": filter_ribbon_options,
            "chips": chips,
        },
        "decisions": rows.iter().map(decision_to_json).collect::<Vec<_>>(),
        "has_decisions": !rows.is_empty(),
        "total_count": total_count,
        "page_size": DEFAULT_PAGE_SIZE,
        "page_index": page,
        "page_count": total_pages,
        "pagination": pagination,
        "search_query": query.q.clone().unwrap_or_default(),
        "csv_url": csv_url(&query, &range),
        "sort": sort_to_str(sort.column),
        "dir": dir_to_str(sort.dir),
    });

    super::render_page(&engine, "governance-audit", &data, &user_ctx, &mkt_ctx)
}

fn filter_from_query(query: &AuditQuery) -> DecisionFilter {
    DecisionFilter {
        user_id: empty_to_none(query.user_id.as_ref()),
        agent_id: empty_to_none(query.agent_id.as_ref()),
        agent_scope: empty_to_none(query.agent_scope.as_ref()),
        policy: empty_to_none(query.policy.as_ref()),
        decision: empty_to_none(query.decision.as_ref()),
        tool_name: empty_to_none(query.tool_name.as_ref()),
        search: empty_to_none(query.q.as_ref()),
    }
}

fn empty_to_none(v: Option<&String>) -> Option<String> {
    v.map(String::as_str)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
}

fn sort_from_query(query: &AuditQuery) -> SortSpec {
    let column = match query.sort.as_deref() {
        Some("cost") => SortColumn::Cost,
        Some("latency") => SortColumn::Latency,
        Some("policy") => SortColumn::Policy,
        _ => SortColumn::CreatedAt,
    };
    let dir = match query.dir.as_deref() {
        Some("asc") => SortDir::Asc,
        _ => SortDir::Desc,
    };
    SortSpec { column, dir }
}

const fn sort_to_str(c: SortColumn) -> &'static str {
    match c {
        SortColumn::CreatedAt => "created_at",
        SortColumn::Cost => "cost",
        SortColumn::Latency => "latency",
        SortColumn::Policy => "policy",
    }
}

const fn dir_to_str(d: SortDir) -> &'static str {
    match d {
        SortDir::Asc => "asc",
        SortDir::Desc => "desc",
    }
}

fn time_range_context(range: &TimeRange, query: &AuditQuery) -> serde_json::Value {
    let preset = query.preset.clone().unwrap_or_else(|| {
        // Mirror parse_time_range's default preset name.
        if query.from.is_some() && query.to.is_some() {
            "custom".to_string()
        } else {
            "24h".to_string()
        }
    });
    let qs = preserved_query_string(query, &["preset", "from", "to"]);
    let q_suffix = if qs.is_empty() {
        String::new()
    } else {
        format!("&{qs}")
    };
    json!({
        "preset": preset,
        "from": range.from.to_rfc3339(),
        "to": range.to.to_rfc3339(),
        "base_url": BASE_URL,
        "query": q_suffix,
    })
}

fn build_preserved(query: &AuditQuery, range: &TimeRange) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    let preset = query.preset.clone().unwrap_or_else(|| {
        if query.from.is_some() && query.to.is_some() {
            "custom".to_string()
        } else {
            "24h".to_string()
        }
    });
    out.push(json!({ "name": "preset", "value": preset }));
    out.push(json!({ "name": "from", "value": range.from.to_rfc3339() }));
    out.push(json!({ "name": "to", "value": range.to.to_rfc3339() }));
    if let Some(q) = empty_to_none(query.q.as_ref()) {
        out.push(json!({ "name": "q", "value": q }));
    }
    out
}

fn build_chips(query: &AuditQuery) -> Vec<serde_json::Value> {
    const GROUPS: &[(&str, &str)] = &[
        ("user_id", "User"),
        ("agent_id", "Agent"),
        ("agent_scope", "Scope"),
        ("policy", "Policy"),
        ("decision", "Decision"),
        ("tool_name", "Tool"),
    ];
    let mut chips = Vec::new();
    for (param, label) in GROUPS {
        let val: Option<&String> = match *param {
            "user_id" => query.user_id.as_ref(),
            "agent_id" => query.agent_id.as_ref(),
            "agent_scope" => query.agent_scope.as_ref(),
            "policy" => query.policy.as_ref(),
            "decision" => query.decision.as_ref(),
            "tool_name" => query.tool_name.as_ref(),
            _ => None,
        };
        let Some(v) = empty_to_none(val) else { continue };
        chips.push(json!({
            "group_label": label,
            "label": v,
            "value": v,
            "remove_url": chip_remove_url(query, param),
        }));
    }
    chips
}

fn chip_remove_url(query: &AuditQuery, drop_param: &str) -> String {
    let qs = preserved_query_string(query, &[drop_param]);
    if qs.is_empty() {
        BASE_URL.to_string()
    } else {
        format!("{BASE_URL}?{qs}")
    }
}

/// Build a URL-encoded query string of all populated `AuditQuery` fields,
/// excluding any whose name appears in `drop`.
fn preserved_query_string(query: &AuditQuery, drop: &[&str]) -> String {
    let mut parts: Vec<String> = Vec::new();
    let pairs: [(&str, Option<&str>); 12] = [
        ("preset", query.preset.as_deref()),
        ("from", query.from.as_deref()),
        ("to", query.to.as_deref()),
        ("user_id", query.user_id.as_deref()),
        ("agent_id", query.agent_id.as_deref()),
        ("agent_scope", query.agent_scope.as_deref()),
        ("policy", query.policy.as_deref()),
        ("decision", query.decision.as_deref()),
        ("tool_name", query.tool_name.as_deref()),
        ("q", query.q.as_deref()),
        ("sort", query.sort.as_deref()),
        ("dir", query.dir.as_deref()),
    ];
    for (name, value) in pairs {
        if drop.contains(&name) {
            continue;
        }
        let Some(v) = value.filter(|s| !s.is_empty()) else {
            continue;
        };
        parts.push(format!("{}={}", name, urlencode(v)));
    }
    parts.join("&")
}

fn annotate_options(options: &FilterOptions, filter: &DecisionFilter) -> serde_json::Value {
    json!({
        "users": annotate_group(&options.users, filter.user_id.as_deref()),
        "agents": annotate_group(&options.agents, filter.agent_id.as_deref()),
        "agent_scopes": annotate_group(&options.agent_scopes, filter.agent_scope.as_deref()),
        "policies": annotate_group(&options.policies, filter.policy.as_deref()),
        "decisions": annotate_group(&options.decisions, filter.decision.as_deref()),
    })
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

fn build_pagination(
    query: &AuditQuery,
    page: i64,
    total_pages: i64,
) -> serde_json::Value {
    let qs_no_page = preserved_query_string(query, &["page"]);
    let prefix = if qs_no_page.is_empty() {
        format!("{BASE_URL}?")
    } else {
        format!("{BASE_URL}?{qs_no_page}&")
    };
    let prev_url = if page > 0 {
        Some(format!("{prefix}page={}", page - 1))
    } else {
        None
    };
    let next_url = if page + 1 < total_pages {
        Some(format!("{prefix}page={}", page + 1))
    } else {
        None
    };
    json!({
        "current_page": page + 1,
        "total_pages": total_pages,
        "has_prev": prev_url.is_some(),
        "has_next": next_url.is_some(),
        "prev_url": prev_url,
        "next_url": next_url,
    })
}

fn csv_url(query: &AuditQuery, range: &TimeRange) -> String {
    let mut qs = preserved_query_string(query, &["page", "sort", "dir"]);
    if !qs.contains("from=") {
        if !qs.is_empty() {
            qs.push('&');
        }
        qs.push_str(&format!("from={}", urlencode(&range.from.to_rfc3339())));
    }
    if !qs.contains("to=") {
        if !qs.is_empty() {
            qs.push('&');
        }
        qs.push_str(&format!("to={}", urlencode(&range.to.to_rfc3339())));
    }
    format!("/admin/api/governance/decisions.csv?{qs}")
}

fn decision_to_json(r: &DecisionRow) -> serde_json::Value {
    let stage = pipeline_stage(&r.policy);
    let evidence = extract_evidence(&r.evaluated_rules, &r.policy);
    json!({
        "id": r.id,
        "trace_id": r.trace_id,
        "session_id": r.session_id,
        "user_id": r.user_id,
        "tool_name": r.tool_name,
        "agent_id": r.agent_id,
        "agent_scope": r.agent_scope,
        "decision": r.decision,
        "is_denied": r.decision == "deny",
        "policy": r.policy,
        "pipeline_stage": stage.id,
        "pipeline_stage_label": stage.label,
        "reason": r.reason,
        "rule_name": evidence.rule_name,
        "pattern_name": evidence.pattern_name,
        "evidence_redacted": evidence.evidence_redacted,
        "risk_score": evidence.risk_score,
        "cost_microdollars": r.cost_microdollars,
        "cost_display": format_cost(r.cost_microdollars),
        "latency_ms": r.latency_ms,
        "created_at": r.created_at.to_rfc3339(),
        "created_at_local": r
            .created_at
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    })
}

struct PipelineStage {
    id: &'static str,
    label: &'static str,
}

fn pipeline_stage(policy: &str) -> PipelineStage {
    match policy {
        "scope_check" | "scope" => PipelineStage {
            id: "scope",
            label: "Scope",
        },
        "secret_scan" | "secret_injection" => PipelineStage {
            id: "secret",
            label: "Secret",
        },
        "tool_blocklist" | "blocklist" => PipelineStage {
            id: "blocklist",
            label: "Blocklist",
        },
        "rate_limit" => PipelineStage {
            id: "rate_limit",
            label: "Rate-Limit",
        },
        _ => PipelineStage {
            id: "other",
            label: "Other",
        },
    }
}

#[derive(Debug, Default)]
struct EvidenceFields {
    rule_name: Option<String>,
    pattern_name: Option<String>,
    evidence_redacted: Option<String>,
    risk_score: Option<f64>,
}

/// `evaluated_rules` is a JSONB array of `{ rule, result, detail, ... }` rows.
/// The interesting hit for the matched-evidence column is the first entry whose
/// `rule` matches the row's `policy` and whose `result` is not "skip".
fn extract_evidence(rules: &serde_json::Value, policy: &str) -> EvidenceFields {
    let Some(arr) = rules.as_array() else {
        return EvidenceFields::default();
    };
    let entry = arr
        .iter()
        .find(|e| {
            let rule_match = e
                .get("rule")
                .and_then(|v| v.as_str())
                .is_some_and(|s| s == policy);
            let not_skipped = e.get("result").and_then(|v| v.as_str()) != Some("skip");
            rule_match && not_skipped
        })
        .or_else(|| arr.first());
    let Some(e) = entry else {
        return EvidenceFields::default();
    };
    EvidenceFields {
        rule_name: e
            .get("rule_name")
            .or_else(|| e.get("rule"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        pattern_name: e
            .get("pattern_name")
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        evidence_redacted: e
            .get("evidence_redacted")
            .or_else(|| e.get("detail"))
            .and_then(|v| v.as_str())
            .map(ToString::to_string),
        risk_score: e.get("risk_score").and_then(serde_json::Value::as_f64),
    }
}

fn format_cost(microdollars: Option<i64>) -> String {
    let Some(m) = microdollars else {
        return "—".to_string();
    };
    let dollars = m as f64 / 1_000_000.0;
    if dollars == 0.0 {
        "$0".to_string()
    } else if dollars < 0.01 {
        format!("${dollars:.6}")
    } else {
        format!("${dollars:.4}")
    }
}
