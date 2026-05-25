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

use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use sqlx::PgPool;

use systemprompt::identifiers::{RouteId, UserId};
use systemprompt_security::authz::{EntityRef, ResolveInput};

use crate::handlers::shared;
use crate::repositories::{
    self,
    gateway_acl::{self, Decision},
};
use crate::types::{GatewayRouteView, UserContext};

#[derive(Debug, Serialize)]
pub struct CatalogEntry {
    pub id: String,
    pub model_pattern: String,
    pub provider: String,
    pub upstream_model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CatalogResponse {
    pub user_id: String,
    pub routes: Vec<CatalogEntry>,
}

pub async fn for_user_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id): Path<String>,
) -> Response {
    if !user_ctx.is_admin && user_ctx.user_id.as_str() != user_id {
        return shared::error_response(StatusCode::FORBIDDEN, "Forbidden");
    }
    let profile_path = match shared::get_profile_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };
    let cfg = match repositories::get_gateway_config(&profile_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load gateway config");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load gateway",
            );
        }
    };

    let (user_roles, department) =
        match repositories::get_user_roles_department(&pool, &user_id).await {
            Ok(Some(rd)) => rd,
            Ok(None) => return shared::error_response(StatusCode::NOT_FOUND, "User not found"),
            Err(e) => {
                tracing::error!(error = %e, user_id, "Failed to load user roles");
                return shared::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to load user",
                );
            }
        };

    match collect_allowed_routes(&pool, &cfg.routes, &user_id, &user_roles, &department).await {
        Ok(routes) => Json(CatalogResponse { user_id, routes }).into_response(),
        Err(resp) => *resp,
    }
}

async fn collect_allowed_routes(
    pool: &PgPool,
    routes: &[GatewayRouteView],
    user_id: &str,
    user_roles: &[String],
    department: &str,
) -> Result<Vec<CatalogEntry>, Box<Response>> {
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
        let default_included = gateway_acl::get_entity(pool, &route.id)
            .await
            .unwrap_or_else(|e| {
                tracing::error!(error = %e, route_id = %route.id, "Failed to load catalog entity");
                None
            })
            .map(|e| e.default_included);
        let entity = EntityRef::GatewayRoute(RouteId::new(route.id.clone()));
        let uid = UserId::new(user_id);
        if matches!(
            gateway_acl::resolve(ResolveInput {
                entity: &entity,
                rules: &rules,
                user_id: &uid,
                user_roles,
                department,
                default_included,
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
pub struct DetectQuery {
    #[serde(default = "default_since_minutes")]
    pub since_minutes: i64,
}

const fn default_since_minutes() -> i64 {
    60
}

#[derive(Debug, Serialize)]
pub struct DetectResponse {
    pub emitted: usize,
    pub since_minutes: i64,
}

/// Admin-triggered after-the-fact detector. POST endpoint that scans recent
/// `ai_requests` and emits `governance_decisions` rows for denied combos.
/// Until a scheduled job wires this up automatically, admins can poke it
/// from the CLI or a dashboard button — gap deliberately small.
pub async fn detect_handler(
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
    let cfg = match repositories::get_gateway_config(&profile_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "Failed to load gateway config");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load gateway",
            );
        }
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
        }
    }
}

/// After-the-fact detector: scan recent `ai_requests` and emit a
/// `governance_decisions` row for any request whose user/model combination
/// the ACL would have denied. Best-effort; called by [`detect_handler`].
pub async fn detect_after_the_fact(
    pool: &PgPool,
    routes: &[GatewayRouteView],
    since_minutes: i64,
) -> Result<usize, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT id AS "id!", user_id, session_id, model
           FROM ai_requests
           WHERE created_at >= NOW() - ($1 || ' minutes')::interval
             AND status NOT IN ('rejected', 'denied')"#,
        since_minutes.to_string()
    )
    .fetch_all(pool)
    .await?;

    let mut emitted = 0usize;
    for row in rows {
        let Some(route) = repositories::find_matching_route(routes, &row.model) else {
            continue;
        };
        let Some((user_roles, department)) =
            repositories::get_user_roles_department(pool, &row.user_id).await?
        else {
            continue;
        };
        let rules = gateway_acl::list_rules_for_route(pool, &route.id).await?;
        let default_included = gateway_acl::get_entity(pool, &route.id)
            .await?
            .map(|e| e.default_included);
        let entity = EntityRef::GatewayRoute(RouteId::new(route.id.clone()));
        let uid = UserId::new(&row.user_id);
        if let Decision::Deny { reason } = gateway_acl::resolve(ResolveInput {
            entity: &entity,
            rules: &rules,
            user_id: &uid,
            user_roles: &user_roles,
            department: &department,
            default_included,
        }) {
            let decision_id = uuid::Uuid::new_v4().to_string();
            let reason_str = reason.to_string();
            let evaluated = serde_json::json!({
                "ai_request_id": row.id,
                "model": row.model,
                "matched_route_id": route.id,
                "reason": reason,
            });
            sqlx::query!(
                "INSERT INTO governance_decisions \
                 (id, user_id, session_id, tool_name, agent_id, agent_scope, \
                  decision, policy, reason, evaluated_rules, plugin_id) \
                 VALUES ($1, $2, $3, $4, NULL, $5, $6, 'gateway_acl', $7, $8, NULL)",
                decision_id,
                row.user_id,
                row.session_id.unwrap_or_default(),
                row.model,
                "inference",
                "deny_after_the_fact",
                reason_str,
                evaluated,
            )
            .execute(pool)
            .await?;
            emitted += 1;
        }
    }
    Ok(emitted)
}
