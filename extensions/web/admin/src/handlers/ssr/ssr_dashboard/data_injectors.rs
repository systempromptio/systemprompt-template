use crate::repositories;
use crate::repositories::analytics_grp::cost_stats::fetch_cost_kpis;
use crate::repositories::external_agents_grp::list_external_agents;
use crate::repositories::governance_grp::time_range::{TimeRange, TimeRangePreset};
use crate::types::{
    IncidentGroup, TopActor, TopPolicy, WindowedCounts, ACTION_GRANTED, DECISION_DENY,
};
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;

const WINDOW_5M_SECS: i64 = 5 * 60;
const WINDOW_1H_SECS: i64 = 60 * 60;
const WINDOW_24H_SECS: i64 = 24 * 60 * 60;
const BASELINE_LOOKBACK_DAYS: i64 = 14;
const TOP_LIMIT: i64 = 10;
const INCIDENT_LIMIT: i64 = 20;

pub(super) async fn inject_mcp_access_and_costs(pool: &PgPool, data: &mut serde_json::Value) {
    let range_30d = TimeRange {
        from: Utc::now() - chrono::Duration::days(30),
        to: Utc::now(),
        preset: TimeRangePreset::Days30,
    };
    let (mcp_access, token_usage, cost_kpis) = tokio::join!(
        repositories::dashboard_queries::fetch_mcp_access_events(pool),
        repositories::dashboard_queries::fetch_token_usage_by_user(pool),
        fetch_cost_kpis(pool, range_30d),
    );

    let mcp_events = mcp_access.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch MCP access events for dashboard");
        vec![]
    });
    let mcp_json: Vec<serde_json::Value> = mcp_events
        .iter()
        .map(|r| {
            json!({
                "server_name": r.server_name,
                "action": r.action,
                "is_granted": r.action == ACTION_GRANTED,
                "description": r.description,
                "created_at": r.created_at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string(),
            })
        })
        .collect();

    let tokens = token_usage.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch token usage for dashboard");
        vec![]
    });
    let max_tokens: i64 = tokens
        .iter()
        .map(|t| t.input_tokens + t.output_tokens)
        .max()
        .unwrap_or(1)
        .max(1);
    let tokens_json: Vec<serde_json::Value> = tokens
        .iter()
        .map(|r| {
            let total = r.input_tokens + r.output_tokens;
            let pct = total.saturating_mul(100) / max_tokens;
            json!({
                "label": r.label,
                "input_tokens": r.input_tokens,
                "output_tokens": r.output_tokens,
                "total_tokens": total,
                "event_count": r.event_count,
                "pct": pct,
            })
        })
        .collect();

    let cost_json = cost_kpis.map_or_else(
        |e| {
            tracing::warn!(error = %e, "Failed to fetch cost KPIs for dashboard");
            serde_json::Value::Null
        },
        |k| {
            let total_usd = k.total_cost_microdollars as f64 / 1_000_000.0;
            json!({
                "total_cost_usd": format!("{total_usd:.4}"),
                "requests": k.requests,
                "input_tokens": k.input_tokens,
                "output_tokens": k.output_tokens,
                "total_tokens": k.total_tokens,
                "tokens_per_min": format!("{:.1}", k.tokens_per_minute),
            })
        },
    );

    if let Some(obj) = data.as_object_mut() {
        obj.insert("mcp_access_events".to_string(), json!(mcp_json));
        obj.insert(
            "has_mcp_access_events".to_string(),
            json!(!mcp_json.is_empty()),
        );
        obj.insert("token_usage".to_string(), json!(tokens_json));
        obj.insert(
            "has_token_usage".to_string(),
            json!(!tokens_json.is_empty()),
        );
        obj.insert("has_cost_kpis".to_string(), json!(!cost_json.is_null()));
        obj.insert("cost_kpis".to_string(), cost_json);
    }
}

#[derive(Debug, sqlx::FromRow)]
struct AgentCallRow {
    agent_id: String,
    calls: i64,
}

#[derive(Debug, sqlx::FromRow)]
struct McpCallRow {
    server_name: String,
    calls: i64,
}

