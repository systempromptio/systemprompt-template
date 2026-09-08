//! Bootstrap loader: `services/access-control/salesforce.yaml` → DB.
//!
//! Projects a downstream-link gate into `access_control_rules`: one allow-row
//! per listed entity at the dimension's rule type with `rule_value = 'linked'`,
//! and the catalog row forced to `default_included = false`. Runs after the
//! roles pass so it owns the final word on those entities' defaults — an entity
//! in one of these files is reachable by a linked user or not at all, whatever
//! a role grant used to say.
//!
//! A link gate has exactly one shape; the only thing that varies is which
//! dimension it writes at, which is why `rule_type` is a parameter rather than
//! a constant, so a second downstream can reuse it.
//!
//! Reconciliation is scoped to the loader's own rows, so dashboard edits at
//! other rule types survive.

use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use sqlx::PgPool;
use systemprompt_security::authz::{
    Access, AccessControlRepository, EntityKind, RuleType, UpsertRuleParams,
};
use systemprompt_web_shared::error::MarketplaceError;

use crate::authz::salesforce::{SALESFORCE_LINKED_VALUE, salesforce_rule_type};

use super::linked_yaml_types::{LinkedGrant, LinkedGrantsDoc, LinkedGrantsLoadReport};

const SALESFORCE_FILE: &str = "access-control/salesforce.yaml";

pub async fn load_link_gates_from_yaml(
    pool: &PgPool,
    services_path: &Path,
) -> Result<LinkedGrantsLoadReport, MarketplaceError> {
    let mut report = LinkedGrantsLoadReport::default();

    report.grants_projected +=
        load_one(pool, services_path, SALESFORCE_FILE, salesforce_rule_type()).await?;

    tracing::info!(
        grants = report.grants_projected,
        "bootstrap_link_gates_loaded"
    );
    Ok(report)
}

async fn load_one(
    pool: &PgPool,
    services_path: &Path,
    file: &str,
    rule_type: RuleType,
) -> Result<usize, MarketplaceError> {
    let Some(doc) = read_doc(services_path, file).await? else {
        return Ok(0);
    };
    let repo = AccessControlRepository::from_pool(Arc::new(pool.clone()));
    project_grants(pool, &repo, &doc.grants, file, &rule_type).await
}

async fn read_doc(
    services_path: &Path,
    file: &str,
) -> Result<Option<LinkedGrantsDoc>, MarketplaceError> {
    let path = services_path.join(file);
    match tokio::fs::read_to_string(&path).await {
        Ok(s) if s.trim().is_empty() => Ok(Some(LinkedGrantsDoc::default())),
        Ok(s) => serde_yaml::from_str::<LinkedGrantsDoc>(&s)
            .map(Some)
            .map_err(|e| MarketplaceError::config_file(file, e)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

async fn project_grants(
    pool: &PgPool,
    repo: &AccessControlRepository,
    grants: &[LinkedGrant],
    file: &str,
    rule_type: &RuleType,
) -> Result<usize, MarketplaceError> {
    let mut kept_types: Vec<String> = Vec::new();
    let mut kept_ids: Vec<String> = Vec::new();
    let mut projected = 0;

    for grant in grants {
        let entry = format!("{file}: {}", grant.entity_id);
        let kind = EntityKind::from_str(&grant.entity_type)
            .map_err(|e| MarketplaceError::config_file(entry, e))?;

        // Why: `default_included = false` is load-bearing — an unlinked user
        // matches no rule at this dimension and falls through to the entity
        // default, which must therefore be closed.
        repo.upsert_entity(kind, &grant.entity_id, false, file)
            .await
            .map_err(|e| MarketplaceError::Internal(e.to_string()))?;

        repo.upsert_rule(UpsertRuleParams {
            entity_type: kind,
            entity_id: &grant.entity_id,
            rule_type: rule_type.clone(),
            rule_value: SALESFORCE_LINKED_VALUE,
            access: Access::Allow,
            justification: Some("granted to users who linked the downstream account"),
        })
        .await
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;

        kept_types.push(grant.entity_type.clone());
        kept_ids.push(grant.entity_id.clone());
        projected += 1;
    }

    sqlx::query!(
        "DELETE FROM access_control_rules
         WHERE rule_type = $1
           AND (entity_type, entity_id) NOT IN (
                 SELECT t, i FROM UNNEST($2::TEXT[], $3::TEXT[]) AS kept(t, i)
               )",
        rule_type.as_str(),
        &kept_types,
        &kept_ids,
    )
    .execute(pool)
    .await?;

    Ok(projected)
}
