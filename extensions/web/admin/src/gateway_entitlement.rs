//! Gateway guard enforcing which routes a caller is entitled to.
//!
//! Registered through `register_gateway_guard!` and consulted by the gateway
//! on every `/v1/messages` request right after the quota precheck. The
//! resolved route is a `gateway_route` entity, so this is the ordinary authz
//! resolver run over the same rules the access matrix shows, with every
//! subject dimension in the ladder. A denial is 403: no amount of retrying
//! buys a model the caller is not entitled to.

use sqlx::PgPool;
use systemprompt::extension::{
    GatewayDenyReason, GatewayGuardRequest, GatewayRequestGuard, register_gateway_guard,
};
use systemprompt::identifiers::{RouteId, UserId};
use systemprompt_security::authz::resolver::{ResolveInput, resolve};
use systemprompt_security::authz::{Decision, EntityRef};

use crate::authz;
use crate::repositories::config::gateway_acl;

#[derive(Debug, Clone, Copy, Default)]
pub struct RouteEntitlementGuard;

#[async_trait::async_trait]
impl GatewayRequestGuard for RouteEntitlementGuard {
    async fn check(
        &self,
        pool: &PgPool,
        request: &GatewayGuardRequest<'_>,
    ) -> Result<(), GatewayDenyReason> {
        let Some(route_id) = request.route_id else {
            return Ok(());
        };
        let user_id = UserId::new(request.user_id.to_owned());
        let decision = resolve_route(pool, &RouteId::new(route_id), &user_id)
            .await
            .map_err(|error| {
                tracing::error!(%error, %user_id, route_id, "authorization_unavailable");
                GatewayDenyReason::unavailable("Authorization temporarily unavailable")
            })?;
        if decision.permits() {
            return Ok(());
        }

        tracing::warn!(
            user_id = request.user_id,
            route_id,
            model = request.model,
            ?decision,
            "gateway request denied: route not granted to the caller",
        );
        Err(GatewayDenyReason::forbidden(format!(
            "{} is not available to your account.",
            request.model
        )))
    }
}

async fn resolve_route(
    pool: &PgPool,
    route_id: &RouteId,
    user_id: &UserId,
) -> Result<Decision, sqlx::Error> {
    let entity = gateway_acl::find_entity(pool, route_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, %route_id, "route entity lookup failed"))?;
    let rules = gateway_acl::list_rules_for_route(pool, route_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, %route_id, "route rule lookup failed"))?;
    let user_roles = load_roles(pool, user_id).await?;
    let attributes = authz::subject_attributes_for(pool, user_id).await?;

    let entity_ref = EntityRef::GatewayRoute(route_id.clone());

    Ok(resolve(ResolveInput {
        entity: &entity_ref,
        rules: &rules,
        user_id,
        user_roles: &user_roles,
        default_included: entity.map(|e| e.default_included),
        parents: &[],
        attributes: &attributes,
        dimensions: authz::dimensions(pool),
    }))
}

async fn load_roles(pool: &PgPool, user_id: &UserId) -> Result<Vec<String>, sqlx::Error> {
    let identity = crate::repositories::users::queries::find_identity_envelope(pool, user_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;
    if identity.status != "active" {
        return Err(sqlx::Error::RowNotFound);
    }
    Ok(identity.roles)
}

register_gateway_guard!(RouteEntitlementGuard);