pub(super) async fn inject_services_data(
    pool: &PgPool,
    services_path: Option<&std::path::PathBuf>,
    data: &mut serde_json::Value,
) {
    let Some(path) = services_path else {
        return;
    };

    let agents = repositories::list_agents(path).unwrap_or_default();
    let mcp_servers =
        repositories::mcp_grp::mcp_servers::list_mcp_servers(path).unwrap_or_default();
    let external = list_external_agents();

    let (agent_calls_res, mcp_calls_res) = tokio::join!(
        sqlx::query_as::<_, AgentCallRow>(
            r"SELECT COALESCE(metadata->>'agent_id', plugin_id, '') AS agent_id,
                     COUNT(*)::BIGINT AS calls
              FROM ai_requests
              WHERE created_at > NOW() - INTERVAL '24 hours'
                AND (metadata->>'agent_id' IS NOT NULL OR plugin_id IS NOT NULL)
              GROUP BY COALESCE(metadata->>'agent_id', plugin_id, '')"
        )
        .fetch_all(pool),
        sqlx::query_as::<_, McpCallRow>(
            r"SELECT COALESCE(entity_name, 'unknown') AS server_name,
                     COUNT(*)::BIGINT AS calls
              FROM user_activity
              WHERE category = 'mcp_access'
                AND created_at > NOW() - INTERVAL '24 hours'
              GROUP BY entity_name"
        )
        .fetch_all(pool),
    );

    let agent_call_map: std::collections::HashMap<String, i64> = agent_calls_res
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch agent call counts for services table");
            vec![]
        })
        .into_iter()
        .map(|r| (r.agent_id, r.calls))
        .collect();

    let mcp_call_map: std::collections::HashMap<String, i64> = mcp_calls_res
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to fetch MCP call counts for services table");
            vec![]
        })
        .into_iter()
        .map(|r| (r.server_name, r.calls))
        .collect();

    let mut services: Vec<serde_json::Value> = Vec::new();

    for agent in &agents {
        let calls = agent_call_map.get(agent.id.as_str()).copied().unwrap_or(0);
        services.push(json!({
            "name": agent.name,
            "service_type": "Agent",
            "enabled": agent.enabled,
            "calls_24h": calls,
        }));
    }

    for server in &mcp_servers {
        let calls = mcp_call_map.get(server.id.as_str()).copied().unwrap_or(0);
        services.push(json!({
            "name": server.id.as_str(),
            "service_type": "MCP",
            "enabled": server.enabled,
            "calls_24h": calls,
        }));
    }

    for ext in &external {
        services.push(json!({
            "name": ext.display_name,
            "service_type": "External",
            "enabled": ext.enabled,
            "calls_24h": 0i64,
        }));
    }

    services.sort_by(|a, b| {
        let a_en = a["enabled"].as_bool().unwrap_or(false);
        let b_en = b["enabled"].as_bool().unwrap_or(false);
        b_en.cmp(&a_en).then_with(|| {
            a["name"]
                .as_str()
                .unwrap_or("")
                .cmp(b["name"].as_str().unwrap_or(""))
        })
    });

    let has_services = !services.is_empty();
    if let Some(obj) = data.as_object_mut() {
        obj.insert("services".to_string(), json!(services));
        obj.insert("has_services".to_string(), json!(has_services));
    }
}

#[derive(Default, Clone)]
struct WindowSlice {
    current: WindowedCounts,
    baseline: Vec<WindowedCounts>,
}

#[derive(Default)]
struct GovernanceFetch {
    events: Vec<crate::types::GovernanceEvent>,
    counts: repositories::governance::GovernanceCounts,
    short: WindowSlice,
    medium: WindowSlice,
    long: WindowSlice,
    top_actors: Vec<TopActor>,
    top_policies: Vec<TopPolicy>,
    incidents: Vec<IncidentGroup>,
}

fn log_or_default<T: Default>(r: Result<T, sqlx::Error>, what: &str) -> T {
    r.unwrap_or_else(|e| {
        tracing::warn!(error = %e, what, "Governance fetch failed");
        T::default()
    })
}

