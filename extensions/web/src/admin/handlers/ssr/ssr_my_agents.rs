use std::collections::HashMap;
use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::repositories::conversation_analytics;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::conversation_analytics::{EntityLastUsed, EntityQualityTrend};
use crate::admin::types::{EntityEffectiveness, MarketplaceContext, UserAgent, UserContext};

const PROMPT_PREVIEW_LEN: usize = 200;
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

pub async fn my_agents_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let (agents, effectiveness, last_used, trends, plugin_assignments) = tokio::join!(
        async {
            repositories::list_user_agents(&pool, &user_ctx.user_id)
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!(error = %e, "Failed to list user agents");
                    vec![]
                })
        },
        async {
            conversation_analytics::fetch_entity_effectiveness(&pool, &user_ctx.user_id, "agent")
                .await
                .unwrap_or_else(|_| vec![])
        },
        async {
            conversation_analytics::fetch_entity_last_used(&pool, &user_ctx.user_id)
                .await
                .unwrap_or_else(|_| vec![])
        },
        async {
            conversation_analytics::fetch_entity_quality_trend(&pool, &user_ctx.user_id, "agent")
                .await
                .unwrap_or_else(|_| vec![])
        },
        async {
            repositories::fetch_agent_plugin_assignments(&pool, &user_ctx.user_id)
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!(error = %e, "Failed to fetch plugin assignments for agents");
                    HashMap::new()
                })
        },
    );

    let agents_json = build_agents_json(
        &agents,
        &effectiveness,
        &last_used,
        &trends,
        &plugin_assignments,
    );
    let agent_count = agents.len();

    let data = json!({
        "page": "my-agents",
        "title": "My Agents",
        "agents": agents_json,
        "stats": {
            "agent_count": agent_count,
        },
    });
    super::render_page(&engine, "my-agents", &data, &user_ctx, &mkt_ctx)
}

fn build_agents_json(
    agents: &[UserAgent],
    effectiveness: &[EntityEffectiveness],
    last_used: &[EntityLastUsed],
    trends: &[EntityQualityTrend],
    plugin_assignments: &HashMap<String, Vec<String>>,
) -> Vec<serde_json::Value> {
    let effectiveness_map: HashMap<&str, _> = effectiveness
        .iter()
        .map(|e| (e.entity_name.as_str(), e))
        .collect();
    let last_used_map: HashMap<&str, _> = last_used
        .iter()
        .filter(|lu| lu.entity_type == "agent")
        .map(|lu| (lu.entity_name.as_str(), lu))
        .collect();
    let trend_map: HashMap<&str, _> = trends.iter().map(|t| (t.entity_name.as_str(), t)).collect();

    agents
        .iter()
        .map(|a| {
            build_single_agent_json(
                a,
                &effectiveness_map,
                &last_used_map,
                &trend_map,
                plugin_assignments,
            )
        })
        .collect()
}

fn build_single_agent_json(
    a: &UserAgent,
    effectiveness_map: &HashMap<&str, &EntityEffectiveness>,
    last_used_map: &HashMap<&str, &EntityLastUsed>,
    trend_map: &HashMap<&str, &EntityQualityTrend>,
    plugin_assignments: &HashMap<String, Vec<String>>,
) -> serde_json::Value {
    let eff = effectiveness_map.get(a.name.as_str());
    let lu = last_used_map.get(a.name.as_str());
    let trend = trend_map.get(a.name.as_str());
    let prompt_preview = if a.system_prompt.len() > PROMPT_PREVIEW_LEN {
        format!("{}...", &a.system_prompt[..PROMPT_PREVIEW_LEN])
    } else {
        a.system_prompt.clone()
    };
    let is_forked = a.base_agent_id.is_some();
    let plugins = plugin_assignments
        .get(a.agent_id.as_str())
        .cloned()
        .unwrap_or_else(Vec::new);
    let trend_direction = trend.map_or("flat", |t| {
        if t.recent_avg > t.previous_avg + 0.2 {
            "up"
        } else if t.recent_avg < t.previous_avg - 0.2 {
            "down"
        } else {
            "flat"
        }
    });
    let mut v = serde_json::to_value(a).unwrap_or(json!({}));
    if let Some(obj) = v.as_object_mut() {
        obj.insert(
            "total_uses".to_string(),
            json!(eff.map_or(0, |e| e.total_uses)),
        );
        obj.insert(
            "sessions_used_in".to_string(),
            json!(eff.map_or(0, |e| e.sessions_used_in)),
        );
        obj.insert(
            "avg_effectiveness".to_string(),
            json!(format!("{:.1}", eff.map_or(0.0, |e| e.avg_effectiveness))),
        );
        obj.insert(
            "scored_sessions".to_string(),
            json!(eff.map_or(0, |e| e.scored_sessions)),
        );
        obj.insert(
            "goal_achievement_pct".to_string(),
            json!(format!(
                "{:.0}",
                eff.map_or(0.0, |e| e.goal_achievement_pct)
            )),
        );
        obj.insert(
            "last_used".to_string(),
            lu.map(|l| json!(l.last_used.to_rfc3339()))
                .unwrap_or(json!(null)),
        );
        obj.insert("trend_direction".to_string(), json!(trend_direction));
        obj.insert("prompt_preview".to_string(), json!(prompt_preview));
        obj.insert("is_forked".to_string(), json!(is_forked));
        obj.insert("plugin_names".to_string(), json!(plugins));
    }
    v
}

pub async fn my_agent_edit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let agent_id = params.get("id");
    let is_edit = agent_id.is_some();

    let agent = if let Some(id) = agent_id {
        let agents = repositories::list_user_agents(&pool, &user_ctx.user_id)
            .await
            .unwrap_or_else(|_| vec![]);
        agents
            .into_iter()
            .find(|a| a.agent_id.as_str() == id.as_str())
    } else {
        None
    };

    let is_forked = agent
        .as_ref()
        .and_then(|a| a.base_agent_id.as_ref())
        .is_some();

    let agent_json = match &agent {
        Some(a) => serde_json::to_value(a).unwrap_or(json!({})),
        None => json!({
            "agent_id": "",
            "name": "",
            "description": "",
            "system_prompt": "",
        }),
    };

    let data = json!({
        "page": "my-agent-edit",
        "title": if is_edit { "Edit My Agent" } else { "Create My Agent" },
        "is_edit": is_edit,
        "is_forked": is_forked,
        "agent": agent_json,
    });
    super::render_page(&engine, "my-agent-edit", &data, &user_ctx, &mkt_ctx)
}
