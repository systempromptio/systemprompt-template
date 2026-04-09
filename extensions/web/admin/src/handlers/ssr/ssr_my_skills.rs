use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use systemprompt::identifiers::SkillId;

use crate::repositories;
use crate::repositories::conversation_analytics;
use crate::templates::AdminTemplateEngine;
use crate::types::conversation_analytics::{SkillEffectiveness, SkillRating};
use crate::types::{MarketplaceContext, UserContext};

const CONTENT_PREVIEW_LEN: usize = 200;
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use sqlx::PgPool;

use super::types::{
    MySkillEditPageData, MySkillsPageData, NamedEntity, RequiredSecretView, SkillEditView,
    SkillStats, SkillViewExtra,
};

pub async fn my_skills_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let skills = repositories::list_user_skills(&pool, &user_ctx.user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list user skills");
            vec![]
        });

    let skill_ids: Vec<SkillId> = skills.iter().map(|s| s.skill_id.clone()).collect();

    let (usage_counts, user_plugins, effectiveness, skill_ratings) = tokio::join!(
        repositories::fetch_skill_usage_counts(&pool, &skill_ids),
        async {
            repositories::list_user_plugins_enriched(&pool, &user_ctx.user_id)
                .await
                .unwrap_or_else(|_| vec![])
        },
        async {
            conversation_analytics::fetch_skill_effectiveness(&pool, &user_ctx.user_id)
                .await
                .unwrap_or_else(|_| vec![])
        },
        async {
            conversation_analytics::fetch_all_skill_ratings(&pool, &user_ctx.user_id)
                .await
                .unwrap_or_else(|_| vec![])
        },
    );

    let skill_plugin_map = build_skill_plugin_map(&user_plugins);
    let all_tags = collect_sorted_tags(&skills);
    let skill_count = skills.len();
    let skills_json = build_skills_json(
        &skills,
        &usage_counts,
        &skill_plugin_map,
        &effectiveness,
        &skill_ratings,
    );

    let data = MySkillsPageData {
        page: "my-skills",
        title: "My Skills",
        skills: skills_json,
        all_tags,
        stats: SkillStats { skill_count },
    };
    let data_value =
        serde_json::to_value(&data).unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    super::render_page(&engine, "my-skills", &data_value, &user_ctx, &mkt_ctx)
}

fn build_skill_plugin_map(
    user_plugins: &[repositories::user_plugins::UserPluginEnriched],
) -> HashMap<String, Vec<NamedEntity>> {
    let mut map: HashMap<String, Vec<NamedEntity>> = HashMap::new();
    for ep in user_plugins {
        for s in &ep.skills {
            map.entry(s.id.clone()).or_default().push(NamedEntity {
                id: ep.plugin.plugin_id.clone(),
                name: ep.plugin.name.clone(),
            });
        }
    }
    map
}

fn collect_sorted_tags(skills: &[crate::types::UserSkill]) -> Vec<String> {
    let mut tags_set = HashSet::new();
    for skill in skills {
        for tag in &skill.tags {
            if !tag.is_empty() {
                tags_set.insert(tag.clone());
            }
        }
    }
    let mut all_tags: Vec<String> = tags_set.into_iter().collect();
    all_tags.sort();
    all_tags
}

