//! `/admin/governance/identity` — heatmap + top-10 violators.
//!
//! Group-by toggle (User / Tenant / Agent / Agent Scope) drives both the
//! heatmap (rows = identities, cols = policies, cells = deny count) and the
//! ranked top-violators table (deny count + risk score). Click a cell to
//! jump to the audit trail filtered to that identity + policy. Click an
//! identity row to open its profile.

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

use crate::repositories::governance_grp::identity::{
    fetch_violations_by_identity, IdentityGroupBy as RepoGroupBy, IdentityViolations,
};
use crate::repositories::governance_grp::risk_score::{
    compute_risk_score, fetch_violation_counts, weights, IdentityGroupBy, RiskScore,
    ViolationCounts,
};
use crate::repositories::governance_grp::time_range::{parse_time_range, TimeRangeQuery};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

const BASE_URL: &str = "/admin/governance/identity";
const TOP_N: usize = 10;

#[derive(Debug, Deserialize)]
pub struct IdentityIndexQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub group_by: Option<String>,
}

pub async fn governance_identity_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<IdentityIndexQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });
    let group_by = IdentityGroupBy::parse(query.group_by.as_deref());
    let repo_group_by = repo_group_by(group_by);
    let weights = weights();

    let (heatmap_res, counts_res) = tokio::join!(
        fetch_violations_by_identity(&pool, range, repo_group_by),
        fetch_violation_counts(&pool, range, group_by),
    );
    let heatmap_rows = heatmap_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_violations_by_identity failed");
        Vec::new()
    });
    let counts = counts_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_violation_counts failed");
        Vec::new()
    });

    let policies = collect_policies(&heatmap_rows);
    let heatmap = build_heatmap(&heatmap_rows, &policies, group_by);
    let max_cell = heatmap.iter().map(|h| h.max_cell).max().unwrap_or(0);
    let top = build_top_violators(&counts, weights);

    let group_options = group_by_options(group_by);

    let data = json!({
        "page": "governance-identity",
        "title": "Identity & Violations",
        "time_range": time_range_context(&query, &range),
        "group_by": group_by.as_str(),
        "group_options": group_options,
        "policies": policies,
        "heatmap": heatmap,
        "max_cell": max_cell,
        "has_data": !heatmap.is_empty(),
        "top_violators": top,
    });

    super::render_page(&engine, "governance-identity", &data, &user_ctx, &mkt_ctx)
}

const fn repo_group_by(g: IdentityGroupBy) -> RepoGroupBy {
    match g {
        IdentityGroupBy::User => RepoGroupBy::User,
        IdentityGroupBy::Agent => RepoGroupBy::Agent,
        IdentityGroupBy::AgentScope => RepoGroupBy::AgentScope,
    }
}

fn group_by_options(active: IdentityGroupBy) -> Vec<serde_json::Value> {
    [
        (IdentityGroupBy::User, "User"),
        (IdentityGroupBy::Agent, "Agent"),
        (IdentityGroupBy::AgentScope, "Scope"),
    ]
    .into_iter()
    .map(|(g, label)| {
        json!({
            "id": g.as_str(),
            "label": label,
            "active": active.as_str() == g.as_str(),
        })
    })
    .collect()
}

fn collect_policies(rows: &[IdentityViolations]) -> Vec<String> {
    let mut seen = std::collections::BTreeSet::new();
    for r in rows {
        seen.insert(r.policy.clone());
    }
    seen.into_iter().collect()
}

