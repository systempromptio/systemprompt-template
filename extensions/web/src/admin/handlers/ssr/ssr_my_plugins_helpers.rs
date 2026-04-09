use crate::admin::numeric;
use crate::admin::repositories;
use crate::admin::types::conversation_analytics::{
    EntityEffectiveness, EntityUsageSummary, SkillEffectiveness,
};
use crate::admin::types::UserContext;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};

use super::types::{CheckableEntity, NamedEntity, PluginEditData, PluginView, SkillWithStats};

pub(super) fn collect_my_plugins(
    enriched: &[repositories::user_plugins::UserPluginEnriched],
    skill_usage_map: &HashMap<&str, &EntityUsageSummary>,
    skill_eff_map: &HashMap<&str, &SkillEffectiveness>,
    agent_eff_map: &HashMap<&str, &EntityEffectiveness>,
) -> (Vec<serde_json::Value>, Vec<String>) {
    let mut categories_set: HashSet<String> = HashSet::new();
    let mut plugins_json: Vec<serde_json::Value> = Vec::new();

    for ep in enriched {
        let p = &ep.plugin;
        if !p.category.is_empty() {
            categories_set.insert(p.category.clone());
        }
        let view = enriched_plugin_to_view(ep, skill_usage_map, skill_eff_map, agent_eff_map);
        if let Ok(v) = serde_json::to_value(&view) {
            plugins_json.push(v);
        }
    }

    let mut categories: Vec<String> = categories_set.into_iter().collect();
    categories.sort();
    (plugins_json, categories)
}

fn enriched_plugin_to_view(
    ep: &repositories::user_plugins::UserPluginEnriched,
    skill_usage_map: &HashMap<&str, &EntityUsageSummary>,
    skill_eff_map: &HashMap<&str, &SkillEffectiveness>,
    agent_eff_map: &HashMap<&str, &EntityEffectiveness>,
) -> PluginView {
    let p = &ep.plugin;
    let mut plugin_total_uses: i64 = 0;
    let mut plugin_session_count: i64 = 0;
    let mut weighted_quality_sum: f64 = 0.0;
    let mut weighted_goal_sum: f64 = 0.0;
    let mut quality_weight_total: i64 = 0;
    let mut total_scored_sessions: i64 = 0;

    let skills: Vec<SkillWithStats> = ep
        .skills
        .iter()
        .map(|s| {
            let usage = skill_usage_map.get(s.id.as_str());
            let uses = usage.map_or(0, |u| u.total_uses);
            plugin_total_uses += uses;
            plugin_session_count += usage.map_or(0, |u| u.session_count);

            let eff = skill_eff_map.get(s.name.as_str());
            let avg_effectiveness = eff.map_or(0.0, |e| e.avg_effectiveness);
            let goal_pct = eff.map_or(0.0, |e| e.goal_achievement_pct);
            let scored = eff.map_or(0, |e| e.scored_sessions);
            if scored > 0 {
                weighted_quality_sum += avg_effectiveness * numeric::to_f64(scored);
                weighted_goal_sum += goal_pct * numeric::to_f64(scored);
                quality_weight_total += scored;
                total_scored_sessions += scored;
            }

            SkillWithStats {
                id: s.id.clone(),
                name: s.name.clone(),
                uses,
                avg_effectiveness: format!("{avg_effectiveness:.1}"),
                goal_pct: format!("{goal_pct:.0}"),
                scored_sessions: scored,
            }
        })
        .collect();

    let agents: Vec<NamedEntity> = ep
        .agents
        .iter()
        .map(|a| {
            let eff = agent_eff_map.get(a.name.as_str());
            let scored = eff.map_or(0, |e| e.scored_sessions);
            if scored > 0 {
                weighted_quality_sum +=
                    eff.map_or(0.0, |e| e.avg_effectiveness) * numeric::to_f64(scored);
                weighted_goal_sum +=
                    eff.map_or(0.0, |e| e.goal_achievement_pct) * numeric::to_f64(scored);
                quality_weight_total += scored;
                total_scored_sessions += scored;
            }
            NamedEntity::from(a)
        })
        .collect();

    let mcp_servers: Vec<NamedEntity> = ep.mcp_servers.iter().map(NamedEntity::from).collect();

    let plugin_avg_quality = if quality_weight_total > 0 {
        weighted_quality_sum / numeric::to_f64(quality_weight_total)
    } else {
        0.0
    };
    let plugin_goal_pct = if quality_weight_total > 0 {
        weighted_goal_sum / numeric::to_f64(quality_weight_total)
    } else {
        0.0
    };

    PluginView {
        plugin_id: p.plugin_id.clone(),
        name: p.name.clone(),
        description: p.description.clone(),
        category: p.category.clone(),
        version: p.version.clone(),
        base_plugin_id: p.base_plugin_id.clone(),
        author_name: p.author_name.clone(),
        skill_count: ep.skill_count,
        agent_count: ep.agent_count,
        mcp_count: ep.mcp_count,
        total_uses: plugin_total_uses,
        session_count: plugin_session_count,
        avg_quality_score: format!("{plugin_avg_quality:.1}"),
        goal_achievement_pct: format!("{plugin_goal_pct:.0}"),
        scored_sessions: total_scored_sessions,
        skills,
        agents,
        mcp_servers,
    }
}

