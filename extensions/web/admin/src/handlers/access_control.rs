use std::sync::Arc;

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use crate::activity::{self, ActivityEntity, NewActivity};
use crate::repositories;
use crate::types::access_control::{
    AccessControlQuery, BulkAssignRequest, UpdateEntityRulesRequest,
};
use crate::types::UserContext;

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
                Json(serde_json::json!({"error": "Internal server error"})),
            )
                .into_response()
        }
    }
}

pub async fn update_entity_rules_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path((entity_type, entity_id)): Path<(String, String)>,
    Json(body): Json<UpdateEntityRulesRequest>,
) -> Response {
    if ![
        "plugin",
        "agent",
        "mcp_server",
        "marketplace",
        "gateway_route",
    ]
    .contains(&entity_type.as_str())
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Invalid entity_type. Must be plugin, agent, mcp_server, marketplace, or gateway_route."})),
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
            if entity_type == "gateway_route" {
                let uid = user_ctx.user_id.clone();
                let eid = entity_id.clone();
                let pool_arc = Arc::clone(&pool);
                tokio::spawn(async move {
                    activity::record(
                        &pool_arc,
                        NewActivity::entity_updated(&uid, ActivityEntity::GatewayRoute, &eid, &eid),
                    )
                    .await;
                });
            }
            Json(serde_json::json!({ "rules": rules })).into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, entity_type, entity_id, "Failed to update access control rules");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal server error"})),
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
        Ok(count) => Json(serde_json::json!({
            "updated_count": count,
            "rules_per_entity": body.rules.len(),
        }))
        .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to bulk assign access control rules");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal server error"})),
            )
                .into_response()
        }
    }
}

pub async fn user_matrix_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<String>,
) -> Response {
    use crate::handlers::shared;
    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    let profile_path = match shared::get_profile_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let sections = build_matrix_sections(&services_path, &profile_path);

    match repositories::access_control::resolve_user_matrix(&pool, &user_id, sections).await {
        Ok(matrix) => Json(matrix).into_response(),
        Err(e) => {
            tracing::error!(error = %e, user_id, "Failed to resolve user matrix");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal server error"})),
            )
                .into_response()
        }
    }
}

fn build_matrix_sections(
    services_path: &std::path::Path,
    profile_path: &std::path::Path,
) -> Vec<repositories::access_control::SectionInput> {
    // Each source is best-effort: a config that fails to load is skipped so the
    // matrix renders whatever resolved instead of failing the whole view.
    let mut sections: Vec<repositories::access_control::SectionInput> = Vec::new();

    if let Ok(cfg) = repositories::get_gateway_config(profile_path) {
        let rows = cfg
            .routes
            .into_iter()
            .map(|r| {
                let label = r.model_pattern.clone();
                (r.id, label, None)
            })
            .collect();
        sections.push((
            "gateway_route".to_string(),
            "Gateway routes".to_string(),
            rows,
        ));
    }
    if let Ok(servers) = repositories::mcp_servers::list_mcp_servers(services_path) {
        let rows: Vec<(String, String, Option<String>)> = servers
            .into_iter()
            .map(|s| {
                let id = s.id.as_str().to_string();
                let desc = if s.description.is_empty() {
                    None
                } else {
                    Some(s.description)
                };
                (id.clone(), id, desc)
            })
            .collect();
        sections.push(("mcp_server".to_string(), "MCP servers".to_string(), rows));
    }
    let admin_roles = vec!["admin".to_string()];
    if let Ok(plugins) = repositories::list_plugins_for_roles(services_path, &admin_roles) {
        let rows: Vec<(String, String, Option<String>)> = plugins
            .into_iter()
            .map(|p| (p.id, p.name, Some(p.description)))
            .collect();
        sections.push(("plugin".to_string(), "Plugins".to_string(), rows));
    }
    if let Ok(agents) = repositories::list_agents(services_path) {
        let rows: Vec<(String, String, Option<String>)> = agents
            .into_iter()
            .map(|a| {
                (
                    a.id.as_str().to_string(),
                    a.name.clone(),
                    Some(a.description.clone()),
                )
            })
            .collect();
        sections.push(("agent".to_string(), "Agents".to_string(), rows));
    }
    if let Ok(skills) = repositories::list_skill_catalog(services_path) {
        let rows: Vec<(String, String, Option<String>)> = skills
            .into_iter()
            .map(|s| {
                let desc = if s.description.is_empty() {
                    None
                } else {
                    Some(s.description)
                };
                (s.id.as_str().to_string(), s.name, desc)
            })
            .collect();
        sections.push(("skill".to_string(), "Skills".to_string(), rows));
    }

    sections
}

/// Read-only YAML rendering of the current DB state of role/department rules.
/// Used by the dashboard's "Show as YAML" button so admins can copy-paste
/// instance-local edits into the committed baseline. Writes nothing to disk.
pub async fn yaml_snapshot_handler(State(pool): State<Arc<PgPool>>) -> Response {
    use crate::repositories::governance_grp::acl_yaml_snapshot;
    match acl_yaml_snapshot::render_yaml_snapshot(&pool).await {
        Ok(yaml) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "application/yaml")],
            yaml,
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to render yaml snapshot");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Internal server error"})),
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
                Json(serde_json::json!({"error": "Internal server error"})),
            )
                .into_response()
        }
    }
}
