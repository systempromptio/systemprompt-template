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

use chrono::{DateTime, Months, Utc};
use sqlx::PgPool;
use systemprompt::extension::{
    GatewayDenyKind, GatewayDenyReason, GatewayGuardRequest, GatewayRequestGuard,
    register_gateway_guard,
};
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{Decision, EntityRef, ResolveInput, resolve};

use crate::authz;
use crate::repositories::config::gateway_acl;
use crate::repositories::organizations;
use crate::repositories::organizations::spend::OrganizationSpend;
use crate::util::month_range::month_start;

#[derive(Debug, Clone, Copy, Default)]
pub struct RouteEntitlementGuard;

#[derive(Debug, Clone, Copy, Default)]
pub struct OrgBudgetGuard;

#[async_trait::async_trait]
impl GatewayRequestGuard for RouteEntitlementGuard {
    // Why: Fails **open** on a lookup error and on an unresolved route, and
    // **closed** on an actual deny decision.
    //
    // The asymmetry is deliberate. A deny is an answer — the resolver read
    // the rules and the customer is not entitled — and honouring it is the
    // whole point. A database error is not an answer, and treating it as one
    // would turn a blip into a total inference outage for every customer at
    // once. An unmatched route means the request never reached a governed
    // entity and the gateway will fail it on its own terms.
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
    // Why: Fails **open**: a user with no organization, an organization with no
    // cap, or a lookup error all allow the request. A cap that is one request
    // late on a transient error is the cheaper failure, and the audit trail
    // still records the spend.
    //
    // The denial carries `retry-after` set to the month boundary rather than
    // the default zero. A budget deny is a 429 because the month rolls over,
    // but it does not roll over *soon*, and a zero tells a well-behaved client
    // to come straight back — which is how one exhausted cap turns into ten
    // rejected requests.
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
            record_forecast_overrun(pool, &spend, Utc::now()).await;
            return Ok(());
        }

        tracing::warn!(
            user_id = request.user_id,
            organization = %spend.name,
            spent_microdollars = spend.spent_microdollars,
            cap_microdollars = spend.cap_microdollars,
            "gateway request denied: organization monthly budget exhausted",
        );
        Err(GatewayDenyReason {
            message: format!(
                "{} has reached its monthly spend cap of {}. Contact your administrator to raise \
                 the plan limit.",
                spend.name,
                display_usd(spend.cap_microdollars),
            ),
            retry_after_seconds: seconds_until_next_month(Utc::now()),
            kind: GatewayDenyKind::Quota,
        })
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
    match organizations::budget_warnings::upsert_org_budget_warning(
        pool,
        &spend.org_id,
        organizations::budget_warnings::BudgetWarningKind::SoftCap,
        warn,
        spend.spent_microdollars,
    )
    .await
    {
        // Why: only the first crossing alerts. This runs on every request once
        // spend is past the threshold, so alerting unconditionally would post
        // once per request for the rest of the month. The dashboard already
        // shows the standing state; the alert is for the transition.
        Ok(true) => crate::slack_alerts::send_alert(format!(
            "*Budget warning* — {} has crossed its soft spend threshold for this \
             month. Spent {} of a {} cap (warning at {}).",
            spend.name,
            display_usd(spend.spent_microdollars),
            display_usd(spend.cap_microdollars),
            display_usd(warn),
        )),
        Ok(false) => {},
        Err(e) => {
            tracing::warn!(error = %e, organization = %spend.name, "failed to record soft-cap crossing");
        },
    }
}

// Why: three or more elapsed days before projecting — a linear run-rate over
// the month's first hours multiplies noise into an alert, and a projection
// that cries wolf on the 1st teaches budget owners to ignore the one that
// matters on the 20th. The projection alerts only while actual spend is still
// under the cap: once the cap itself is crossed the guard denies, which is a
// louder signal than any forecast.
const MIN_PROJECTION_ELAPSED_SECONDS: i64 = 3 * 24 * 3600;

async fn record_forecast_overrun(pool: &PgPool, spend: &OrganizationSpend, now: DateTime<Utc>) {
    let Some(projected) = projected_month_end_spend(spend.spent_microdollars, now) else {
        return;
    };
    if projected <= spend.cap_microdollars {
        return;
    }
    tracing::warn!(
        organization = %spend.name,
        spent_microdollars = spend.spent_microdollars,
        projected_microdollars = projected,
        cap_microdollars = spend.cap_microdollars,
        "organization spend is on pace to exceed its monthly cap",
    );
    match organizations::budget_warnings::upsert_org_budget_warning(
        pool,
        &spend.org_id,
        organizations::budget_warnings::BudgetWarningKind::ForecastOverrun,
        spend.cap_microdollars,
        spend.spent_microdollars,
    )
    .await
    {
        Ok(true) => crate::slack_alerts::send_alert(format!(
            "*Budget forecast warning* — {} is on pace to exceed its monthly cap. Spent {} so \
             far; linear projection ~{} against a {} cap. Intervene now or the cap will start \
             refusing requests before month end.",
            spend.name,
            display_usd(spend.spent_microdollars),
            display_usd(projected),
            display_usd(spend.cap_microdollars),
        )),
        Ok(false) => {},
        Err(e) => {
            tracing::warn!(error = %e, organization = %spend.name, "failed to record forecast-overrun crossing");
        },
    }
}

// Why: seconds-based rather than day-based so the projection moves smoothly
// through the day instead of jumping at midnight, and `None` before the
// minimum elapsed window rather than a wildly-extrapolated number.
pub fn projected_month_end_spend(spent_microdollars: i64, now: DateTime<Utc>) -> Option<i64> {
    let start = month_start(now);
    let elapsed = (now - start).num_seconds();
    if elapsed < MIN_PROJECTION_ELAPSED_SECONDS {
        return None;
    }
    let month_total = (month_start(now) + Months::new(1) - start).num_seconds();
    Some(spent_microdollars.saturating_mul(month_total) / elapsed.max(1))
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

// Why: Seconds from `now` until the next calendar month begins in UTC. The
// boundary is UTC month start because that is what the spend query counts from
// (`DATE_TRUNC('month', NOW())`), so the hint expires exactly when the budget
// it describes resets. Shares `month_start` with the month-scoped reports,
// which bill against the same boundary.
pub fn seconds_until_next_month(now: DateTime<Utc>) -> i32 {
    let seconds = (month_start(now) + Months::new(1) - now).num_seconds();
    i32::try_from(seconds.max(0)).unwrap_or(i32::MAX)
}

// Why: two decimals round a sub-cent evaluation cap to "$0.00", which reads
// as "no cap at all" in the denial message; tiny caps get four decimals.
#[expect(
    clippy::cast_precision_loss,
    reason = "display only: a contract cap in dollars is far below f64's exact-integer range"
)]
fn display_usd(microdollars: i64) -> String {
    let usd = microdollars as f64 / 1_000_000.0;
    if microdollars < 10_000 {
        format!("${usd:.4}")
    } else {
        format!("${usd:.2}")
    }
}

register_gateway_guard!(RouteEntitlementGuard);
register_gateway_guard!(OrgBudgetGuard);