pub(super) async fn build_association_lists(
    pool: &PgPool,
    user_ctx: &UserContext,
    plugin_with_assoc: Option<&crate::admin::types::UserPluginWithAssociations>,
) -> (
    Vec<CheckableEntity>,
    Vec<CheckableEntity>,
    Vec<CheckableEntity>,
) {
    let user_skills = repositories::list_user_skills(pool, &user_ctx.user_id)
        .await
        .unwrap_or_else(|_| vec![]);
    let user_agents = repositories::list_user_agents(pool, &user_ctx.user_id)
        .await
        .unwrap_or_else(|_| vec![]);
    let user_mcp_servers =
        repositories::user_mcp_servers::list_user_mcp_servers(pool, &user_ctx.user_id)
            .await
            .unwrap_or_else(|_| vec![]);

    let selected_skills: Vec<String> = plugin_with_assoc.map_or(vec![], |p| {
        p.skill_ids
            .iter()
            .map(ToString::to_string)
            .collect()
    });
    let selected_agents: Vec<String> = plugin_with_assoc.map_or(vec![], |p| {
        p.agent_ids
            .iter()
            .map(ToString::to_string)
            .collect()
    });
    let selected_mcp: Vec<String> = plugin_with_assoc.map_or(vec![], |p| {
        p.mcp_server_ids
            .iter()
            .map(ToString::to_string)
            .collect()
    });

    let skills_list: Vec<CheckableEntity> = user_skills
        .iter()
        .map(|s| CheckableEntity {
            value: s.id.clone(),
            name: s.name.clone(),
            checked: selected_skills.contains(&s.id),
        })
        .collect();
    let agents_list: Vec<CheckableEntity> = user_agents
        .iter()
        .map(|a| CheckableEntity {
            value: a.id.clone(),
            name: a.name.clone(),
            checked: selected_agents.contains(&a.id),
        })
        .collect();
    let mcp_list: Vec<CheckableEntity> = user_mcp_servers
        .iter()
        .map(|m| CheckableEntity {
            value: m.id.clone(),
            name: m.name.clone(),
            checked: selected_mcp.contains(&m.id),
        })
        .collect();

    (skills_list, agents_list, mcp_list)
}

pub(super) fn build_plugin_edit_data(
    plugin_with_assoc: Option<&crate::admin::types::UserPluginWithAssociations>,
) -> PluginEditData {
    match plugin_with_assoc {
        Some(p) => PluginEditData {
            id: Some(p.plugin.id.clone()),
            plugin_id: p.plugin.plugin_id.clone(),
            name: p.plugin.name.clone(),
            description: p.plugin.description.clone(),
            version: p.plugin.version.clone(),
            enabled: p.plugin.enabled,
            category: p.plugin.category.clone(),
            keywords: p.plugin.keywords.clone(),
            author_name: p.plugin.author_name.clone(),
            base_plugin_id: p.plugin.base_plugin_id.clone(),
        },
        None => PluginEditData {
            id: None,
            plugin_id: String::new(),
            name: String::new(),
            description: String::new(),
            version: "1.0.0".to_string(),
            enabled: true,
            category: String::new(),
            keywords: vec![],
            author_name: String::new(),
            base_plugin_id: None,
        },
    }
}