async fn fetch_all(pool: &PgPool) -> GovernanceFetch {
    let (
        events_r,
        counts_r,
        cur_short_r,
        cur_med_r,
        cur_long_r,
        base_short_r,
        base_med_r,
        base_long_r,
        actors_r,
        policies_r,
        incidents_r,
    ) = tokio::join!(
        repositories::governance::fetch_governance_events(pool),
        repositories::governance::fetch_governance_counts(pool),
        repositories::governance::fetch_windowed_counts(pool, WINDOW_5M_SECS),
        repositories::governance::fetch_windowed_counts(pool, WINDOW_1H_SECS),
        repositories::governance::fetch_windowed_counts(pool, WINDOW_24H_SECS),
        repositories::governance::fetch_baseline_window_samples(
            pool,
            WINDOW_5M_SECS,
            BASELINE_LOOKBACK_DAYS
        ),
        repositories::governance::fetch_baseline_window_samples(
            pool,
            WINDOW_1H_SECS,
            BASELINE_LOOKBACK_DAYS
        ),
        repositories::governance::fetch_baseline_window_samples(
            pool,
            WINDOW_24H_SECS,
            BASELINE_LOOKBACK_DAYS
        ),
        repositories::governance::fetch_top_actors(pool, WINDOW_24H_SECS, TOP_LIMIT),
        repositories::governance::fetch_top_policies(pool, WINDOW_24H_SECS, TOP_LIMIT),
        repositories::governance::fetch_grouped_incidents(pool, WINDOW_24H_SECS, INCIDENT_LIMIT),
    );
    GovernanceFetch {
        events: log_or_default(events_r, "governance events"),
        counts: log_or_default(counts_r, "governance counts"),
        short: WindowSlice {
            current: log_or_default(cur_short_r, "windowed counts short"),
            baseline: log_or_default(base_short_r, "baseline short"),
        },
        medium: WindowSlice {
            current: log_or_default(cur_med_r, "windowed counts medium"),
            baseline: log_or_default(base_med_r, "baseline medium"),
        },
        long: WindowSlice {
            current: log_or_default(cur_long_r, "windowed counts long"),
            baseline: log_or_default(base_long_r, "baseline long"),
        },
        top_actors: log_or_default(actors_r, "top actors"),
        top_policies: log_or_default(policies_r, "top policies"),
        incidents: log_or_default(incidents_r, "grouped incidents"),
    }
}

fn mean_stddev(samples: &[i64]) -> (f64, f64) {
    if samples.is_empty() {
        return (0.0, 0.0);
    }
    let n = samples.len() as f64;
    let total: i64 = samples.iter().sum();
    let mean = total as f64 / n;
    let variance = samples
        .iter()
        .map(|v| {
            let d = *v as f64 - mean;
            d * d
        })
        .sum::<f64>()
        / n;
    (mean, variance.sqrt())
}

fn sigma(current: i64, mean: f64, stddev: f64) -> f64 {
    let cur = current as f64;
    if stddev < f64::EPSILON {
        if cur > mean {
            99.0
        } else {
            0.0
        }
    } else {
        (cur - mean) / stddev
    }
}

fn denied_sigma_for(slice: &WindowSlice) -> f64 {
    let samples: Vec<i64> = slice.baseline.iter().map(|b| b.denied).collect();
    let (mean, sd) = mean_stddev(&samples);
    sigma(slice.current.denied, mean, sd)
}

