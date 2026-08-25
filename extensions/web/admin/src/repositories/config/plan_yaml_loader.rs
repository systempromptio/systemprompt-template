//! Bootstrap loader: `services/access-control/plans.yaml` → DB.
//!
//! Runs inside the same governance bootstrap pass as the roles and departments
//! files, whose earlier passes materialise the catalog rows a plan's grants
//! reference.
//!
//! The load is staged so nothing is written until the whole document is known
//! to be coherent: [`validate`] parses every reference in the file,
//! [`resolve_catalog`] proves each granted entity already has a catalog row,
//! and only then does [`persist`] upsert plans and organizations and project
//! each organization's plan grants into `access_control_rules` at
//! `rule_type = 'organization'`. Each stage consumes its predecessor's output,
//! so the ordering is carried by the types rather than by call-site
//! discipline. A grant naming an entity no earlier pass registered is a typo
//! in the YAML and fails the load; it does not mint a phantom catalog row.
//!
//! Projection is the whole point — a plan is a named bundle of ordinary rules,
//! so nothing downstream needs to know plans exist. The resolver, the access
//! matrix, and the governance audit all see rule rows they already understand.
//!
//! Like every other bootstrap loader here, the direction is fixed (YAML → DB)
//! and there is no write-back: a customer's own narrowing of their plan lives
//! in the DB and survives redeploys, because projection only ever touches rows
//! whose `rule_value` is an organization slug.

use std::collections::BTreeSet;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use sqlx::PgPool;
use systemprompt_security::authz::{Access, AccessControlRepository, EntityKind, UpsertRuleParams};
use systemprompt_web_shared::error::MarketplaceError;

use crate::authz::organization::organization_rule_type;

use super::plan_yaml_types::{PlanLoadReport, PlansDoc, YamlGrant, YamlOrganization, YamlPlan};

const PLANS_FILE: &str = "access-control/plans.yaml";

const MICRODOLLARS_PER_USD: f64 = 1_000_000.0;

pub async fn load_plans_from_yaml(
    pool: &PgPool,
    services_path: &Path,
) -> Result<PlanLoadReport, MarketplaceError> {
    let Some(doc) = read_plans(services_path).await? else {
        return Ok(PlanLoadReport::default());
    };

    let repo = AccessControlRepository::from_pool(Arc::new(pool.clone()));
    let validated = validate(&doc)?;
    let resolved = resolve_catalog(&repo, validated).await?;
    let report = persist(pool, &repo, resolved).await?;

    tracing::info!(
        plans = report.plans_upserted,
        organizations = report.organizations_upserted,
        grants = report.grants_projected,
        "bootstrap_plans_loaded"
    );
    Ok(report)
}

struct ValidatedGrant<'a> {
    kind: EntityKind,
    access: Access,
    grant: &'a YamlGrant,
}

struct ValidatedOrg<'a> {
    org: &'a YamlOrganization,
    grants: Vec<ValidatedGrant<'a>>,
}

struct ValidatedDoc<'a> {
    plans: &'a [YamlPlan],
    organizations: Vec<ValidatedOrg<'a>>,
    referenced: BTreeSet<(EntityKind, &'a str)>,
}

struct ResolvedDoc<'a>(ValidatedDoc<'a>);

fn validate(doc: &PlansDoc) -> Result<ValidatedDoc<'_>, MarketplaceError> {
    let mut referenced = BTreeSet::new();
    let mut organizations = Vec::with_capacity(doc.organizations.len());

    for org in &doc.organizations {
        let Some(plan) = doc.plans.iter().find(|p| p.id == org.plan) else {
            return Err(MarketplaceError::Internal(format!(
                "{PLANS_FILE}: organization '{}' references unknown plan '{}'",
                org.slug, org.plan
            )));
        };

        let mut grants = Vec::with_capacity(plan.grants.len());
        for grant in &plan.grants {
            let entry = format!("{PLANS_FILE}: {}", org.slug);
            let kind = EntityKind::from_str(&grant.entity_type)
                .map_err(|e| MarketplaceError::config_file(entry.clone(), e))?;
            let access = Access::from_str(&grant.access)
                .map_err(|e| MarketplaceError::config_file(entry, e))?;
            referenced.insert((kind, grant.entity_id.as_str()));
            grants.push(ValidatedGrant {
                kind,
                access,
                grant,
            });
        }
        organizations.push(ValidatedOrg { org, grants });
    }

    Ok(ValidatedDoc {
        plans: &doc.plans,
        organizations,
        referenced,
    })
}

