//! `/admin/governance/identity/:id` — single-identity profile.
//!
//! Risk-score breakdown + decision timeline + top-fired rules + sessions.
//! All decision data is fetched via `fetch_decisions_paged` with the identity
//! filter pre-applied, so the timeline matches the audit trail one-to-one.

use std::sync::Arc;

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use urlencoding::encode as urlencode;

use crate::repositories::governance_grp::paged::{
    fetch_decisions_paged, DecisionFilter, DecisionRow, SortColumn, SortDir, SortSpec,
};
use crate::repositories::governance_grp::risk_score::{
    compute_risk_score, fetch_violation_counts, weights, IdentityGroupBy,
};
use crate::repositories::governance_grp::time_range::{parse_time_range, TimeRangeQuery};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

const PAGE_LIMIT: i64 = 100;

#[derive(Debug, Deserialize)]
pub struct ProfileQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
    pub group_by: Option<String>,
}

pub async fn governance_identity_profile_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(identity_id): Path<String>,
    Query(query): Query<ProfileQuery>,
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
    let weights = weights();

    let filter = filter_for_identity(group_by, &identity_id);
    let sort = SortSpec {
        column: SortColumn::CreatedAt,
        dir: SortDir::Desc,
    };

    let (rows_res, counts_res) = tokio::join!(
        fetch_decisions_paged(&pool, &filter, range, sort, PAGE_LIMIT, 0),
        fetch_violation_counts(&pool, range, group_by),
    );
    let (rows, total) = rows_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_decisions_paged (profile) failed");
        (Vec::new(), 0)
    });
    let counts = counts_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_violation_counts (profile) failed");
        Vec::new()
    });

    let identity_counts = counts
        .iter()
        .find(|(id, _)| id == &identity_id)
        .map(|(_, v)| *v)
        .unwrap_or_default();
    let score = compute_risk_score(&identity_counts, weights);

    let timeline: Vec<serde_json::Value> = rows.iter().map(decision_to_json).collect();
    let top_rules = build_top_rules(&rows);
    let sessions = build_sessions(&rows);

    let data = json!({
        "page": "governance-identity",
        "title": format!("Identity — {identity_id}"),
        "identity_id": identity_id,
        "group_by": group_by.as_str(),
        "group_label": group_label(group_by),
        "audit_url": audit_url_for(group_by, &identity_id),
        "back_url": format!(
            "/admin/governance/identity?group_by={}",
            urlencode(group_by.as_str())
        ),
        "time_range": time_range_context(&query, &range, &identity_id, group_by),
        "risk": {
            "score": format!("{:.1}", score.score),
            "band": risk_band(score.score),
            "raw": format!("{:.2}", score.raw),
            "normalised": format!("{:.4}", score.normalised),
            "deny_count": identity_counts.deny_count,
            "secret_breach_count": identity_counts.secret_breach_count,
            "scope_violation_count": identity_counts.scope_violation_count,
            "activity_volume": identity_counts.activity_volume,
            "weights": {
                "deny_weight": weights.deny_weight,
                "secret_breach_weight": weights.secret_breach_weight,
                "scope_violation_weight": weights.scope_violation_weight,
                "scale": weights.scale,
                "normalization_floor": weights.normalization_floor,
            },
        },
        "timeline": timeline,
        "has_timeline": !rows.is_empty(),
        "total_count": total,
        "top_rules": top_rules,
        "sessions": sessions,
    });

    super::render_page(
        &engine,
        "governance-identity-profile",
        &data,
        &user_ctx,
        &mkt_ctx,
    )
}

fn filter_for_identity(group_by: IdentityGroupBy, identity_id: &str) -> DecisionFilter {
    let mut f = DecisionFilter::default();
    let v = identity_id.to_string();
    match group_by {
        IdentityGroupBy::User => f.user_id = Some(v),
        IdentityGroupBy::Agent => f.agent_id = Some(v),
        IdentityGroupBy::AgentScope => f.agent_scope = Some(v),
    }
    f
}

