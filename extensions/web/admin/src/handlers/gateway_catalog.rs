//! Per-user gateway catalog and defense-in-depth detector.
//!
//! Real enforcement happens in core via the `AuthzDecisionHook` (template's
//! `/govern/authz` webhook). This module provides two redundant surfaces:
//!
//! * [`for_user_handler`] — `GET /api/admin/gateway/catalog/for-user/{user_id}`
//!   filters the catalog to routes the user can see, reducing UI clutter.
//! * [`detect_handler`] — sweeps recent `ai_requests` and writes
//!   `governance_decisions` rows for any request that should not have been
//!   allowed. Acts as a redundancy check that core enforcement actually fired.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;

use systemprompt::identifiers::{RouteId, UserId};
use systemprompt_security::authz::{EntityRef, ResolveInput};

use crate::authz::{dimensions, subject_attributes_for};
use crate::handlers::shared;
use crate::repositories;
use crate::repositories::config::acl_detect;
use crate::repositories::config::gateway_acl::{self, Decision};

use crate::types::{GatewayRouteView, UserContext};

#[derive(Debug, Serialize)]
pub(crate) struct CatalogEntry {
    pub id: String,
    pub model_pattern: String,
    pub provider: String,
    pub upstream_model: Option<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct CatalogResponse {
    pub user_id: UserId,
    pub routes: Vec<CatalogEntry>,
}

pub(crate) async fn for_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id): Path<String>,
) -> Response {
    let user_id = UserId::new(user_id);
    if !user_ctx.is_admin && user_ctx.user_id != user_id {
        return shared::error_response(StatusCode::FORBIDDEN, "Forbidden");
    }
    let profile_path = match shared::get_profile_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    let cfg = match repositories::config::gateway::get_gateway_config(&profile_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load gateway config");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load gateway",
            );
        },
    };

    let (user_roles, _department) =
        match repositories::users::queries::find_user_roles_department(&pool, &user_id).await {
            Ok(Some(rd)) => rd,
            Ok(None) => return shared::error_response(StatusCode::NOT_FOUND, "User not found"),
            Err(e) => {
                tracing::error!(error = %e, user_id = %user_id, "Failed to load user roles");
                return shared::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to load user",
                );
            },
        };

    match collect_allowed_routes(&pool, &cfg.routes, &user_id, &user_roles).await {
        Ok(routes) => Json(CatalogResponse { user_id, routes }).into_response(),
        Err(resp) => *resp,
    }
}

async fn collect_allowed_routes(
    pool: &PgPool,
    routes: &[GatewayRouteView],
    user_id: &UserId,
    user_roles: &[String],
) -> Result<Vec<CatalogEntry>, Box<Response>> {
    let attributes = subject_attributes_for(pool, user_id).await;
    let mut allowed = Vec::with_capacity(routes.len());
    for route in routes {
        let rules = gateway_acl::list_rules_for_route(pool, &route.id)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, route_id = %route.id, "Failed to list ACL rules");
                Box::new(shared::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to load ACL rules",
                ))
            })?;
        let default_included = gateway_acl::find_entity(pool, &route.id)
            .await
            .unwrap_or_else(|e| {
                tracing::error!(error = %e, route_id = %route.id, "Failed to load catalog entity");
                None
            })
            .map(|e| e.default_included);
        let entity = EntityRef::GatewayRoute(RouteId::new(route.id.clone()));
        if matches!(
            gateway_acl::resolve(ResolveInput {
                entity: &entity,
                rules: &rules,
                user_id,
                user_roles,
                default_included,
                parents: &[],
                attributes: &attributes,
                dimensions: dimensions(pool),
            }),
            Decision::Allow { .. }
        ) {
            allowed.push(CatalogEntry {
                id: route.id.clone(),
                model_pattern: route.model_pattern.clone(),
                provider: route.provider.clone(),
                upstream_model: route.upstream_model.clone(),
            });
        }
    }
    Ok(allowed)
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct DetectQuery {
    #[serde(default = "default_since_minutes")]
    pub since_minutes: i64,
}

const fn default_since_minutes() -> i64 {
    60
}

#[derive(Debug, Serialize)]
pub(crate) struct DetectResponse {
    pub emitted: usize,
    pub since_minutes: i64,
}

/// Detection is admin-triggered rather than scheduled, so decisions for
/// denied combinations only appear once someone runs it.
pub(crate) async fn detect_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    axum::extract::Query(query): axum::extract::Query<DetectQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return shared::error_response(StatusCode::FORBIDDEN, "Admin only");
    }
    let profile_path = match shared::get_profile_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    let cfg = match repositories::config::gateway::get_gateway_config(&profile_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load gateway config");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load gateway",
            );
        },
    };
    match detect_after_the_fact(&pool, &cfg.routes, query.since_minutes).await {
        Ok(emitted) => Json(DetectResponse {
            emitted,
            since_minutes: query.since_minutes,
        })
        .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "ACL detector failed");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Detector failed")
        },
    }
}

/// After-the-fact detector: scan recent `ai_requests` and emit a
/// `governance_decisions` row for any request whose user/model combination
/// the ACL would have denied. Best-effort; called by [`detect_handler`].
pub(crate) async fn detect_after_the_fact(
    pool: &PgPool,
    routes: &[GatewayRouteView],
    since_minutes: i64,
) -> Result<usize, sqlx::Error> {
    let rows = acl_detect::list_recent_unrejected_requests(pool, since_minutes).await?;

    let mut emitted = 0usize;
    for row in rows {
        let Some(route) = repositories::config::gateway::find_matching_route(routes, &row.model)
        else {
            continue;
        };
        let Some((user_roles, _department)) =
            repositories::users::queries::find_user_roles_department(pool, &row.user_id).await?
        else {
            continue;
        };
        let attributes = subject_attributes_for(pool, &UserId::new(&row.user_id)).await;
        let rules = gateway_acl::list_rules_for_route(pool, &route.id).await?;
        let default_included = gateway_acl::find_entity(pool, &route.id)
            .await?
            .map(|e| e.default_included);
        let entity = EntityRef::GatewayRoute(RouteId::new(route.id.clone()));
        let uid = UserId::new(&row.user_id);
        if let Decision::Deny { reason } = gateway_acl::resolve(ResolveInput {
            entity: &entity,
            rules: &rules,
            user_id: &uid,
            user_roles: &user_roles,
            default_included,
            parents: &[],
            attributes: &attributes,
            dimensions: dimensions(pool),
        }) {
            let decision_id = uuid::Uuid::new_v4().to_string();
            let reason_str = reason.to_string();
            let session_id = row
                .session_id
                .as_ref()
                .map(|s| s.as_str().to_owned())
                .unwrap_or_default();
            // variable-shape: governance audit `evaluated_rules` JSONB payload, not a
            // template/response body
            let evaluated = serde_json::json!({
                "ai_request_id": row.id,
                "model": row.model,
                "matched_route_id": route.id,
                "reason": reason,
            });
            acl_detect::insert_gateway_acl_decision(
                pool,
                acl_detect::GatewayAclDecision {
                    decision_id: &decision_id,
                    user_id: row.user_id.as_str(),
                    session_id: &session_id,
                    model: &row.model,
                    agent_scope: "inference",
                    decision: "deny_after_the_fact",
                    reason: &reason_str,
                    evaluated_rules: &evaluated,
                },
            )
            .await?;
            emitted += 1;
        }
    }
    Ok(emitted)
}
