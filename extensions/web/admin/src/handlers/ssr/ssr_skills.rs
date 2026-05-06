use std::sync::Arc;

use systemprompt::identifiers::SkillId;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{IdQuery, MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

pub async fn skill_edit_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<IdQuery>,
) -> Response {
    let skill_id = params.id();
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
