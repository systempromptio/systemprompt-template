//! View-model assembly for the Governance Policies page.
//!
//! Joins the registered policy chain (from `inventory`) against the lifetime /
//! windowed decision counts to build per-policy cards, the deny-sorted
//! enforcement table, top-deniers / top-actors leaderboards, and the orphan
//! list (policies that produced decisions but are no longer registered). Also
//! renders each policy's `services/governance/config.yaml` params block.

use std::collections::HashMap;

use serde_yaml::Value as YamlValue;

use crate::handlers::webhook::governance;
use crate::repositories;

use super::context::{OrphanRow, ParamPreview, PolicyRow, TopActorRow, TopToolRow};

pub(super) fn build_policies_json(
    lifetime_by_id: &mut HashMap<String, repositories::governance::PerPolicyCounts>,
    window_by_id: &mut HashMap<String, repositories::governance::PerPolicyCounts>,
) -> Vec<PolicyRow> {
    let chain = governance::chain();
    chain
        .iter()
        .map(|(cfg, p)| build_policy_row(cfg, p, lifetime_by_id, window_by_id))
        .collect()
}

fn build_policy_row(
    cfg: &governance::policy::PolicyConfig,
    p: &dyn governance::policy::GovernancePolicy,
    lifetime_by_id: &mut HashMap<String, repositories::governance::PerPolicyCounts>,
    window_by_id: &mut HashMap<String, repositories::governance::PerPolicyCounts>,
) -> PolicyRow {
    let id = p.id();
    let id_str = id.as_str();
    let life = lifetime_by_id.remove(id_str);
    let win = window_by_id.remove(id_str);
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
    let has_params = !params_preview.is_empty();
    PolicyRow {
        id: id_str.to_owned(),
        name: p.name().to_owned(),
        description: p.description().to_owned(),
        enabled: cfg.enabled,
        source_path: governance::policy::source_path_for(id_str).to_owned(),
        params_preview,
        has_params,
        lifetime_allowed,
        lifetime_denied,
        window_allowed,
        window_denied,
        window_evaluations: window_evals,
        deny_rate,
        has_recent_denies: window_denied > 0,
        last_at,
        last_at_window,
        edit_url: format!("/admin/governance/{id_str}"),
        decisions_url: format!("/admin/governance/decisions?policy={id_str}"),
        deny_decisions_url: format!("/admin/governance/decisions?policy={id_str}&outcome=deny"),
    }
}

fn format_deny_rate(denied: i64, evaluations: i64) -> String {
    if evaluations <= 0 {
        return "—".to_owned();
    }
    let r = (denied as f64 / evaluations as f64) * 100.0;
    format!("{r:.1}%")
}

/// Anything left in `lifetime_by_id` is a policy that produced decisions in
/// the past but is no longer registered (renamed / removed). Surface it so
/// operators don't lose sight of it.
pub(super) fn build_orphans_json(
    lifetime_by_id: &HashMap<String, repositories::governance::PerPolicyCounts>,
) -> Vec<OrphanRow> {
    lifetime_by_id
        .values()
        .map(|s| OrphanRow {
            id: s.policy.clone(),
            allowed: s.allowed,
            denied: s.denied,
            last_at: s.last_at.map(format_local).unwrap_or_default(),
        })
        .collect()
}

/// Per-policy enforcement table (24h). Reshape `policies` sorted by
/// denied DESC so operators see the busiest deniers first.
pub(super) fn build_enforcement_json(policies: &[PolicyRow]) -> (Vec<PolicyRow>, bool) {
    let mut rows: Vec<PolicyRow> = policies.to_vec();
    rows.sort_by(|a, b| {
        b.window_denied
            .cmp(&a.window_denied)
            .then_with(|| b.window_evaluations.cmp(&a.window_evaluations))
    });
    let any = rows.iter().any(|r| r.window_evaluations > 0);
    (rows, any)
}

pub(super) fn build_top_tools_json(top_tools: &[crate::types::TopPolicy]) -> Vec<TopToolRow> {
    top_tools
        .iter()
        .map(|t| TopToolRow {
            policy: t.policy.clone(),
            tool_name: t.tool_name.clone(),
            hits: t.hits,
            distinct_actors: t.distinct_actors,
            decisions_url: format!(
                "/admin/governance/decisions?policy={}&outcome=deny",
                t.policy
            ),
        })
        .collect()
}

pub(super) fn build_top_actors_json(top_actors: &[crate::types::TopActor]) -> Vec<TopActorRow> {
    top_actors
        .iter()
        .map(|a| TopActorRow {
            user_id: a.user_id.clone(),
            display_name: a.display_name.clone(),
            email: a.email.clone(),
            deny_count: a.deny_count,
            secret_count: a.secret_count,
            total: a.total,
            decisions_url: format!(
                "/admin/governance/decisions?user_id={}&outcome=deny",
                a.user_id
            ),
        })
        .collect()
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
fn render_params_preview(params: &YamlValue) -> Vec<ParamPreview> {
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
            YamlValue::Null => "null".to_owned(),
            YamlValue::Bool(b) => b.to_string(),
            YamlValue::Number(n) => n.to_string(),
            YamlValue::String(s) => s.clone(),
            YamlValue::Sequence(seq) => seq.iter().map(yaml_inline).collect::<Vec<_>>().join(", "),
            YamlValue::Mapping(_) | YamlValue::Tagged(_) => yaml_inline(v),
        };
        out.push(ParamPreview {
            key: key.to_owned(),
            value: value_str,
        });
    }
    out
}

fn yaml_inline(v: &YamlValue) -> String {
    match v {
        YamlValue::Null => "null".to_owned(),
        YamlValue::Bool(b) => b.to_string(),
        YamlValue::Number(n) => n.to_string(),
        YamlValue::String(s) => s.clone(),
        other => serde_json::to_string(other).unwrap_or_else(|_| "<?>".to_owned()),
    }
}
