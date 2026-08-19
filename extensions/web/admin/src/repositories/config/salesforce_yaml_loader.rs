//! Bootstrap loader: `services/access-control/salesforce.yaml` → DB.
//!
//! Projects the Salesforce gate into `access_control_rules`: one
//! `rule_type = 'salesforce'`, `rule_value = 'linked'` allow-row per listed
//! entity, with the catalog row forced to `default_included = false`. Runs
//! after the roles pass so it owns the final word on those entities' defaults:
//! an entity in this file is reachable through the `linked` attribute or not
//! at all, whatever a role grant used to say.
//!
//! Like the plan loader, reconciliation is scoped to this loader's own rows
//! (`rule_value = 'linked'`), so dashboard edits at other rule types survive —
//! with one deliberate exception: the pre-gate bootstrap rule that granted
//! `mcp_server:salesforce` to every `user` role is deleted, because roles.yaml
//! ingestion never deletes orphans and leaving it would hand the server back
//! to the role band on installs that predate the gate.

use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use sqlx::PgPool;
use systemprompt_security::authz::{Access, AccessControlRepository, EntityKind, UpsertRuleParams};
use systemprompt_web_shared::error::MarketplaceError;

use crate::authz::salesforce::{SALESFORCE_LINKED_VALUE, salesforce_rule_type};

use super::salesforce_yaml_types::{SalesforceDoc, SalesforceGrant, SalesforceLoadReport};

const SALESFORCE_FILE: &str = "access-control/salesforce.yaml";

const SOURCE_LABEL: &str = "salesforce.yaml";

pub async fn load_salesforce_from_yaml(
    pool: &PgPool,
    services_path: &Path,
) -> Result<SalesforceLoadReport, MarketplaceError> {
    let mut report = SalesforceLoadReport::default();
    let Some(doc) = read_salesforce(services_path).await? else {
        return Ok(report);
    };

    let repo = AccessControlRepository::from_pool(Arc::new(pool.clone()));
    report.grants_projected = project_grants(pool, &repo, &doc.grants).await?;
    delete_legacy_role_grant(pool).await?;

    tracing::info!(
        grants = report.grants_projected,
        "bootstrap_salesforce_gate_loaded"
    );
    Ok(report)
}

async fn read_salesforce(services_path: &Path) -> Result<Option<SalesforceDoc>, MarketplaceError> {
    let path = services_path.join(SALESFORCE_FILE);
    match tokio::fs::read_to_string(&path).await {
        Ok(s) if s.trim().is_empty() => Ok(Some(SalesforceDoc::default())),
        Ok(s) => serde_yaml::from_str::<SalesforceDoc>(&s)
            .map(Some)
            .map_err(|e| MarketplaceError::config_file(SALESFORCE_FILE, e)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

async fn project_grants(
    pool: &PgPool,
    repo: &AccessControlRepository,
    grants: &[SalesforceGrant],
) -> Result<usize, MarketplaceError> {
    let rule_type = salesforce_rule_type();
    let mut kept_types: Vec<String> = Vec::with_capacity(grants.len());
    let mut kept_ids: Vec<String> = Vec::with_capacity(grants.len());

    for grant in grants {
        let entry = format!("{SALESFORCE_FILE}: {}", grant.entity_id);
        let kind = EntityKind::from_str(&grant.entity_type)
            .map_err(|e| MarketplaceError::config_file(entry, e))?;

        // Why: `default_included = false` is load-bearing here — an unlinked
        // user matches no `salesforce` rule and falls through to the entity
        // default, which must therefore be closed.
        repo.upsert_entity(kind, &grant.entity_id, false, SOURCE_LABEL)
            .await
            .map_err(|e| MarketplaceError::Internal(e.to_string()))?;

        repo.upsert_rule(UpsertRuleParams {
            entity_type: kind,
            entity_id: &grant.entity_id,
            rule_type: rule_type.clone(),
            rule_value: SALESFORCE_LINKED_VALUE,
            access: Access::Allow,
            justification: Some("granted to Salesforce-linked users"),
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
        SALESFORCE_LINKED_VALUE,
        &kept_types,
        &kept_ids,
    )
    .execute(pool)
    .await?;

    Ok(grants.len())
}

async fn delete_legacy_role_grant(pool: &PgPool) -> Result<(), MarketplaceError> {
    let deleted = sqlx::query!(
        "DELETE FROM access_control_rules
         WHERE entity_type = 'mcp_server'
           AND entity_id = 'salesforce'
           AND rule_type = 'role'
           AND rule_value = 'user'",
    )
    .execute(pool)
    .await?
    .rows_affected();
    if deleted > 0 {
        tracing::info!("removed pre-gate role grant on mcp_server:salesforce");
    }
    Ok(())
}