#[derive(Debug, Clone)]
struct HeatmapRowBuilder {
    identity_id: String,
    cells: std::collections::HashMap<String, i64>,
    total_deny: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
struct HeatmapCell {
    policy: String,
    deny_count: i64,
    audit_url: String,
}

#[derive(Debug, Clone, serde::Serialize)]
struct HeatmapRow {
    identity_id: String,
    profile_url: String,
    audit_url: String,
    total_deny: i64,
    cells: Vec<HeatmapCell>,
    max_cell: i64,
}

fn build_heatmap(
    rows: &[IdentityViolations],
    policies: &[String],
    group_by: IdentityGroupBy,
) -> Vec<HeatmapRow> {
    let mut by_identity: std::collections::BTreeMap<String, HeatmapRowBuilder> =
        std::collections::BTreeMap::new();
    for r in rows {
        let entry = by_identity
            .entry(r.identity_id.clone())
            .or_insert_with(|| HeatmapRowBuilder {
                identity_id: r.identity_id.clone(),
                cells: std::collections::HashMap::new(),
                total_deny: 0,
            });
        entry.cells.insert(r.policy.clone(), r.deny_count);
        entry.total_deny += r.deny_count;
    }

    let mut out: Vec<HeatmapRow> = by_identity
        .into_values()
        .map(|b| {
            let cells: Vec<HeatmapCell> = policies
                .iter()
                .map(|p| {
                    let count = b.cells.get(p).copied().unwrap_or(0);
                    HeatmapCell {
                        policy: p.clone(),
                        deny_count: count,
                        audit_url: audit_url_for(group_by, &b.identity_id, Some(p)),
                    }
                })
                .collect();
            let max_cell = cells.iter().map(|c| c.deny_count).max().unwrap_or(0);
            HeatmapRow {
                profile_url: profile_url_for(group_by, &b.identity_id),
                audit_url: audit_url_for(group_by, &b.identity_id, None),
                identity_id: b.identity_id,
                total_deny: b.total_deny,
                cells,
                max_cell,
            }
        })
        .collect();
    out.sort_by_key(|row| std::cmp::Reverse(row.total_deny));
    out
}

fn build_top_violators(
    counts: &[(String, ViolationCounts)],
    weights: super::super::super::repositories::governance_grp::risk_score::RiskScoreWeights,
) -> Vec<serde_json::Value> {
    let mut scored: Vec<(String, ViolationCounts, RiskScore)> = counts
        .iter()
        .map(|(id, v)| (id.clone(), *v, compute_risk_score(v, weights)))
        .collect();
    scored.sort_by(|a, b| {
        b.2.score
            .partial_cmp(&a.2.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored
        .into_iter()
        .take(TOP_N)
        .map(|(id, v, r)| {
            json!({
                "identity_id": id,
                "deny_count": v.deny_count,
                "secret_breach_count": v.secret_breach_count,
                "scope_violation_count": v.scope_violation_count,
                "activity_volume": v.activity_volume,
                "risk_score": format!("{:.1}", r.score),
                "risk_band": risk_band(r.score),
            })
        })
        .collect()
}

fn risk_band(score: f64) -> &'static str {
    if score >= 60.0 {
        "high"
    } else if score >= 25.0 {
        "med"
    } else {
        "low"
    }
}

fn audit_url_for(group_by: IdentityGroupBy, identity_id: &str, policy: Option<&str>) -> String {
    let param = match group_by {
        IdentityGroupBy::User => "user_id",
        IdentityGroupBy::Agent => "agent_id",
        IdentityGroupBy::AgentScope => "agent_scope",
    };
    let mut url = format!(
        "/admin/governance/decisions?{param}={}",
        urlencode(identity_id)
    );
    if let Some(p) = policy.filter(|s| !s.is_empty()) {
        url.push_str(&format!("&policy={}", urlencode(p)));
    }
    url
}

fn profile_url_for(group_by: IdentityGroupBy, identity_id: &str) -> String {
    format!(
        "/admin/governance/identity/{}?group_by={}",
        urlencode(identity_id),
        group_by.as_str()
    )
}

fn time_range_context(
    query: &IdentityIndexQuery,
    range: &crate::repositories::governance_grp::time_range::TimeRange,
) -> serde_json::Value {
    let preset = query.preset.clone().unwrap_or_else(|| {
        if query.from.is_some() && query.to.is_some() {
            "custom".to_string()
        } else {
            "24h".to_string()
        }
    });
    let mut q_suffix = String::new();
    if let Some(g) = query.group_by.as_deref().filter(|s| !s.is_empty()) {
        q_suffix.push_str(&format!("&group_by={}", urlencode(g)));
    }
    json!({
        "preset": preset,
        "from": range.from.to_rfc3339(),
        "to": range.to.to_rfc3339(),
        "base_url": BASE_URL,
        "query": q_suffix,
    })
}
