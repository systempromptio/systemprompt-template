use std::collections::HashSet;
use std::sync::Arc;

use systemprompt::identifiers::SkillId;

use crate::repositories;
use crate::repositories::plugin_maps::EntityPluginMap;
use crate::templates::AdminTemplateEngine;
use crate::types::{AgentSkill, MarketplaceContext, UserContext, SOURCE_CUSTOM, SOURCE_USER};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

fn count_entities_via_plugins(
    entity_id: &str,
    entity_plugin_map: &EntityPluginMap,
    target_plugin_map: &EntityPluginMap,
) -> usize {
    entity_plugin_map.get(entity_id).map_or(0, |plugins| {
        let mut ids: HashSet<String> = HashSet::new();
        for entry in plugins {
            let plugin_id = &entry.0;
            for (target_id, target_plugins) in target_plugin_map {
                if target_plugins.iter().any(|e| &e.0 == plugin_id) {
                    ids.insert(target_id.clone());
                }
            }
        }
        ids.len()
    })
}

struct SkillFilters {
    sources: HashSet<String>,
    plugins: HashSet<String>,
    tags: HashSet<String>,
}

struct SkillViewContext<'a> {
    skill_plugin_map: &'a EntityPluginMap,
    agent_plugin_map: &'a EntityPluginMap,
    usage_counts: &'a std::collections::HashMap<String, i64>,
    avg_ratings: &'a std::collections::HashMap<String, (f64, i64)>,
}

fn build_skill_json(
    skill: &AgentSkill,
    ctx: &SkillViewContext<'_>,
    filters: &mut SkillFilters,
) -> serde_json::Value {
    let skill_id_str = skill.skill_id.as_str();
    let assigned_plugins: Vec<serde_json::Value> = ctx
        .skill_plugin_map
        .get(skill_id_str)
        .map(|plugins| {
            plugins
                .iter()
                .map(|entry| json!({"id": entry.0, "name": entry.1}))
                .collect()
        })
        .unwrap_or_default();

    let agent_count =
        count_entities_via_plugins(skill_id_str, ctx.skill_plugin_map, ctx.agent_plugin_map);

    let source = if skill.source_id.as_str() == SOURCE_CUSTOM || skill.source_id.as_str() == SOURCE_USER {
        "custom"
    } else {
        "system"
    };

    filters.sources.insert(source.to_string());
    for p in &assigned_plugins {
        if let Some(name) = p.get("name").and_then(|v| v.as_str()) {
            filters.plugins.insert(name.to_string());
        }
    }
    if let Some(tags) = &skill.tags {
        for tag in tags {
            filters.tags.insert(tag.clone());
        }
    }

    let usage_count = ctx.usage_counts.get(skill_id_str).copied().unwrap_or(0);
    let (avg_rating, rating_count) = ctx
        .avg_ratings
        .get(skill_id_str)
        .copied()
        .unwrap_or((0.0, 0));

    json!({
        "skill_id": skill_id_str,
        "name": skill.name,
        "description": skill.description,
        "tags": skill.tags,
        "category_id": skill.category_id,
        "source": source,
        "assigned_plugins": assigned_plugins,
        "assigned_plugin_ids": assigned_plugins.iter().filter_map(|p| p.get("id").and_then(|v| v.as_str())).collect::<Vec<_>>(),
        "plugin_count": assigned_plugins.len(),
        "agent_count": agent_count,
        "created_at": skill.created_at.to_rfc3339(),
        "updated_at": skill.updated_at.to_rfc3339(),
        "usage_count": usage_count,
        "avg_rating": avg_rating,
        "rating_count": rating_count,
    })
}

pub async fn skills_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let skills = fetch_visible_skills(&pool, &services_path, &user_ctx).await;

    let (skill_plugin_map, agent_plugin_map, _mcp_plugin_map) =
        repositories::build_entity_plugin_maps(&services_path);

    let plugin_list = fetch_plugin_list(&services_path);
    let (usage_counts, avg_ratings) = fetch_skill_metrics(&pool, &skills).await;

    let skill_view_ctx = SkillViewContext {
        skill_plugin_map: &skill_plugin_map,
        agent_plugin_map: &agent_plugin_map,
        usage_counts: &usage_counts,
        avg_ratings: &avg_ratings,
    };

    let result = build_skills_with_filters(&skills, &skill_view_ctx);

    let data = json!({
        "page": "skills",
        "title": "Org Skills",
        "skills": result.skills_data,
        "all_plugins": plugin_list,
        "filter_sources": result.sources,
        "filter_plugins": result.plugins,
        "filter_tags": result.tags,
    });
    super::render_page(&engine, "skills", &data, &user_ctx, &mkt_ctx)
}

