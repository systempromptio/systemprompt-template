//! Bootstrap loader: `services/access-control/{groups,projects}.yaml` → DB.
//!
//! Projects a membership gate into `access_control_rules`: one allow-row per
//! (entity, member) listed, with the catalog row forced to
//! `default_included = false`. Runs after the roles pass so it owns the final
//! word on those entities' defaults — an entity in one of these files is
//! reachable through a listed group or project or not at all, whatever a role
//! grant used to say.
//!
//! Both files share this loader because a membership gate has exactly one
//! shape. The only thing that varies is which dimension it writes at, which is
//! why `rule_type` is a parameter rather than a constant.
//!
//! Reconciliation is scoped to the entities each file declares: within one of
//! those the band is pruned to exactly the listed members, and an entity the
//! file does not name keeps whatever the marketplace ingestion or the dashboard
//! wrote at this dimension. Owning the whole band by rule type deleted every
//! `group`/`project` row core had just projected from
//! `services/marketplaces/*/config.yaml`, so the private workspaces booted
//! closed on every start.

use super::member_grants_yaml_types::{MemberGrant, MemberGrantsDoc, MemberGrantsLoadReport};
use crate::authz::group::group_rule_type;
use crate::authz::project::project_rule_type;
use sqlx::PgPool;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use systemprompt_security::authz::{
    AccessControlRepository, EntityKind, RuleType, UpsertRuleParams,
};
use systemprompt_web_shared::error::MarketplaceError;

const GROUPS_FILE: &str = "access-control/groups.yaml";
const PROJECTS_FILE: &str = "access-control/projects.yaml";

pub async fn load_member_grants_from_yaml(
    pool: &PgPool,
    services_path: &Path,
) -> Result<MemberGrantsLoadReport, MarketplaceError> {
    let mut report = MemberGrantsLoadReport::default();
    report.grants_projected +=
        load_one(pool, services_path, GROUPS_FILE, group_rule_type()).await?;
    report.grants_projected +=
        load_one(pool, services_path, PROJECTS_FILE, project_rule_type()).await?;
    tracing::info!(
        grants = report.grants_projected,
        "bootstrap_member_grants_loaded"
    );
    Ok(report)
}

pub async fn load_one(
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
) -> Result<Option<MemberGrantsDoc>, MarketplaceError> {
    let path = services_path.join(file);
    match tokio::fs::read_to_string(&path).await {
        Ok(s) if s.trim().is_empty() => Ok(Some(MemberGrantsDoc::default())),
        Ok(s) => serde_yaml::from_str::<MemberGrantsDoc>(&s)
            .map(Some)
            .map_err(|e| MarketplaceError::config_file(file, e)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

async fn project_grants(
    pool: &PgPool,
    repo: &AccessControlRepository,
    grants: &[MemberGrant],
    file: &str,
    rule_type: &RuleType,
) -> Result<usize, MarketplaceError> {
    let mut kept_types: Vec<String> = Vec::new();
    let mut kept_ids: Vec<String> = Vec::new();
    let mut kept_groups: Vec<String> = Vec::new();
    let mut declared_types: Vec<String> = Vec::new();
    let mut declared_ids: Vec<String> = Vec::new();
    let mut projected = 0;

    for grant in grants {
        let entry = format!("{file}: {}", grant.entity_id);
        let kind = EntityKind::from_str(&grant.entity_type)
            .map_err(|e| MarketplaceError::config_file(entry.clone(), e))?;
        if grant.members.is_empty() {
            return Err(MarketplaceError::config_file(
                entry,
                "a grant must name at least one member",
            ));
        }
        // Why: `default_included = false` is load-bearing here — a user outside
        // every listed member matches no rule at this dimension and falls
        // through to the entity default, which must therefore be closed.
        repo.upsert_entity(kind, &grant.entity_id, false, file)
            .await
            .map_err(|e| MarketplaceError::Internal(e.to_string()))?;

        declared_types.push(grant.entity_type.clone());
        declared_ids.push(grant.entity_id.clone());

        for member in &grant.members {
            repo.upsert_rule(UpsertRuleParams {
                entity_type: kind,
                entity_id: &grant.entity_id,
                rule_type: rule_type.clone(),
                rule_value: member,
                access: grant.access,
                justification: Some("configured access for members of a group or project"),
                source: systemprompt_security::authz::YAML_SOURCE,
            })
            .await
            .map_err(|e| MarketplaceError::Internal(e.to_string()))?;
            kept_types.push(grant.entity_type.clone());
            kept_ids.push(grant.entity_id.clone());
            kept_groups.push(member.clone());
            projected += 1;
        }
    }

    sqlx::query!(
        "DELETE FROM access_control_rules
         WHERE rule_type = $1
           AND (entity_type, entity_id) IN (
                 SELECT t, i FROM UNNEST($5::TEXT[], $6::TEXT[]) AS declared(t, i)
               )
           AND (entity_type, entity_id, rule_value) NOT IN (
                 SELECT t, i, g FROM UNNEST($2::TEXT[], $3::TEXT[], $4::TEXT[]) AS kept(t, i, g)
               )",
        rule_type.as_str(),
        &kept_types,
        &kept_ids,
        &kept_groups,
        &declared_types,
        &declared_ids,
    )
    .execute(pool)
    .await?;

    Ok(projected)
}
