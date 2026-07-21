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
use crate::error::{AdminError, AdminResult};
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
) -> AdminResult<Response> {
    let rules = if let (Some(et), Some(eid)) = (&query.entity_type, &query.entity_id) {
        repositories::users::access_control::list_rules_for_entity(&pool, et, eid).await?
    } else {
        repositories::users::access_control::list_all_rules(&pool).await?
    };

    Ok(Json(RulesResponse { rules }).into_response())
}

pub(crate) async fn update_entity_rules_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path((entity_type, entity_id)): Path<(String, String)>,
    Json(body): Json<UpdateEntityRulesRequest>,
) -> AdminResult<Response> {
    if ![
        "plugin",
        "agent",
        "mcp_server",
        "marketplace",
        "gateway_route",
    ]
    .contains(&entity_type.as_str())
    {
        return Err(AdminError::BadRequest(
            "Invalid entity_type. Must be plugin, agent, mcp_server, marketplace, or gateway_route."
                .to_owned(),
        ));
    }

    let rules = repositories::users::access_control::set_entity_rules(
        &pool,
        &entity_type,
        &entity_id,
        &body.rules,
    )
    .await?;

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
    Ok(Json(RulesResponse { rules }).into_response())
}

pub(crate) async fn bulk_assign_handler(
    State(pool): State<Arc<PgPool>>,
    Json(body): Json<BulkAssignRequest>,
) -> AdminResult<Response> {
    let entities: Vec<(String, String)> = body
        .entities
        .iter()
        .map(|e| (e.entity_type.clone(), e.entity_id.clone()))
        .collect();

    let rules_per_entity = body.rules.len();
    let updated_count =
        repositories::users::access_control::bulk_set_rules(&pool, &entities, &body.rules).await?;
    Ok(Json(BulkAssignResponse {
        updated_count,
        rules_per_entity,
    })
    .into_response())
}

pub(crate) async fn user_matrix_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<String>,
) -> AdminResult<Response> {
    let services_path = shared::get_services_path()?;
    let profile_path = shared::get_profile_path()?;

    let sections = build_matrix_sections(&services_path, &profile_path);

    let user_id = UserId::new(user_id);
    let matrix =
        repositories::users::access_control::resolve_user_matrix(&pool, &user_id, sections)
            .await?
            .ok_or_else(|| AdminError::NotFound("User not found".to_owned()))?;
    Ok(Json(matrix).into_response())
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
pub(crate) async fn yaml_snapshot_handler(
    State(pool): State<Arc<PgPool>>,
) -> AdminResult<Response> {
    use crate::repositories::config::acl_yaml_snapshot;
    let yaml = acl_yaml_snapshot::render_yaml_snapshot(&pool).await?;
    Ok((
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, "application/yaml")],
        yaml,
    )
        .into_response())
}

pub(crate) async fn access_control_departments_handler(
    State(pool): State<Arc<PgPool>>,
) -> AdminResult<Response> {
    let stats = repositories::users::user_queries::list_department_stats(&pool).await?;
    Ok(Json(stats).into_response())
}