async fn fetch_visible_skills(
    pool: &PgPool,
    services_path: &std::path::Path,
    user_ctx: &UserContext,
) -> Vec<AgentSkill> {
    let all_skills = repositories::list_agent_skills(pool)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list agent skills");
            vec![]
        });

    if user_ctx.is_admin {
        return all_skills;
    }

    let plugins = repositories::list_plugins_for_roles(services_path, &user_ctx.roles)
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list plugins for roles");
            vec![]
        });
    let visible_skill_ids: HashSet<SkillId> = plugins
        .iter()
        .flat_map(|p| p.skills.iter().map(|s| s.id.clone()))
        .collect();
    all_skills
        .into_iter()
        .filter(|s| visible_skill_ids.contains(&s.skill_id))
        .collect()
}

fn fetch_plugin_list(services_path: &std::path::Path) -> Vec<serde_json::Value> {
    let all_plugins = repositories::list_plugins_for_roles(services_path, &["admin".to_string()])
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list all plugins");
            vec![]
        });
    all_plugins
        .iter()
        .map(|p| json!({"id": p.id, "name": p.name}))
        .collect()
}

async fn fetch_skill_metrics(
    pool: &PgPool,
    skills: &[AgentSkill],
) -> (
    std::collections::HashMap<String, i64>,
    std::collections::HashMap<String, (f64, i64)>,
) {
    let skill_ids: Vec<SkillId> = skills.iter().map(|s| s.skill_id.clone()).collect();
    let usage_counts = repositories::fetch_skill_usage_counts(pool, &skill_ids).await;
    let avg_ratings = repositories::fetch_skill_avg_ratings(pool, &skill_ids).await;
    (usage_counts, avg_ratings)
}

struct SkillsWithFilters {
    skills_data: Vec<serde_json::Value>,
    sources: Vec<String>,
    plugins: Vec<String>,
    tags: Vec<String>,
}

fn build_skills_with_filters(
    skills: &[AgentSkill],
    ctx: &SkillViewContext<'_>,
) -> SkillsWithFilters {
    let mut filters = SkillFilters {
        sources: HashSet::new(),
        plugins: HashSet::new(),
        tags: HashSet::new(),
    };

    let skills_data: Vec<serde_json::Value> = skills
        .iter()
        .map(|skill| build_skill_json(skill, ctx, &mut filters))
        .collect();

    let mut sorted_sources: Vec<String> = filters.sources.into_iter().collect();
    sorted_sources.sort();
    let mut sorted_plugins: Vec<String> = filters.plugins.into_iter().collect();
    sorted_plugins.sort();
    let mut sorted_tags: Vec<String> = filters.tags.into_iter().collect();
    sorted_tags.sort();

    SkillsWithFilters {
        skills_data,
        sources: sorted_sources,
        plugins: sorted_plugins,
        tags: sorted_tags,
    }
}

pub async fn skill_edit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let skill_id = params.get("id");
    let is_edit = skill_id.is_some();
    let skill = if let Some(id) = skill_id {
        repositories::find_agent_skill(&pool, &SkillId::new(id))
            .await
            .map_err(|e| {
                tracing::warn!(error = %e, skill_id = %id, "Failed to fetch skill");
            })
            .ok()
            .flatten()
    } else {
        None
    };

    let mut skill_json = serde_json::to_value(&skill).unwrap_or(json!(null));
    if let Some(obj) = skill_json.as_object_mut() {
        if let Some(tags) = obj.get("tags").and_then(|t| t.as_array()) {
            let csv: String = tags
                .iter()
                .filter_map(|t| t.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            obj.insert("tags_csv".to_string(), json!(csv));
        }
    }

    let data = json!({
        "page": "skill-edit",
        "title": if is_edit { "Edit Skill" } else { "Create Skill" },
        "is_edit": is_edit,
        "skill": skill_json,
    });
    super::render_page(&engine, "skill-edit", &data, &user_ctx, &mkt_ctx)
}
