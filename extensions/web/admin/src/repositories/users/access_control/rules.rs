//! CRUD over `access_control_rules`: listing grants and replacing them
//! transactionally for one or many entities.
//!
//! The entity catalog is core's table: the row a grant's FK needs is ensured
//! through [`AccessControlRepository`], never written here.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::security::authz::{
    AccessControlRepository, AuthzError, DASHBOARD_SOURCE, EntityKind,
};

use crate::types::access_control::{
    AccessControlRule, AccessControlRuleInput, AccessDecision, RuleType,
};

pub async fn list_all_rules(pool: &PgPool) -> Result<Vec<AccessControlRule>, sqlx::Error> {
    sqlx::query_as!(
        AccessControlRule,
        r#"SELECT id, entity_type, entity_id,
                  rule_type as "rule_type!: RuleType",
                  rule_value,
                  access as "access!: AccessDecision",
                  created_at, updated_at
           FROM access_control_rules
           ORDER BY entity_type, entity_id, rule_type, rule_value"#,
    )
    .fetch_all(pool)
    .await
}

pub async fn count_assignments_by_entity_type(
    pool: &PgPool,
    entity_type: &str,
) -> Result<std::collections::HashMap<String, i64>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT entity_id, COUNT(*)::BIGINT AS "count!"
           FROM access_control_rules
           WHERE entity_type = $1
           GROUP BY entity_id"#,
        entity_type,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| (r.entity_id, r.count)).collect())
}

pub async fn list_rules_for_entity(
    pool: &PgPool,
    entity_type: &str,
    entity_id: &str,
) -> Result<Vec<AccessControlRule>, sqlx::Error> {
    sqlx::query_as!(
        AccessControlRule,
        r#"SELECT id, entity_type, entity_id,
                  rule_type as "rule_type!: RuleType",
                  rule_value,
                  access as "access!: AccessDecision",
                  created_at, updated_at
           FROM access_control_rules
           WHERE entity_type = $1 AND entity_id = $2
           ORDER BY rule_type, rule_value"#,
        entity_type,
        entity_id,
    )
    .fetch_all(pool)
    .await
}

const SOURCE_LABEL: &str = "admin:dashboard";

// Why: replaces the entity's rule set with `rules`, but as an upsert plus a
// delete of what is absent rather than a wipe and re-insert. A row the file
// already declared and the console left alone keeps its `yaml` source and its
// justification, so it does not turn into drift; only rows the console changed
// or added are stamped `dashboard`.
pub async fn set_entity_rules(
    pool: &PgPool,
    entity_type: EntityKind,
    entity_id: &str,
    rules: &[AccessControlRuleInput],
) -> Result<Vec<AccessControlRule>, AuthzError> {
    // Why: core's `ensure_entity` takes the pool, not this transaction, so the
    // catalog row is committed before the rules are replaced. A failure between
    // the two leaves a catalog row with no grants — an entity that exists and
    // grants nothing, which is what an unruled entity already means here.
    catalog(pool)
        .ensure_entity(entity_type, entity_id, SOURCE_LABEL)
        .await?;
    let mut tx = pool.begin().await?;
    replace_rules(&mut tx, entity_type.as_str(), entity_id, rules).await?;
    tx.commit().await?;
    list_rules_for_entity(pool, entity_type.as_str(), entity_id)
        .await
        .map_err(AuthzError::from)
}

pub async fn bulk_set_rules(
    pool: &PgPool,
    entities: &[(EntityKind, String)],
    rules: &[AccessControlRuleInput],
) -> Result<usize, AuthzError> {
    let catalog = catalog(pool);
    for (entity_type, entity_id) in entities {
        catalog
            .ensure_entity(*entity_type, entity_id, SOURCE_LABEL)
            .await?;
    }
    let mut tx = pool.begin().await?;
    for (entity_type, entity_id) in entities {
        replace_rules(&mut tx, entity_type.as_str(), entity_id, rules).await?;
    }
    tx.commit().await?;
    Ok(entities.len())
}

async fn replace_rules(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    entity_type: &str,
    entity_id: &str,
    rules: &[AccessControlRuleInput],
) -> Result<(), sqlx::Error> {
    let mut kept_types: Vec<String> = Vec::with_capacity(rules.len());
    let mut kept_values: Vec<String> = Vec::with_capacity(rules.len());
    for rule in rules {
        let id = uuid::Uuid::new_v4().to_string();
        let rule_type_str = rule.rule_type.to_string();
        let access_str = rule.access.to_string();
        sqlx::query!(
            r"INSERT INTO access_control_rules
                  (id, entity_type, entity_id, rule_type, rule_value, access, justification, source)
              VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
              ON CONFLICT (entity_type, entity_id, rule_type, rule_value) DO UPDATE
                 SET access = EXCLUDED.access,
                     justification = COALESCE(EXCLUDED.justification, access_control_rules.justification),
                     source = CASE
                         WHEN access_control_rules.access = EXCLUDED.access
                          AND EXCLUDED.justification IS NULL THEN access_control_rules.source
                         ELSE EXCLUDED.source
                     END,
                     updated_at = CASE
                         WHEN access_control_rules.access = EXCLUDED.access
                          AND EXCLUDED.justification IS NULL THEN access_control_rules.updated_at
                         ELSE NOW()
                     END",
            id,
            entity_type,
            entity_id,
            rule_type_str,
            rule.rule_value,
            access_str,
            rule.justification.as_deref().map(str::trim).filter(|j| !j.is_empty()),
            DASHBOARD_SOURCE,
        )
        .execute(&mut **tx)
        .await?;
        kept_types.push(rule_type_str);
        kept_values.push(rule.rule_value.clone());
    }
    sqlx::query!(
        r"DELETE FROM access_control_rules
           WHERE entity_type = $1 AND entity_id = $2
             AND (rule_type, rule_value) NOT IN (
                   SELECT t, v FROM UNNEST($3::TEXT[], $4::TEXT[]) AS kept(t, v)
                 )",
        entity_type,
        entity_id,
        &kept_types,
        &kept_values,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

fn catalog(pool: &PgPool) -> AccessControlRepository {
    AccessControlRepository::from_pool(Arc::new(pool.clone()))
}
