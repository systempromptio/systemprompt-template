use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;
use systemprompt::models::ProfileBootstrap;

use crate::repositories;
use crate::types::access_control::{
    AccessControlQuery, AccessDecision, BulkAssignRequest, RuleType, UpdateEntityRulesRequest,
};
use crate::types::UpdatePluginRequest;

pub async fn list_access_rules_handler(
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AccessControlQuery>,
) -> Response {
    let result = if let (Some(ref et), Some(ref eid)) = (&query.entity_type, &query.entity_id) {
        repositories::access_control::list_rules_for_entity(&pool, et, eid).await
    } else {
        repositories::access_control::list_all_rules(&pool).await
    };

    match result {
        Ok(rules) => Json(serde_json::json!({ "rules": rules })).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list access control rules");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub async fn update_entity_rules_handler(
    State(pool): State<Arc<PgPool>>,
    Path((entity_type, entity_id)): Path<(String, String)>,
    Json(body): Json<UpdateEntityRulesRequest>,
) -> Response {
    if !["plugin", "agent", "mcp_server", "marketplace"].contains(&entity_type.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid entity_type. Must be plugin, agent, or mcp_server."})),
        )
            .into_response();
    }

    let result = repositories::access_control::set_entity_rules(
        &pool,
        &entity_type,
        &entity_id,
        &body.rules,
    )
    .await;

    match result {
        Ok(rules) => {
            if body.sync_yaml && entity_type == "plugin" {
                sync_plugin_roles_to_yaml(&entity_id, &body.rules);
            }
            Json(serde_json::json!({ "rules": rules })).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, entity_type, entity_id, "Failed to update access control rules");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub async fn bulk_assign_handler(
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<BulkAssignRequest>,
) -> Response {
    let entities: Vec<(String, String)> = body
        .entities
        .iter()
        .map(|e| (e.entity_type.clone(), e.entity_id.clone()))
        .collect();

    match repositories::access_control::bulk_set_rules(&pool, &entities, &body.rules).await {
        Ok(count) => {
            if body.sync_yaml {
                for (et, eid) in &entities {
                    if et == "plugin" {
                        sync_plugin_roles_to_yaml(eid, &body.rules);
                    }
                }
            }
            Json(serde_json::json!({
                "updated_count": count,
                "rules_per_entity": body.rules.len(),
            }))
            .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to bulk assign access control rules");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

pub async fn access_control_departments_handler(State(pool): State<Arc<PgPool>>) -> Response {
    match repositories::fetch_department_stats(&pool).await {
        Ok(stats) => Json(stats).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to fetch department stats");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}

fn sync_plugin_roles_to_yaml(
    plugin_id: &str,
    access_rules: &[crate::types::access_control::AccessControlRuleInput],
) {
    let allowed_roles: Vec<String> = access_rules
        .iter()
        .filter(|r| r.rule_type == RuleType::Role && r.access == AccessDecision::Allow)
        .map(|r| r.rule_value.clone())
        .collect();

    let services_path = match ProfileBootstrap::get() {
        Ok(profile) => std::path::PathBuf::from(&profile.paths.services),
        Err(e) => {
            tracing::warn!(error = %e, plugin_id, "Failed to get services path for YAML sync");
            return;
        }
    };

    let req = UpdatePluginRequest {
        name: None,
        description: None,
        version: None,
        enabled: None,
        category: None,
        keywords: None,
        author_name: None,
        roles: Some(allowed_roles),
        skills: None,
        agents: None,
        mcp_servers: None,
        hooks: None,
    };

    if let Err(e) = repositories::update_plugin(&services_path, plugin_id, &req) {
        tracing::error!(error = %e, plugin_id, "Failed to sync roles to plugin YAML");
    }
}
