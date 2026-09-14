//! Which marketplaces a group is entitled to.
//!
//! Entitlement is not a table of its own: it is an ordinary access-control
//! rule on the marketplace entity, keyed by the `group` subject dimension, so
//! the resolver that decides every other entity decides this one too. The
//! functions here are a projection of those rules onto the one question the
//! dashboard asks — which marketplaces this group may see.
//!
//! `default_included` is never touched. It is the marketplace's own answer
//! about the estate, set in its YAML, and flipping it from a group screen
//! would silently re-entitle every other group.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::identifiers::MarketplaceId;
use systemprompt_security::authz::{
    Access, AccessControlRepository, EntityKind, RuleType, UpsertRuleParams,
};
use systemprompt_web_shared::GroupId;

use crate::error::{AdminError, AdminResult};

const GROUP_RULE_TYPE: &str = "group";
const JUSTIFICATION: &str = "admin:groups";

fn group_rule_type() -> AdminResult<RuleType> {
    RuleType::extension(GROUP_RULE_TYPE).map_err(AdminError::from)
}

pub async fn list_group_marketplace_ids(
    pool: &PgPool,
    group_id: &GroupId,
) -> Result<Vec<MarketplaceId>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT entity_id AS "entity_id: MarketplaceId" FROM access_control_rules
         WHERE entity_type = 'marketplace' AND rule_type = $1 AND rule_value = $2
           AND access = 'allow' ORDER BY entity_id"#,
        GROUP_RULE_TYPE,
        group_id.as_str()
    )
    .fetch_all(pool)
    .await
}

// Why: the caller sends the whole set it wants, so this reconciles rather
// than diffs — listed marketplaces are upserted to allow, and this group's
// rules on unlisted ones are deleted. Keeping one would leave an entitlement
// the screen shows as absent.
pub async fn set_group_marketplaces(
    pool: &PgPool,
    group_id: &GroupId,
    marketplace_ids: &[MarketplaceId],
) -> AdminResult<Vec<MarketplaceId>> {
    let rule_type = group_rule_type()?;
    let repo = AccessControlRepository::from_pool(Arc::new(pool.clone()));

    for marketplace_id in marketplace_ids {
        // Why: rules FK onto the entity catalogue, so a marketplace nobody
        // has touched yet has no row to hang a grant on. `ensure_entity`
        // inserts one at `default_included = false` and never overwrites an
        // existing flag, so it anchors the grant without widening anything.
        repo.ensure_entity(
            EntityKind::Marketplace,
            marketplace_id.as_str(),
            JUSTIFICATION,
        )
        .await
        .map_err(AdminError::internal)?;
        repo.upsert_rule(UpsertRuleParams {
            entity_type: EntityKind::Marketplace,
            entity_id: marketplace_id.as_str(),
            rule_type: rule_type.clone(),
            rule_value: group_id.as_str(),
            access: Access::Allow,
            justification: Some(JUSTIFICATION),
            source: systemprompt_security::authz::DASHBOARD_SOURCE,
        })
        .await
        .map_err(AdminError::internal)?;
    }

    sqlx::query!(
        "DELETE FROM access_control_rules
         WHERE entity_type = 'marketplace' AND rule_type = $1 AND rule_value = $2
           AND NOT (entity_id = ANY($3))",
        GROUP_RULE_TYPE,
        group_id.as_str(),
        &marketplace_ids
            .iter()
            .map(|id| id.as_str().to_owned())
            .collect::<Vec<_>>()
    )
    .execute(pool)
    .await?;

    list_group_marketplace_ids(pool, group_id)
        .await
        .map_err(AdminError::from)
}