async fn resolve_catalog<'a>(
    repo: &AccessControlRepository,
    doc: ValidatedDoc<'a>,
) -> Result<ResolvedDoc<'a>, MarketplaceError> {
    for (kind, entity_id) in &doc.referenced {
        let known = repo
            .get_entity(*kind, entity_id)
            .await
            .map_err(|e| MarketplaceError::Internal(e.to_string()))?;
        if known.is_none() {
            return Err(MarketplaceError::Internal(format!(
                "{PLANS_FILE}: grant references {kind} '{entity_id}', which no catalog row \
                 registers — check the id, or the bootstrap pass that should have loaded it",
                kind = kind.as_str(),
            )));
        }
    }
    Ok(ResolvedDoc(doc))
}

async fn persist(
    pool: &PgPool,
    repo: &AccessControlRepository,
    ResolvedDoc(doc): ResolvedDoc<'_>,
) -> Result<PlanLoadReport, MarketplaceError> {
    let mut report = PlanLoadReport::default();

    for plan in doc.plans {
        upsert_plan(pool, plan).await?;
        report.plans_upserted += 1;
    }

    for entry in &doc.organizations {
        upsert_organization(pool, entry.org).await?;
        report.organizations_upserted += 1;
        report.grants_projected +=
            project_grants(pool, repo, &entry.org.slug, &entry.grants).await?;
    }

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
    let grants = serde_json::to_value(&plan.grants)
        .map_err(|e| MarketplaceError::config_file(entry.clone(), e))?;
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
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a warning threshold in dollars cannot exceed i64 microdollars at any plausible price"
    )]
    let warn = plan
        .monthly_cost_warn_usd
        .map(|usd| (usd * MICRODOLLARS_PER_USD) as i64);

    if let Some(warn_usd) = plan.monthly_cost_warn_usd {
        let below_cap = plan.monthly_cost_cap_usd.is_some_and(|cap| warn_usd < cap);
        if !below_cap {
            return Err(MarketplaceError::Internal(format!(
                "{entry}: monthly_cost_warn_usd must be set below monthly_cost_cap_usd — a \
                 warning threshold without a cap beneath it has nothing to warn about"
            )));
        }
    }

    sqlx::query!(
        "INSERT INTO plans (id, name, description, seat_limit, monthly_cost_cap_microdollars, monthly_cost_warn_microdollars, monthly_price_microdollars, grants)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (id) DO UPDATE
            SET name = EXCLUDED.name,
                description = EXCLUDED.description,
                seat_limit = EXCLUDED.seat_limit,
                monthly_cost_cap_microdollars = EXCLUDED.monthly_cost_cap_microdollars,
                monthly_cost_warn_microdollars = EXCLUDED.monthly_cost_warn_microdollars,
                monthly_price_microdollars = EXCLUDED.monthly_price_microdollars,
                grants = EXCLUDED.grants,
                updated_at = NOW()",
        plan.id,
        plan.name,
        plan.description,
        plan.seat_limit,
        cap,
        warn,
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
    grants: &[ValidatedGrant<'_>],
) -> Result<usize, MarketplaceError> {
    let rule_type = organization_rule_type();
    let mut kept_types: Vec<String> = Vec::with_capacity(grants.len());
    let mut kept_ids: Vec<String> = Vec::with_capacity(grants.len());

    for entry in grants {
        repo.upsert_rule(UpsertRuleParams {
            entity_type: entry.kind,
            entity_id: &entry.grant.entity_id,
            rule_type: rule_type.clone(),
            rule_value: slug,
            access: entry.access,
            justification: Some("granted by plan"),
        })
        .await
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;

        kept_types.push(entry.grant.entity_type.clone());
        kept_ids.push(entry.grant.entity_id.clone());
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