fn build_window_json(key: &str, slice: &WindowSlice) -> serde_json::Value {
    let denied_samples: Vec<i64> = slice.baseline.iter().map(|b| b.denied).collect();
    let secret_samples: Vec<i64> = slice.baseline.iter().map(|b| b.secret_blocks).collect();
    let decisions_samples: Vec<i64> = slice.baseline.iter().map(|b| b.decisions).collect();

    let (denied_mean, denied_sd) = mean_stddev(&denied_samples);
    let (secret_mean, secret_sd) = mean_stddev(&secret_samples);
    let (decisions_mean, _) = mean_stddev(&decisions_samples);

    let cur = slice.current;
    let denied_z = sigma(cur.denied, denied_mean, denied_sd);
    let secret_z = sigma(cur.secret_blocks, secret_mean, secret_sd);

    let denied_pct = if cur.decisions > 0 {
        let p = (cur.denied as f64) * 100.0 / (cur.decisions as f64);
        p.round() as i64
    } else {
        0
    };

    json!({
        "key": key,
        "decisions": cur.decisions,
        "denied": cur.denied,
        "denied_pct": denied_pct,
        "secret_blocks": cur.secret_blocks,
        "distinct_actors": cur.distinct_actors,
        "decisions_baseline": decisions_mean.round() as i64,
        "denied_sigma": format!("{denied_z:+.1}"),
        "secret_sigma": format!("{secret_z:+.1}"),
    })
}

fn classify_posture(short: &WindowSlice, medium: &WindowSlice) -> serde_json::Value {
    let z_short = denied_sigma_for(short);
    let z_med = denied_sigma_for(medium);
    let cur = short.current;

    let red = cur.secret_blocks > 0 || z_short >= 3.0 || z_med >= 3.0;
    let amber = z_short >= 1.5 || z_med >= 1.5;

    let level = if red {
        "red"
    } else if amber {
        "amber"
    } else {
        "green"
    };
    let label = match level {
        "red" => "RED",
        "amber" => "AMBER",
        _ => "GREEN",
    };
    let headline = if cur.secret_blocks > 0 {
        format!(
            "{} secret-scan block(s) in last 5 min — triage immediately",
            cur.secret_blocks
        )
    } else if z_short >= 3.0 {
        format!("Denial rate {z_short:+.1}σ vs 14-day baseline (last 5 min)")
    } else if z_med >= 1.5 {
        format!("Elevated denials {z_med:+.1}σ vs baseline (last 1 h)")
    } else {
        format!(
            "{} decisions / {} denied in last 1 h — within baseline",
            medium.current.decisions, medium.current.denied
        )
    };
    json!({
        "level": level,
        "label": label,
        "headline": headline,
        "secret_5m": cur.secret_blocks,
        "denied_5m": cur.denied,
        "decisions_5m": cur.decisions,
    })
}

fn build_action_queue(
    medium: WindowedCounts,
    long: WindowedCounts,
    top_actors: &[TopActor],
    top_policies: &[TopPolicy],
) -> Vec<serde_json::Value> {
    let mut q = Vec::new();
    if medium.secret_blocks > 0 {
        q.push(json!({
            "severity": "danger",
            "kind": "secret_scan",
            "label": format!("{} secret-scan block(s) in last 1 h — triage", medium.secret_blocks),
            "link": "/admin/governance",
        }));
    }
    if let Some(actor) = top_actors.first() {
        if actor.deny_count >= 10 {
            q.push(json!({
                "severity": "warning",
                "kind": "actor_spike",
                "label": format!("{} accumulated {} denials in 24 h", actor.display_name, actor.deny_count),
                "link": format!("/admin/user?id={}", actor.user_id),
            }));
        }
    }
    if let Some(policy) = top_policies.first() {
        if policy.hits >= 20 {
            q.push(json!({
                "severity": "warning",
                "kind": "policy_hot",
                "label": format!("{} / {} firing {}× — review threshold", policy.policy, policy.tool_name, policy.hits),
                "link": "/admin/governance",
            }));
        }
    }
    if long.distinct_actors == 0 && long.decisions == 0 {
        q.push(json!({
            "severity": "info",
            "kind": "quiet",
            "label": "No governance activity in 24 h — verify pipeline is wired".to_string(),
            "link": "/admin/audit",
        }));
    }
    q
}

