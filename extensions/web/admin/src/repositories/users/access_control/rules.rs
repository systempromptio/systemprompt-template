//! CRUD over `access_control_rules`: listing grants and replacing them
//! transactionally for one or many entities.
//!
//! The entity catalog is core's table: the row a grant's FK needs is ensured
//! through [`AccessControlRepository`], never written here.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::security::authz::{AccessControlRepository, AuthzError, EntityKind};

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
    let entity_type = entity_type.as_str();
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "DELETE FROM access_control_rules WHERE entity_type = $1 AND entity_id = $2",
        entity_type,
        entity_id
    )
    .execute(&mut *tx)
    .await?;

    let mut results = Vec::new();
    for rule in rules {
        let id = uuid::Uuid::new_v4().to_string();
        let rule_type_str = rule.rule_type.to_string();
        let access_str = rule.access.to_string();
        let row = sqlx::query_as!(
            AccessControlRule,
            r#"INSERT INTO access_control_rules (id, entity_type, entity_id, rule_type, rule_value, access)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, entity_type, entity_id,
                         rule_type as "rule_type!: RuleType",
                         rule_value,
                         access as "access!: AccessDecision",
                         created_at, updated_at"#,
            id,
            entity_type,
            entity_id,
            rule_type_str,
            rule.rule_value,
            access_str,
        )
        .fetch_one(&mut *tx)
        .await?;
        results.push(row);
    }

    tx.commit().await?;
    Ok(results)
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
    let mut count = 0usize;

    for (entity_type, entity_id) in entities {
        let entity_type = entity_type.as_str();
        sqlx::query!(
            "DELETE FROM access_control_rules WHERE entity_type = $1 AND entity_id = $2",
            entity_type,
            entity_id
        )
        .execute(&mut *tx)
        .await?;

        for rule in rules {
            let id = uuid::Uuid::new_v4().to_string();
            let rule_type_str = rule.rule_type.to_string();
            let access_str = rule.access.to_string();
            sqlx::query!(
                r"INSERT INTO access_control_rules (id, entity_type, entity_id, rule_type, rule_value, access)
                  VALUES ($1, $2, $3, $4, $5, $6)",
                id,
                entity_type,
                entity_id,
                rule_type_str,
                rule.rule_value,
                access_str,
            )
            .execute(&mut *tx)
            .await?;
        }
        count += 1;
    }

    tx.commit().await?;
    Ok(count)
}

fn catalog(pool: &PgPool) -> AccessControlRepository {
    AccessControlRepository::from_pool(Arc::new(pool.clone()))
}
