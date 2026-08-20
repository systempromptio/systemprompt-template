//! Gateway guards enforcing what an organization's plan bought.
//!
//! Two guards, registered through `register_gateway_guard!` and consulted by
//! the gateway on every `/v1/messages` request right after the quota precheck:
//!
//! - [`RouteEntitlementGuard`] — the plan's **model tier**. The resolved route
//!   is a `gateway_route` entity, so this is the ordinary authz resolver run
//!   over the same rules the access matrix shows, with the organization
//!   dimension in the ladder. A denial is 403: no amount of retrying buys a
//!   model the customer did not pay for.
//! - [`OrgBudgetGuard`] — the plan's **monthly spend cap**. A denial is 429
//!   with the default quota kind, because the customer's month does roll over.
//!
//! Why the budget cap is not a `quota_windows` entry now that core supports
//! `subject: organization` buckets: `ai_gateway_policies` rows are global, so a
//! window there is one number for every customer. The cap is per-plan, and a
//! plan is a property of the organization, not of the policy. Core's
//! subject-keyed windows are the right tool for a house-wide backstop; this
//! guard is what makes Standard and Enterprise differ.
//!
//! The cap is enforced one request late, because a request's cost is known only
//! after its response. That is true of core's own ceilings too, and for a
//! monthly contract cap overshooting by one request is immaterial.

use sqlx::PgPool;
use systemprompt::extension::{
    GatewayDenyReason, GatewayGuardRequest, GatewayRequestGuard, register_gateway_guard,
};
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{Decision, EntityRef, ResolveInput, resolve};

use crate::authz;
use crate::repositories::config::gateway_acl;
use crate::repositories::organizations;
use crate::repositories::organizations::spend::OrganizationSpend;

#[derive(Debug, Clone, Copy, Default)]
pub struct RouteEntitlementGuard;

#[derive(Debug, Clone, Copy, Default)]
pub struct OrgBudgetGuard;

#[async_trait::async_trait]
impl GatewayRequestGuard for RouteEntitlementGuard {
    /// Fails **open** on a lookup error and on an unresolved route, and
    /// **closed** on an actual deny decision.
    ///
    /// The asymmetry is deliberate. A deny is an answer — the resolver read
    /// the rules and the customer is not entitled — and honouring it is the
    /// whole point. A database error is not an answer, and treating it as one
    /// would turn a blip into a total inference outage for every customer at
    /// once. An unmatched route means the request never reached a governed
    /// entity and the gateway will fail it on its own terms.
    async fn check(
        &self,
        pool: &PgPool,
        request: &GatewayGuardRequest<'_>,
    ) -> Result<(), GatewayDenyReason> {
        let Some(route_id) = request.route_id else {
            return Ok(());
        };
        let user_id = UserId::new(request.user_id.to_owned());
        let Some(decision) = resolve_route(pool, route_id, &user_id).await else {
            return Ok(());
        };
        let Decision::Deny { reason } = decision else {
            return Ok(());
        };

        tracing::warn!(
            user_id = request.user_id,
            route_id,
            model = request.model,
            ?reason,
            "gateway request denied: route not included in the caller's plan",
        );
        Err(GatewayDenyReason::forbidden(format!(
            "{} is not included in your plan.",
            request.model
        )))
    }
}

async fn resolve_route(pool: &PgPool, route_id: &str, user_id: &UserId) -> Option<Decision> {
    let entity = gateway_acl::find_entity(pool, route_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, route_id, "route entity lookup failed"))
        .ok()?;
    let rules = gateway_acl::list_rules_for_route(pool, route_id)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, route_id, "route rule lookup failed"))
        .ok()?;
    let user_roles = load_roles(pool, user_id).await?;
    let attributes = authz::subject_attributes_for(pool, user_id).await;

    let entity_ref =
        EntityRef::GatewayRoute(systemprompt::identifiers::RouteId::new(route_id.to_owned()));

    Some(resolve(ResolveInput {
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

async fn load_roles(pool: &PgPool, user_id: &UserId) -> Option<Vec<String>> {
    sqlx::query_scalar!(
        r#"SELECT roles AS "roles!: Vec<String>" FROM users WHERE id = $1"#,
        user_id.as_str()
    )
    .fetch_optional(pool)
    .await
    .inspect_err(|e| tracing::warn!(error = %e, user_id = %user_id, "role lookup failed"))
    .ok()
    .flatten()
}

#[async_trait::async_trait]
impl GatewayRequestGuard for OrgBudgetGuard {
    /// Fails **open**: a user with no organization, an organization with no
    /// cap, or a lookup error all allow the request. A cap that is one request
    /// late on a transient error is the cheaper failure, and the audit trail
    /// still records the spend.
    async fn check(
        &self,
        pool: &PgPool,
        request: &GatewayGuardRequest<'_>,
    ) -> Result<(), GatewayDenyReason> {
        let Some(spend) = load_org_spend(pool, &UserId::new(request.user_id.to_owned())).await
        else {
            return Ok(());
        };
        if spend.spent_microdollars < spend.cap_microdollars {
            record_soft_cap_crossing(pool, &spend).await;
            return Ok(());
        }

        tracing::warn!(
            user_id = request.user_id,
            organization = %spend.name,
            spent_microdollars = spend.spent_microdollars,
            cap_microdollars = spend.cap_microdollars,
            "gateway request denied: organization monthly budget exhausted",
        );
        Err(GatewayDenyReason::new(format!(
            "{} has reached its monthly spend cap of ${:.2}. Contact your administrator to raise \
             the plan limit.",
            spend.name,
            micro_to_usd(spend.cap_microdollars),
        )))
    }
}

// Why: warn-only — crossing the soft threshold logs and records the month's
// crossing, never denies. The guard trait has no non-deny channel to the
// client, so the warning surfaces in server logs and the dashboard's spend
// view; a per-request client-visible warning would need a core trait change.
async fn record_soft_cap_crossing(pool: &PgPool, spend: &OrganizationSpend) {
    let Some(warn) = spend.warn_microdollars else {
        return;
    };
    if spend.spent_microdollars < warn {
        return;
    }
    tracing::warn!(
        organization = %spend.name,
        spent_microdollars = spend.spent_microdollars,
        warn_microdollars = warn,
        cap_microdollars = spend.cap_microdollars,
        "organization month-to-date spend crossed its soft budget threshold",
    );
    if let Err(e) = organizations::budget_warnings::upsert_org_budget_warning(
        pool,
        &spend.org_id,
        warn,
        spend.spent_microdollars,
    )
    .await
    {
        tracing::warn!(error = %e, organization = %spend.name, "failed to record soft-cap crossing");
    }
}

async fn load_org_spend(pool: &PgPool, user_id: &UserId) -> Option<OrganizationSpend> {
    organizations::spend::find_spend_for_user(pool, user_id)
        .await
        .inspect_err(|e| {
            tracing::warn!(error = %e, user_id = %user_id, "organization budget lookup failed; allowing request");
        })
        .ok()
        .flatten()
}

#[expect(
    clippy::cast_precision_loss,
    reason = "display only: a contract cap in dollars is far below f64's exact-integer range"
)]
fn micro_to_usd(microdollars: i64) -> f64 {
    microdollars as f64 / 1_000_000.0
}

register_gateway_guard!(RouteEntitlementGuard);
register_gateway_guard!(OrgBudgetGuard);
