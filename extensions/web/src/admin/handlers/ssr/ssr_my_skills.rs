use std::collections::HashSet;
use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

pub(crate) async fn my_skills_page(
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

    let skill_ids: Vec<String> = skills.iter().map(|s| s.skill_id.clone()).collect();
    let usage_counts = repositories::fetch_skill_usage_counts(&pool, &skill_ids).await;

    let mut tags_set = HashSet::new();
    for skill in &skills {
        for tag in &skill.tags {
            if !tag.is_empty() {
                tags_set.insert(tag.clone());
            }
        }
    }
    let mut all_tags: Vec<String> = tags_set.into_iter().collect();
    all_tags.sort();

    let skill_count = skills.len();
    let active_count = skills.iter().filter(|s| s.enabled).count();

    let skills_json: Vec<serde_json::Value> = skills
        .iter()
        .map(|s| {
            let usage = usage_counts.get(&s.skill_id).copied().unwrap_or(0);
            let content_preview = if s.content.len() > 200 {
                format!("{}...", &s.content[..200])
            } else {
                s.content.clone()
            };
            let is_forked = s.base_skill_id.is_some();
            let mut v = serde_json::to_value(s).unwrap_or(json!({}));
            if let Some(obj) = v.as_object_mut() {
                obj.insert("usage_count".to_string(), json!(usage));
                obj.insert("content_preview".to_string(), json!(content_preview));
                obj.insert("is_forked".to_string(), json!(is_forked));
            }
            v
        })
        .collect();

    let data = json!({
        "page": "my-skills",
        "title": "My Skills",
        "skills": skills_json,
        "all_tags": all_tags,
        "stats": {
            "skill_count": skill_count,
            "active_count": active_count,
        },
    });
    super::render_page(&engine, "my-skills", &data, &user_ctx, &mkt_ctx)
}

pub(crate) async fn my_skill_edit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let skill_id = params.get("id");
    let is_edit = skill_id.is_some();

    let skill = if let Some(id) = skill_id {
        let skills = repositories::list_user_skills(&pool, &user_ctx.user_id)
            .await
            .unwrap_or_default();
        skills.into_iter().find(|s| s.skill_id == *id)
    } else {
        None
    };

    let is_forked = skill
        .as_ref()
        .and_then(|s| s.base_skill_id.as_ref())
        .is_some();

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
        "page": "my-skill-edit",
        "title": if is_edit { "Edit My Skill" } else { "Create My Skill" },
        "is_edit": is_edit,
        "is_forked": is_forked,
        "skill": skill_json,
    });
    super::render_page(&engine, "my-skill-edit", &data, &user_ctx, &mkt_ctx)
}
