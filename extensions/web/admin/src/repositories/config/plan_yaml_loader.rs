//! Bootstrap loader: `services/access-control/plans.yaml` → DB.
//!
//! Runs inside the same governance bootstrap pass as the roles and departments
//! files, after them, because a plan's grants reference marketplaces and routes
//! whose catalog rows the earlier passes materialise.
//!
//! Three steps: upsert `plans`, upsert `organizations`, then project each
//! organization's plan grants into `access_control_rules` at
//! `rule_type = 'organization'`. Projection is the whole point — a plan is a
//! named bundle of ordinary rules, so nothing downstream needs to know plans
//! exist. The resolver, the access matrix, and the governance audit all see
//! rule rows they already understand.
//!
//! Like every other bootstrap loader here, the direction is fixed (YAML → DB)
//! and there is no write-back: a customer's own narrowing of their plan lives
//! in the DB and survives redeploys, because projection only ever touches rows
//! whose `rule_value` is an organization slug.

use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use sqlx::PgPool;
use systemprompt_security::authz::{Access, AccessControlRepository, EntityKind, UpsertRuleParams};
use systemprompt_web_shared::error::MarketplaceError;

use crate::authz::organization::organization_rule_type;

use super::plan_yaml_types::{PlanLoadReport, PlansDoc, YamlGrant, YamlOrganization, YamlPlan};

const PLANS_FILE: &str = "access-control/plans.yaml";

const SOURCE_LABEL: &str = "plans.yaml";

const MICRODOLLARS_PER_USD: f64 = 1_000_000.0;

pub async fn load_plans_from_yaml(
    pool: &PgPool,
    services_path: &Path,
) -> Result<PlanLoadReport, MarketplaceError> {
    let mut report = PlanLoadReport::default();
    let Some(doc) = read_plans(services_path).await? else {
        return Ok(report);
    };

    for plan in &doc.plans {
        upsert_plan(pool, plan).await?;
        report.plans_upserted += 1;
    }

    let repo = AccessControlRepository::from_pool(Arc::new(pool.clone()));
    for org in &doc.organizations {
        // Why: the plan must resolve before the insert, or the organizations
        // FK fails first and the operator gets a constraint name instead of
        // the line of YAML that is wrong.
        let Some(plan) = doc.plans.iter().find(|p| p.id == org.plan) else {
            return Err(MarketplaceError::Internal(format!(
                "{PLANS_FILE}: organization '{}' references unknown plan '{}'",
                org.slug, org.plan
            )));
        };

        upsert_organization(pool, org).await?;
        report.organizations_upserted += 1;
        report.grants_projected += project_grants(pool, &repo, &org.slug, &plan.grants).await?;
    }

    tracing::info!(
        plans = report.plans_upserted,
        organizations = report.organizations_upserted,
        grants = report.grants_projected,
        "bootstrap_plans_loaded"
    );
    Ok(report)
}

