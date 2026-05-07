//! `/admin/governance` — Policies dashboard.
//!
//! Lists every policy registered via `inventory::submit!` together with its
//! enabled state, per-policy params from `services/governance/config.yaml`,
//! the source file the impl lives in, and 24h enforcement counts pulled from
//! `governance_decisions`. The page is the front door to the modular policy
//! framework — operators land here to see (a) what policies exist as code,
//! (b) what config they're running with, and (c) what they're actually doing
//! at runtime.

use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use serde_yaml::Value as YamlValue;
use sqlx::PgPool;

use crate::handlers::webhook::governance;
use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

const WINDOW_24H_SECS: i64 = 86_400;
const TOP_POLICIES_LIMIT: i64 = 10;
const TOP_ACTORS_LIMIT: i64 = 10;

pub async fn governance_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let (lifetime, window, per_policy_lifetime, per_policy_window, top_tools, top_actors) = tokio::join!(
        repositories::governance::fetch_governance_counts(&pool),
        repositories::governance::fetch_governance_counts_windowed(&pool, WINDOW_24H_SECS),
        repositories::governance::fetch_per_policy_counts(&pool),
        repositories::governance::fetch_per_policy_counts_windowed(&pool, WINDOW_24H_SECS),
        repositories::governance::fetch_top_policies(&pool, WINDOW_24H_SECS, TOP_POLICIES_LIMIT),
        repositories::governance::fetch_top_actors(&pool, WINDOW_24H_SECS, TOP_ACTORS_LIMIT),
    );

    let lifetime = lifetime.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "governance lifetime counts query failed");
        repositories::governance::GovernanceCounts::default()
    });
    let window = window.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "governance windowed counts query failed");
        repositories::governance::GovernanceCounts::default()
    });
    let per_policy_lifetime_rows = per_policy_lifetime.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "per-policy lifetime counts query failed");
        Vec::new()
    });
    let per_policy_window_rows = per_policy_window.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "per-policy windowed counts query failed");
        Vec::new()
    });
    let top_tools = top_tools.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "top denying tools query failed");
        Vec::new()
    });
    let top_actors = top_actors.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "top denied actors query failed");
        Vec::new()
    });

    let mut lifetime_by_id: HashMap<String, repositories::governance::PerPolicyCounts> =
        per_policy_lifetime_rows
            .into_iter()
            .map(|r| (r.policy.clone(), r))
            .collect();
    let mut window_by_id: HashMap<String, repositories::governance::PerPolicyCounts> =
        per_policy_window_rows
            .into_iter()
            .map(|r| (r.policy.clone(), r))
            .collect();

    let policies_json = build_policies_json(&mut lifetime_by_id, &mut window_by_id);
    let orphan_json = build_orphans_json(&lifetime_by_id);
    let (enforcement_json, any_enforcement_activity) = build_enforcement_json(&policies_json);
    let top_tools_json = build_top_tools_json(&top_tools);
    let top_actors_json = build_top_actors_json(&top_actors);

    let data = json!({
        "page": "governance",
        "title": "Governance Policies",
        "lifetime_total": lifetime.total,
        "lifetime_allowed": lifetime.allowed,
        "lifetime_denied": lifetime.denied,
        "window_total": window.total,
        "window_allowed": window.allowed,
        "window_denied": window.denied,
        "window_breaches": window.secret_breaches,
        "policies": policies_json,
        "has_policies": !policies_json.is_empty(),
        "enforcement": enforcement_json,
        "has_enforcement_activity": any_enforcement_activity,
        "top_tools": top_tools_json,
        "has_top_tools": !top_tools_json.is_empty(),
        "top_actors": top_actors_json,
        "has_top_actors": !top_actors_json.is_empty(),
        "orphans": orphan_json,
        "has_orphans": !orphan_json.is_empty(),
        "orphans_count": orphan_json.len(),
        "config_path": "services/governance/config.yaml",
    });

    super::render_page(&engine, "governance", &data, &user_ctx, &mkt_ctx)
}

fn build_policies_json(
    lifetime_by_id: &mut HashMap<String, repositories::governance::PerPolicyCounts>,
    window_by_id: &mut HashMap<String, repositories::governance::PerPolicyCounts>,
) -> Vec<serde_json::Value> {
    let chain = governance::chain();
    chain
        .iter()
        .map(|(cfg, p)| build_policy_row(cfg, p, lifetime_by_id, window_by_id))
        .collect()
}

fn build_policy_row(
    cfg: &governance::policy::PolicyConfig,
    p: &dyn governance::policy::Policy,
    lifetime_by_id: &mut HashMap<String, repositories::governance::PerPolicyCounts>,
    window_by_id: &mut HashMap<String, repositories::governance::PerPolicyCounts>,
) -> serde_json::Value {
    let life = lifetime_by_id.remove(p.id());
    let win = window_by_id.remove(p.id());
    let lifetime_allowed = life.as_ref().map_or(0, |s| s.allowed);
    let lifetime_denied = life.as_ref().map_or(0, |s| s.denied);
    let window_allowed = win.as_ref().map_or(0, |s| s.allowed);
    let window_denied = win.as_ref().map_or(0, |s| s.denied);
    let window_evals = window_allowed + window_denied;
    let last_at = life
        .as_ref()
        .and_then(|s| s.last_at)
        .map(format_local)
        .unwrap_or_default();
    let last_at_window = win
        .as_ref()
        .and_then(|s| s.last_at)
        .map(format_local)
        .unwrap_or_default();
    let params_preview = render_params_preview(&cfg.params);
    let deny_rate = format_deny_rate(window_denied, window_evals);
    json!({
        "id": p.id(),
        "name": p.name(),
        "description": p.description(),
        "enabled": cfg.enabled,
        "source_path": governance::policy::source_path_for(p.id()),
        "params_preview": params_preview,
        "has_params": !params_preview.is_empty(),
        "lifetime_allowed": lifetime_allowed,
        "lifetime_denied": lifetime_denied,
        "window_allowed": window_allowed,
        "window_denied": window_denied,
        "window_evaluations": window_evals,
        "deny_rate": deny_rate,
        "has_recent_denies": window_denied > 0,
        "last_at": last_at,
        "last_at_window": last_at_window,
        "edit_url": format!("/admin/governance/{}", p.id()),
        "decisions_url": format!("/admin/governance/decisions?policy={}", p.id()),
        "deny_decisions_url": format!(
            "/admin/governance/decisions?policy={}&outcome=deny",
            p.id()
        ),
    })
}

