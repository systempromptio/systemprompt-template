//! HTTP handlers for the access-control matrix and rule editing.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::activity::{self, ActivityEntity, NewActivity};
use crate::handlers::shared;
use crate::repositories;
use crate::types::UserContext;
use crate::types::access_control::{
    AccessControlQuery, AccessControlRule, BulkAssignRequest, UpdateEntityRulesRequest,
};

/// JSON body returned by the rule-listing endpoints (`{ "rules": [...] }`).
#[derive(Debug, Serialize)]
pub(crate) struct RulesResponse {
    pub rules: Vec<AccessControlRule>,
}

/// JSON body returned by `bulk_assign_handler`.
#[derive(Debug, Serialize)]
pub(crate) struct BulkAssignResponse {
    pub updated_count: usize,
    pub rules_per_entity: usize,
}

pub(crate) async fn list_access_rules_handler(
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<AccessControlQuery>,
) -> Response {
    let result = if let (Some(et), Some(eid)) = (&query.entity_type, &query.entity_id) {
        repositories::users::access_control::list_rules_for_entity(&pool, et, eid).await
    } else {
        repositories::users::access_control::list_all_rules(&pool).await
    };

    match result {
        Ok(rules) => Json(RulesResponse { rules }).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list access control rules");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        },
    }
}

pub(crate) async fn update_entity_rules_handler(
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
        return shared::error_response(
            StatusCode::BAD_REQUEST,
            "Invalid entity_type. Must be plugin, agent, mcp_server, marketplace, or gateway_route.",
        );
    }

    let result = repositories::users::access_control::set_entity_rules(
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
            Json(RulesResponse { rules }).into_response()
        },
        Err(e) => {
            tracing::error!(error = %e, entity_type, entity_id, "Failed to update access control rules");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        },
    }
}

pub(crate) async fn bulk_assign_handler(
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<BulkAssignRequest>,
) -> Response {
    let entities: Vec<(String, String)> = body
        .entities
        .iter()
        .map(|e| (e.entity_type.clone(), e.entity_id.clone()))
        .collect();

    let rules_per_entity = body.rules.len();
    match repositories::users::access_control::bulk_set_rules(&pool, &entities, &body.rules).await {
        Ok(updated_count) => Json(BulkAssignResponse {
            updated_count,
            rules_per_entity,
        })
        .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to bulk assign access control rules");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        },
    }
}

pub(crate) async fn user_matrix_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<String>,
) -> Response {
    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };
    let profile_path = match shared::get_profile_path() {
        Ok(p) => p,
        Err(e) => return e.into_response(),
    };

    let sections = build_matrix_sections(&services_path, &profile_path);

    let user_id = UserId::new(user_id);
    match repositories::users::access_control::resolve_user_matrix(&pool, &user_id, sections).await
    {
        Ok(Some(matrix)) => Json(matrix).into_response(),
        Ok(None) => shared::error_response(StatusCode::NOT_FOUND, "User not found"),
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "Failed to resolve user matrix");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        },
    }
}

fn build_matrix_sections(
    services_path: &std::path::Path,
    profile_path: &std::path::Path,
) -> Vec<repositories::users::access_control::SectionInput> {
    // Each source is best-effort: a config that fails to load is skipped so the
    // matrix renders whatever resolved instead of failing the whole view.
    let mut sections: Vec<repositories::users::access_control::SectionInput> = Vec::new();

    if let Ok(cfg) = repositories::config::gateway::get_gateway_config(profile_path) {
        let rows = cfg
            .routes
            .into_iter()
            .map(|r| {
                let label = r.model_pattern.clone();
                (r.id, label, None)
            })
            .collect();
        sections.push((
            "gateway_route".to_owned(),
            "Gateway routes".to_owned(),
            rows,
        ));
    }
    if let Ok(servers) = repositories::mcp::mcp_servers::list_mcp_servers(services_path) {
        let rows: Vec<(String, String, Option<String>)> = servers
            .into_iter()
            .map(|s| {
                let id = s.id.as_str().to_owned();
                let desc = if s.description.is_empty() {
                    None
                } else {
                    Some(s.description)
                };
                (id.clone(), id, desc)
            })
            .collect();
        sections.push(("mcp_server".to_owned(), "MCP servers".to_owned(), rows));
    }
    let admin_roles = vec!["admin".to_owned()];
    if let Ok(plugins) =
        repositories::marketplace::plugins::list_plugins_for_roles(services_path, &admin_roles)
    {
        let rows: Vec<(String, String, Option<String>)> = plugins
            .into_iter()
            .map(|p| (p.id, p.name, Some(p.description)))
            .collect();
        sections.push(("plugin".to_owned(), "Plugins".to_owned(), rows));
    }
    if let Ok(agents) = repositories::config::agents::list_agents(services_path) {
        let rows: Vec<(String, String, Option<String>)> = agents
            .into_iter()
            .map(|a| {
                (
                    a.id.as_str().to_owned(),
                    a.name.clone(),
                    Some(a.description.clone()),
                )
            })
            .collect();
        sections.push(("agent".to_owned(), "Agents".to_owned(), rows));
    }
    if let Ok(skills) = repositories::marketplace::plugins::list_skill_catalog(services_path) {
        let rows: Vec<(String, String, Option<String>)> = skills
            .into_iter()
            .map(|s| {
                let desc = if s.description.is_empty() {
                    None
                } else {
                    Some(s.description)
                };
                (s.id.as_str().to_owned(), s.name, desc)
            })
            .collect();
        sections.push(("skill".to_owned(), "Skills".to_owned(), rows));
    }

    sections
}

/// Renders current DB state as YAML for copying into the committed baseline.
/// Writes nothing to disk — instances never write back to `services/`.
pub(crate) async fn yaml_snapshot_handler(State(pool): State<Arc<PgPool>>) -> Response {
    use crate::repositories::config::acl_yaml_snapshot;
    match acl_yaml_snapshot::render_yaml_snapshot(&pool).await {
        Ok(yaml) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "application/yaml")],
            yaml,
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to render yaml snapshot");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        },
    }
}

pub(crate) async fn access_control_departments_handler(
    State(pool): State<Arc<PgPool>>,
) -> Response {
    match repositories::users::user_queries::list_department_stats(&pool).await {
        Ok(stats) => Json(stats).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to fetch department stats");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
        },
    }
}