fn local_ts(t: chrono::DateTime<Utc>) -> String {
    t.with_timezone(&chrono::Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

fn build_events_json(events: &[crate::types::GovernanceEvent]) -> Vec<serde_json::Value> {
    events
        .iter()
        .map(|r| {
            json!({
                "user_id": r.user_id,
                "tool_name": r.tool_name,
                "agent_id": r.agent_id,
                "decision": r.decision,
                "is_denied": r.decision == DECISION_DENY,
                "reason": r.reason,
                "created_at": local_ts(r.created_at),
            })
        })
        .collect()
}

fn build_actors_json(actors: &[TopActor]) -> Vec<serde_json::Value> {
    actors
        .iter()
        .map(|a| {
            json!({
                "user_id": a.user_id,
                "display_name": a.display_name,
                "email": a.email,
                "deny_count": a.deny_count,
                "secret_count": a.secret_count,
                "total": a.total,
                "is_secret_offender": a.secret_count > 0,
            })
        })
        .collect()
}

fn build_policies_json(policies: &[TopPolicy]) -> Vec<serde_json::Value> {
    policies
        .iter()
        .map(|p| {
            json!({
                "policy": p.policy,
                "tool_name": p.tool_name,
                "hits": p.hits,
                "distinct_actors": p.distinct_actors,
                "is_secret_policy": p.policy == "secret_scan",
            })
        })
        .collect()
}

fn build_incidents_json(incidents: &[IncidentGroup]) -> Vec<serde_json::Value> {
    incidents
        .iter()
        .map(|i| {
            let label = i.display_name.clone().unwrap_or_else(|| i.user_id.clone());
            json!({
                "agent_id": i.agent_id,
                "user_id": i.user_id,
                "actor_label": label,
                "policy": i.policy,
                "tool_name": i.tool_name,
                "attempts": i.attempts,
                "first_seen": local_ts(i.first_seen),
                "last_seen": local_ts(i.last_seen),
                "sample_reason": i.sample_reason,
                "is_secret_policy": i.policy == "secret_scan",
            })
        })
        .collect()
}

pub(super) async fn inject_governance_data(pool: &PgPool, data: &mut serde_json::Value) {
    let g = fetch_all(pool).await;

    let events_json = build_events_json(&g.events);
    let actors_json = build_actors_json(&g.top_actors);
    let policies_json = build_policies_json(&g.top_policies);
    let incidents_json = build_incidents_json(&g.incidents);

    let posture = classify_posture(&g.short, &g.medium);
    let action_queue = build_action_queue(
        g.medium.current,
        g.long.current,
        &g.top_actors,
        &g.top_policies,
    );

    let win_short = build_window_json("5m", &g.short);
    let win_medium = build_window_json("1h", &g.medium);
    let win_long = build_window_json("24h", &g.long);

    let Some(obj) = data.as_object_mut() else {
        return;
    };
    obj.insert("governance_total".to_string(), json!(g.counts.total));
    obj.insert("governance_allowed".to_string(), json!(g.counts.allowed));
    obj.insert("governance_denied".to_string(), json!(g.counts.denied));
    obj.insert(
        "governance_secret_breaches".to_string(),
        json!(g.counts.secret_breaches),
    );
    obj.insert("governance_events".to_string(), json!(events_json));
    obj.insert(
        "has_governance_events".to_string(),
        json!(!events_json.is_empty()),
    );
    obj.insert("governance_posture".to_string(), posture);
    obj.insert("governance_window_5m".to_string(), win_short);
    obj.insert("governance_window_1h".to_string(), win_medium);
    obj.insert("governance_window_24h".to_string(), win_long);
    obj.insert("governance_top_actors".to_string(), json!(actors_json));
    obj.insert(
        "has_governance_top_actors".to_string(),
        json!(!actors_json.is_empty()),
    );
    obj.insert("governance_top_policies".to_string(), json!(policies_json));
    obj.insert(
        "has_governance_top_policies".to_string(),
        json!(!policies_json.is_empty()),
    );
    obj.insert("governance_incidents".to_string(), json!(incidents_json));
    obj.insert(
        "has_governance_incidents".to_string(),
        json!(!incidents_json.is_empty()),
    );
    obj.insert("governance_action_queue".to_string(), json!(action_queue));
    obj.insert(
        "has_governance_action_queue".to_string(),
        json!(!action_queue.is_empty()),
    );
}
