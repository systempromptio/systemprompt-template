//! View-model assembly for the Governance Policies page.
//!
//! Joins the engine's policy set against the lifetime / windowed decision
//! counts to build the chain table, the KPI band, the top-deniers and
//! top-actors leaderboards, and the orphan list (policies that produced
//! decisions but are no longer registered). Also flattens each policy's
//! `services/governance/config.yaml` params into one line.

use std::collections::HashMap;

use serde_yaml::Value as YamlValue;

use crate::handlers::ssr::format::local_time;
use crate::handlers::webhook::governance;
use crate::repositories::governance::{GovernanceCounts, PerPolicyCounts};

use super::context::{GovernanceKpiView, OrphanRow, PolicyRow, TopActorRow, TopToolRow};

const DECISIONS_URL: &str = "/admin/governance/decisions";

pub(super) fn build_kpis(
    window: &GovernanceCounts,
    lifetime: &GovernanceCounts,
) -> Vec<GovernanceKpiView> {
    vec![
        GovernanceKpiView {
            label: "Decisions · 24h",
            value: window.total.to_string(),
            note: "every tool call the chain evaluated".to_owned(),
            tone: "accent",
            href: DECISIONS_URL.to_owned(),
        },
        GovernanceKpiView {
            label: "Allowed · 24h",
            value: window.allowed.to_string(),
            note: "passed every enabled stage".to_owned(),
            tone: "ok",
            href: format!("{DECISIONS_URL}?outcome=allow"),
        },
        GovernanceKpiView {
            label: "Denied · 24h",
            value: window.denied.to_string(),
            note: "stopped at the first failing stage".to_owned(),
            tone: if window.denied > 0 { "err" } else { "ok" },
            href: format!("{DECISIONS_URL}?outcome=deny"),
        },
        GovernanceKpiView {
            label: "Secret breaches · 24h",
            value: window.secret_breaches.to_string(),
            note: "credentials caught before leaving".to_owned(),
            tone: if window.secret_breaches > 0 {
                "err"
            } else {
                "ok"
            },
            href: format!("{DECISIONS_URL}?policy=secret_scan&outcome=deny"),
        },
        GovernanceKpiView {
            label: "Decisions · all time",
            value: lifetime.total.to_string(),
            note: format!("{} denied", lifetime.denied),
            tone: "accent",
            href: DECISIONS_URL.to_owned(),
        },
    ]
}

pub(super) fn build_policies(
    lifetime_by_id: &mut HashMap<String, PerPolicyCounts>,
    window_by_id: &mut HashMap<String, PerPolicyCounts>,
) -> Result<Vec<PolicyRow>, systemprompt_security::policy::GovernanceEngineError> {
    let engine = governance::engine()?;
    Ok(engine
        .policies()
        .enumerate()
        .map(|(idx, (cfg, p))| build_policy_row(idx + 1, cfg, p, lifetime_by_id, window_by_id))
        .collect())
}

fn build_policy_row(
    order: usize,
    cfg: &systemprompt_security::policy::PolicyConfig,
    p: &dyn systemprompt_security::policy::GovernancePolicy,
    lifetime_by_id: &mut HashMap<String, PerPolicyCounts>,
    window_by_id: &mut HashMap<String, PerPolicyCounts>,
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
        .map(local_time)
        .unwrap_or_default();
    let params_line = render_params_line(&cfg.params);
    PolicyRow {
        order,
        id: id_str.to_owned(),
        name: p.name().to_owned(),
        description: p.description().to_owned(),
        enabled: cfg.enabled,
        state: if cfg.enabled { "Enabled" } else { "Disabled" },
        state_tone: if cfg.enabled { "ok" } else { "muted" },
        has_params: !params_line.is_empty(),
        params_line,
        lifetime_allowed,
        lifetime_denied,
        window_allowed,
        window_denied,
        window_evaluations: window_evals,
        deny_rate: format_deny_rate(window_denied, window_evals),
        has_recent_denies: window_denied > 0,
        last_at,
        edit_url: format!("/admin/governance/policies/{id_str}"),
        decisions_url: format!("{DECISIONS_URL}?policy={id_str}"),
        deny_decisions_url: format!("{DECISIONS_URL}?policy={id_str}&outcome=deny"),
    }
}

fn format_deny_rate(denied: i64, evaluations: i64) -> String {
    if evaluations <= 0 {
        return "—".to_owned();
    }
    let r = (denied as f64 / evaluations as f64) * 100.0;
    format!("{r:.1}%")
}

// Why: anything left in `lifetime_by_id` is a policy that has decisions on
// record under an id the registry does not carry — typically a rename or a
// removal. Surface it so operators don't lose sight of it.
pub(super) fn build_orphans(lifetime_by_id: &HashMap<String, PerPolicyCounts>) -> Vec<OrphanRow> {
    let mut rows: Vec<OrphanRow> = lifetime_by_id
        .values()
        .map(|s| OrphanRow {
            id: s.policy.clone(),
            allowed: s.allowed,
            denied: s.denied,
            last_at: s.last_at.map(local_time).unwrap_or_default(),
        })
        .collect();
    rows.sort_by(|a, b| a.id.cmp(&b.id));
    rows
}

pub(super) fn build_top_tools(top_tools: &[crate::types::TopPolicy]) -> Vec<TopToolRow> {
    top_tools
        .iter()
        .map(|t| TopToolRow {
            policy: t.policy.clone(),
            tool_name: t.tool_name.clone(),
            hits: t.hits,
            distinct_actors: t.distinct_actors,
            decisions_url: format!("{DECISIONS_URL}?policy={}&outcome=deny", t.policy),
        })
        .collect()
}

pub(super) fn build_top_actors(top_actors: &[crate::types::TopActor]) -> Vec<TopActorRow> {
    top_actors
        .iter()
        .map(|a| TopActorRow {
            user_id: a.user_id.clone(),
            display_name: a.display_name.clone(),
            email: a.email.clone().unwrap_or_default(),
            deny_count: a.deny_count,
            secret_count: a.secret_count,
            total: a.total,
            decisions_url: format!("{DECISIONS_URL}?user_id={}&outcome=deny", a.user_id),
            user_url: format!("/admin/user?id={}", urlencoding::encode(a.user_id.as_str())),
        })
        .collect()
}

// Why: the parameters are read as one clipped line in a table cell, so they
// are joined here rather than rendered as a list the row would wrap around.
fn render_params_line(params: &YamlValue) -> String {
    let YamlValue::Mapping(map) = params else {
        return String::new();
    };
    map.iter()
        .filter_map(|(k, v)| {
            let key = k.as_str()?;
            if matches!(key, "id" | "enabled") {
                return None;
            }
            Some(format!("{key}={}", yaml_inline(v)))
        })
        .collect::<Vec<_>>()
        .join(" · ")
}

fn yaml_inline(v: &YamlValue) -> String {
    match v {
        YamlValue::Null => "null".to_owned(),
        YamlValue::Bool(b) => b.to_string(),
        YamlValue::Number(n) => n.to_string(),
        YamlValue::String(s) => s.clone(),
        YamlValue::Sequence(seq) => seq.iter().map(yaml_inline).collect::<Vec<_>>().join(", "),
        other => serde_json::to_string(other).unwrap_or_else(|_| "<?>".to_owned()),
    }
}