fn build_skills_json(
    skills: &[crate::types::UserSkill],
    usage_counts: &HashMap<String, i64>,
    skill_plugin_map: &HashMap<String, Vec<NamedEntity>>,
    effectiveness: &[SkillEffectiveness],
    skill_ratings: &[SkillRating],
) -> Vec<serde_json::Value> {
    let eff_map: HashMap<&str, &SkillEffectiveness> = effectiveness
        .iter()
        .map(|e| (e.skill_id.as_str(), e))
        .collect();
    let rating_map: HashMap<&str, &SkillRating> = skill_ratings
        .iter()
        .map(|r| {
            let key = r
                .skill_name
                .rsplit_once(':')
                .map_or(r.skill_name.as_str(), |(_, slug)| slug);
            (key, r)
        })
        .collect();

    skills
        .iter()
        .map(|s| {
            let usage = usage_counts.get(s.skill_id.as_str()).copied().unwrap_or(0);
            let content_preview = if s.content.len() > CONTENT_PREVIEW_LEN {
                format!("{}...", &s.content[..CONTENT_PREVIEW_LEN])
            } else {
                s.content.clone()
            };
            let is_forked = s.base_skill_id.is_some();
            let eff = eff_map.get(s.skill_id.as_str());
            let rating = rating_map.get(s.skill_id.as_str());
            let avg_eff = eff.map_or(0.0, |e| e.avg_effectiveness);

            let extra = SkillViewExtra {
                usage_count: usage,
                content_preview,
                is_forked,
                plugin_names: skill_plugin_map
                    .get(s.skill_id.as_str())
                    .cloned()
                    .unwrap_or_else(Vec::new),
                total_uses: eff.map_or(0, |e| e.total_uses),
                sessions_used_in: eff.map_or(0, |e| e.sessions_used_in),
                avg_effectiveness: format!("{avg_eff:.1}"),
                scored_sessions: eff.map_or(0, |e| e.scored_sessions),
                goal_achievement_pct: format!("{:.0}", eff.map_or(0.0, |e| e.goal_achievement_pct)),
                skill_rating: rating.map(|r| r.rating),
                skill_rating_notes: rating.map(|r| r.notes.clone()),
            };

            let mut v = serde_json::to_value(s)
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
            if let Some(obj) = v.as_object_mut() {
                if let Ok(extra_value) = serde_json::to_value(&extra) {
                    if let Some(extra_obj) = extra_value.as_object() {
                        for (k, val) in extra_obj {
                            obj.insert(k.clone(), val.clone());
                        }
                    }
                }
            }
            v
        })
        .collect()
}

pub async fn my_skill_edit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let skill_id = params.get("id");
    let is_edit = skill_id.is_some();

    let skill = if let Some(id) = skill_id {
        let skills = repositories::list_user_skills(&pool, &user_ctx.user_id)
            .await
            .unwrap_or_else(|_| vec![]);
        skills
            .into_iter()
            .find(|s| s.skill_id.as_str() == id.as_str())
    } else {
        None
    };

    let is_forked = skill
        .as_ref()
        .and_then(|s| s.base_skill_id.as_ref())
        .is_some();
    let skill_json = build_skill_edit_json(skill.as_ref());
    let required_secrets = build_required_secrets(&pool, &user_ctx, skill_id).await;

    let data = MySkillEditPageData {
        page: "my-skill-edit",
        title: if is_edit {
            "Edit My Skill"
        } else {
            "Create My Skill"
        },
        is_edit,
        is_forked,
        skill: skill_json,
        required_secrets,
    };
    let data_value =
        serde_json::to_value(&data).unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    super::render_page(&engine, "my-skill-edit", &data_value, &user_ctx, &mkt_ctx)
}

fn build_skill_edit_json(skill: Option<&crate::types::UserSkill>) -> serde_json::Value {
    skill.map_or_else(
        || {
            serde_json::to_value(&SkillEditView {
                skill_id: String::new(),
                name: String::new(),
                description: String::new(),
                content: String::new(),
                tags: vec![],
                tags_csv: String::new(),
            })
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
        },
        |s| {
            let mut v = serde_json::to_value(s)
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
            if let Some(obj) = v.as_object_mut() {
                if let Some(tags) = obj.get("tags").and_then(|t| t.as_array()) {
                    let csv: String = tags
                        .iter()
                        .filter_map(|t| t.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                    obj.insert("tags_csv".to_string(), serde_json::Value::String(csv));
                }
            }
            v
        },
    )
}

async fn build_required_secrets(
    pool: &PgPool,
    user_ctx: &UserContext,
    skill_id: Option<&String>,
) -> Vec<RequiredSecretView> {
    let Some(sid) = skill_id else {
        return vec![];
    };
    let req_secrets = super::get_services_path().map_or_else(
        |_| vec![],
        |sp| repositories::read_skill_required_secrets(&sp.join("skills"), sid),
    );

    let stored = repositories::list_skill_secrets(pool, &user_ctx.user_id, &SkillId::new(sid))
        .await
        .unwrap_or_else(|_| vec![]);
    let stored_names: HashSet<String> = stored.iter().map(|s| s.var_name.clone()).collect();

    req_secrets
        .iter()
        .map(|rs| RequiredSecretView {
            name: rs.key.clone(),
            description: rs.description.clone(),
            optional: !rs.required,
            is_configured: stored_names.contains(&rs.key),
        })
        .collect()
}