fn format_deny_rate(denied: i64, evaluations: i64) -> String {
    if evaluations <= 0 {
        return "—".to_string();
    }
    #[allow(clippy::cast_precision_loss)]
    let r = (denied as f64 / evaluations as f64) * 100.0;
    format!("{r:.1}%")
}

/// Anything left in `lifetime_by_id` is a policy that produced decisions in
/// the past but is no longer registered (renamed / removed). Surface it so
/// operators don't lose sight of it.
fn build_orphans_json(
    lifetime_by_id: &HashMap<String, repositories::governance::PerPolicyCounts>,
) -> Vec<serde_json::Value> {
    lifetime_by_id
        .values()
        .map(|s| {
            json!({
                "id": s.policy,
                "allowed": s.allowed,
                "denied": s.denied,
                "last_at": s.last_at.map(format_local).unwrap_or_default(),
            })
        })
        .collect()
}

/// Per-policy enforcement table (24h). Reshape `policies_json` sorted by
/// denied DESC so operators see the busiest deniers first.
fn build_enforcement_json(
    policies_json: &[serde_json::Value],
) -> (Vec<serde_json::Value>, bool) {
    let mut rows: Vec<serde_json::Value> = policies_json.to_vec();
    rows.sort_by(|a, b| {
        let bd = a_i64(b, "window_denied");
        let ad = a_i64(a, "window_denied");
        bd.cmp(&ad).then_with(|| {
            let be = a_i64(b, "window_evaluations");
            let ae = a_i64(a, "window_evaluations");
            be.cmp(&ae)
        })
    });
    let any = rows.iter().any(|r| a_i64(r, "window_evaluations") > 0);
    (rows, any)
}

fn build_top_tools_json(top_tools: &[crate::types::TopPolicy]) -> Vec<serde_json::Value> {
    top_tools
        .iter()
        .map(|t| {
            json!({
                "policy": t.policy,
                "tool_name": t.tool_name,
                "hits": t.hits,
                "distinct_actors": t.distinct_actors,
                "decisions_url": format!(
                    "/admin/governance/decisions?policy={}&outcome=deny",
                    t.policy
                ),
            })
        })
        .collect()
}

fn build_top_actors_json(top_actors: &[crate::types::TopActor]) -> Vec<serde_json::Value> {
    top_actors
        .iter()
        .map(|a| {
            json!({
                "user_id": a.user_id,
                "display_name": a.display_name,
                "email": a.email,
                "deny_count": a.deny_count,
                "secret_count": a.secret_count,
                "total": a.total,
                "decisions_url": format!(
                    "/admin/governance/decisions?user_id={}&outcome=deny",
                    a.user_id
                ),
            })
        })
        .collect()
}

fn a_i64(v: &serde_json::Value, key: &str) -> i64 {
    v.get(key)
        .and_then(serde_json::Value::as_i64)
        .unwrap_or(0)
}

fn format_local(t: chrono::DateTime<chrono::Utc>) -> String {
    t.with_timezone(&chrono::Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

/// Renders the config block for a single policy as a compact list of
/// `key: value` strings. Skips the policy's own `id` and `enabled` fields
/// (already shown in the card chrome). Scalars render verbatim; sequences as
/// comma-joined; nested maps as JSON one-liners.
fn render_params_preview(params: &YamlValue) -> Vec<serde_json::Value> {
    let YamlValue::Mapping(map) = params else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for (k, v) in map {
        let Some(key) = k.as_str() else { continue };
        if matches!(key, "id" | "enabled") {
            continue;
        }
        let value_str = match v {
            YamlValue::Null => "null".to_string(),
            YamlValue::Bool(b) => b.to_string(),
            YamlValue::Number(n) => n.to_string(),
            YamlValue::String(s) => s.clone(),
            YamlValue::Sequence(seq) => seq
                .iter()
                .map(yaml_inline)
                .collect::<Vec<_>>()
                .join(", "),
            YamlValue::Mapping(_) | YamlValue::Tagged(_) => yaml_inline(v),
        };
        out.push(json!({ "key": key, "value": value_str }));
    }
    out
}

fn yaml_inline(v: &YamlValue) -> String {
    match v {
        YamlValue::Null => "null".to_string(),
        YamlValue::Bool(b) => b.to_string(),
        YamlValue::Number(n) => n.to_string(),
        YamlValue::String(s) => s.clone(),
        other => serde_json::to_string(other).unwrap_or_else(|_| "<?>".to_string()),
    }
}
