//! Gateway guard enforcing which routes a caller is entitled to.
//!
//! Contributed by `WebExtension::contribute` through `gateway_guard` and
//! consulted by the gateway on every `/v1/messages` request right after the
//! quota precheck. The resolved route is a `gateway_route` entity, so this is
//! the ordinary authz resolver run over the same rules the access matrix shows,
//! with every subject dimension in the ladder. A denial is 403: no amount of
//! retrying buys a model the caller is not entitled to.

use sqlx::PgPool;
use systemprompt::extension::{GatewayDenyReason, GatewayGuardRequest, GatewayRequestGuard};
use systemprompt::governance::authz::{
    AccessControlRepository, AuthzError, ChainSources, Decision, DecisionFacts, EntityKind,
    ParentChainIndex, ResolveBase, SubjectSourceCell,
};
use systemprompt::identifiers::{RouteId, UserId};
use systemprompt::traits::DatabaseHandle;

use crate::authz;
use crate::repositories::config::gateway_acl;

#[derive(Debug, Clone)]
pub struct RouteEntitlementGuard {
    subjects: SubjectSourceCell,
}

impl RouteEntitlementGuard {
    #[must_use]
    pub const fn new(subjects: SubjectSourceCell) -> Self {
        Self { subjects }
    }
}

#[async_trait::async_trait]
impl GatewayRequestGuard for RouteEntitlementGuard {
    async fn check(
        &self,
        db: &dyn DatabaseHandle,
        request: &GatewayGuardRequest<'_>,
    ) -> Result<(), GatewayDenyReason> {
        let Some(route_id) = request.route_id else {
            return Ok(());
        };
        let Some(database) = db
            .as_any()
            .downcast_ref::<systemprompt::database::Database>()
        else {
            return Err(GatewayDenyReason::unavailable(
                "Authorization database unavailable",
            ));
        };
        let pool = database.pool();
        let user_id = UserId::new(request.user_id.to_owned());
        let route_id = RouteId::new(route_id);
        let decision = resolve_route(pool.as_ref(), &self.subjects, &route_id, request, &user_id)
            .await
            .map_err(|error| {
                tracing::error!(%error, %user_id, %route_id, "authorization_unavailable");
                GatewayDenyReason::unavailable("Authorization temporarily unavailable")
            })?;
        if decision.permits() {
            return Ok(());
        }

        tracing::warn!(
            user_id = %request.user_id,
            route_id = %route_id,
            model = %request.model,
            ?decision,
            "gateway request denied: route not granted to the caller",
        );
        Err(GatewayDenyReason::forbidden(format!(
            "{} is not available to your account.",
            request.model
        )))
    }
}

// Why: the access plane stores `gateway_route/*` and `model/<glob>` as written
// and core matches them at decision time, so the route is resolved through
// core's parent chain with the requested model and provider as facts, never
// against the route's own rows alone.
async fn resolve_route(
    pool: &PgPool,
    subjects: &SubjectSourceCell,
    route_id: &RouteId,
    request: &GatewayGuardRequest<'_>,
    user_id: &UserId,
) -> Result<Decision, RouteDecisionError> {
    let entity = gateway_acl::find_entity(pool, route_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, %route_id, "route entity lookup failed"))?;
    let rules = gateway_acl::list_rules_for_route(pool, route_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, %route_id, "route rule lookup failed"))?;
    let repo = AccessControlRepository::from_pool(std::sync::Arc::new(pool.clone()));
    let index = ParentChainIndex::load(&repo, std::sync::Arc::new(ChainSources::default())).await?;
    let user_roles = load_roles(pool, user_id).await?;
    let attributes = authz::subject_attributes_for(subjects, user_id).await?;
    let dimensions = authz::dimensions(subjects)?;

    Ok(index.resolve(
        EntityKind::GatewayRoute,
        route_id.as_str(),
        ResolveBase {
            rules: &rules,
            user_id,
            user_roles: &user_roles,
            default_included: entity.map(|e| e.default_included),
            attributes: &attributes,
            dimensions: &dimensions,
            facts: DecisionFacts {
                model: Some(request.model.as_str()),
                provider: Some(request.provider.as_str()),
                folder_ancestors: &[],
            },
        },
    ))
}

#[derive(Debug, thiserror::Error)]
enum RouteDecisionError {
    #[error("route grant lookup failed")]
    Database(#[from] sqlx::Error),
    #[error("route parent chain failed to load")]
    ParentChain(#[from] AuthzError),
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