async fn read_plans(services_path: &Path) -> Result<Option<PlansDoc>, MarketplaceError> {
    let path = services_path.join(PLANS_FILE);
    match tokio::fs::read_to_string(&path).await {
        Ok(s) if s.trim().is_empty() => Ok(Some(PlansDoc::default())),
        Ok(s) => serde_yaml::from_str::<PlansDoc>(&s)
            .map(Some)
            .map_err(|e| MarketplaceError::config_file(PLANS_FILE, e)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

async fn upsert_plan(pool: &PgPool, plan: &YamlPlan) -> Result<(), MarketplaceError> {
    let entry = format!("{PLANS_FILE}: {}", plan.id);
    let grants =
        serde_json::to_value(&plan.grants).map_err(|e| MarketplaceError::config_file(entry, e))?;
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a contract cap in dollars cannot exceed i64 microdollars at any plausible price"
    )]
    let cap = plan
        .monthly_cost_cap_usd
        .map(|usd| (usd * MICRODOLLARS_PER_USD) as i64);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a licence price in dollars cannot exceed i64 microdollars at any plausible price"
    )]
    let price = plan
        .monthly_price_usd
        .map_or(0, |usd| (usd * MICRODOLLARS_PER_USD) as i64);

    sqlx::query!(
        "INSERT INTO plans (id, name, description, seat_limit, monthly_cost_cap_microdollars, monthly_price_microdollars, grants)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (id) DO UPDATE
            SET name = EXCLUDED.name,
                description = EXCLUDED.description,
                seat_limit = EXCLUDED.seat_limit,
                monthly_cost_cap_microdollars = EXCLUDED.monthly_cost_cap_microdollars,
                monthly_price_microdollars = EXCLUDED.monthly_price_microdollars,
                grants = EXCLUDED.grants,
                updated_at = NOW()",
        plan.id,
        plan.name,
        plan.description,
        plan.seat_limit,
        cap,
        price,
        grants,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn upsert_organization(
    pool: &PgPool,
    org: &YamlOrganization,
) -> Result<(), MarketplaceError> {
    sqlx::query!(
        "INSERT INTO organizations (slug, name, plan_id, seat_limit_override, email_domains, status, is_platform)
         VALUES ($1, $2, $3, $4, $5, $6, $7)
         ON CONFLICT (slug) DO UPDATE
            SET name = EXCLUDED.name,
                plan_id = EXCLUDED.plan_id,
                seat_limit_override = EXCLUDED.seat_limit_override,
                email_domains = EXCLUDED.email_domains,
                status = EXCLUDED.status,
                is_platform = EXCLUDED.is_platform,
                updated_at = NOW()",
        org.slug,
        org.name,
        org.plan,
        org.seat_limit_override,
        &org.email_domains,
        org.status,
        org.platform,
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Why: the delete is scoped to `rule_value = <this org's slug>`, so a plan
// downgrade cannot touch another customer's rules or the per-user overrides.
async fn project_grants(
    pool: &PgPool,
    repo: &AccessControlRepository,
    slug: &str,
    grants: &[YamlGrant],
) -> Result<usize, MarketplaceError> {
    let rule_type = organization_rule_type();
    let mut kept_types: Vec<String> = Vec::with_capacity(grants.len());
    let mut kept_ids: Vec<String> = Vec::with_capacity(grants.len());

    for grant in grants {
        let entry = format!("{PLANS_FILE}: {slug}");
        let kind = EntityKind::from_str(&grant.entity_type)
            .map_err(|e| MarketplaceError::config_file(entry.clone(), e))?;
        let access =
            Access::from_str(&grant.access).map_err(|e| MarketplaceError::config_file(entry, e))?;

        // Why: the FK on access_control_rules requires a catalog row. An earlier
        // bootstrap pass will usually have registered it; a plan naming an
        // entity that pass did not see must still resolve, so register it here
        // rather than failing the whole load on ordering.
        repo.upsert_entity(kind, &grant.entity_id, false, SOURCE_LABEL)
            .await
            .map_err(|e| MarketplaceError::Internal(e.to_string()))?;

        repo.upsert_rule(UpsertRuleParams {
            entity_type: kind,
            entity_id: &grant.entity_id,
            rule_type: rule_type.clone(),
            rule_value: slug,
            access,
            justification: Some("granted by plan"),
        })
        .await
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;

        kept_types.push(grant.entity_type.clone());
        kept_ids.push(grant.entity_id.clone());
    }

    sqlx::query!(
        "DELETE FROM access_control_rules
         WHERE rule_type = $1
           AND rule_value = $2
           AND (entity_type, entity_id) NOT IN (
                 SELECT t, i FROM UNNEST($3::TEXT[], $4::TEXT[]) AS kept(t, i)
               )",
        rule_type.as_str(),
        slug,
        &kept_types,
        &kept_ids,
    )
    .execute(pool)
    .await?;

    Ok(grants.len())
}