const fn group_label(g: IdentityGroupBy) -> &'static str {
    match g {
        IdentityGroupBy::User => "User",
        IdentityGroupBy::Agent => "Agent",
        IdentityGroupBy::AgentScope => "Scope",
    }
}

fn audit_url_for(group_by: IdentityGroupBy, identity_id: &str) -> String {
    let param = match group_by {
        IdentityGroupBy::User => "user_id",
        IdentityGroupBy::Agent => "agent_id",
        IdentityGroupBy::AgentScope => "agent_scope",
    };
    format!(
        "/admin/governance/decisions?{param}={}",
        urlencode(identity_id)
    )
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

fn decision_to_json(r: &DecisionRow) -> serde_json::Value {
    json!({
        "id": r.id,
        "trace_id": r.trace_id,
        "session_id": r.session_id,
        "tool_name": r.tool_name,
        "policy": r.policy,
        "decision": r.decision,
        "is_denied": r.decision == "deny",
        "reason": r.reason,
        "created_at": r.created_at.to_rfc3339(),
        "created_at_local": r
            .created_at
            .with_timezone(&chrono::Local)
            .format("%Y-%m-%d %H:%M:%S")
            .to_string(),
    })
}

fn build_top_rules(rows: &[DecisionRow]) -> Vec<serde_json::Value> {
    let mut counts: std::collections::HashMap<String, (i64, i64)> =
        std::collections::HashMap::new();
    for r in rows {
        let entry = counts.entry(r.policy.clone()).or_insert((0, 0));
        entry.0 += 1;
        if r.decision == "deny" {
            entry.1 += 1;
        }
    }
    let mut sorted: Vec<(String, i64, i64)> = counts
        .into_iter()
        .map(|(p, (total, deny))| (p, total, deny))
        .collect();
    sorted.sort_by(|a, b| b.2.cmp(&a.2).then(b.1.cmp(&a.1)));
    sorted
        .into_iter()
        .take(10)
        .map(|(policy, total, deny)| {
            json!({
                "policy": policy,
                "total": total,
                "deny": deny,
            })
        })
        .collect()
}

fn build_sessions(rows: &[DecisionRow]) -> Vec<serde_json::Value> {
    let mut latest: std::collections::HashMap<String, (chrono::DateTime<chrono::Utc>, i64, i64)> =
        std::collections::HashMap::new();
    for r in rows {
        let entry = latest.entry(r.session_id.clone()).or_insert((r.created_at, 0, 0));
        if r.created_at > entry.0 {
            entry.0 = r.created_at;
        }
        entry.1 += 1;
        if r.decision == "deny" {
            entry.2 += 1;
        }
    }
    let mut out: Vec<(String, chrono::DateTime<chrono::Utc>, i64, i64)> = latest
        .into_iter()
        .map(|(id, (last, total, deny))| (id, last, total, deny))
        .collect();
    out.sort_by_key(|row| std::cmp::Reverse(row.1));
    out.into_iter()
        .take(20)
        .map(|(session_id, last, total, deny)| {
            json!({
                "session_id": session_id,
                "last_at": last
                    .with_timezone(&chrono::Local)
                    .format("%Y-%m-%d %H:%M:%S")
                    .to_string(),
                "total": total,
                "deny": deny,
                "audit_url": format!("/admin/governance/decisions?q={}", urlencode(&session_id)),
            })
        })
        .collect()
}

fn time_range_context(
    query: &ProfileQuery,
    range: &crate::repositories::governance_grp::time_range::TimeRange,
    identity_id: &str,
    group_by: IdentityGroupBy,
) -> serde_json::Value {
    let preset = query.preset.clone().unwrap_or_else(|| {
        if query.from.is_some() && query.to.is_some() {
            "custom".to_string()
        } else {
            "24h".to_string()
        }
    });
    json!({
        "preset": preset,
        "from": range.from.to_rfc3339(),
        "to": range.to.to_rfc3339(),
        "base_url": format!("/admin/governance/identity/{}", urlencode(identity_id)),
        "query": format!("&group_by={}", urlencode(group_by.as_str())),
    })
}
