//! CRUD over `access_control_rules`: listing grants and replacing them
//! transactionally for one or many entities.

use sqlx::PgPool;

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

/// Replace every grant on `(entity_type, entity_id)` with `rules`.
///
/// Ensures the entity catalog row exists before inserting grants — the FK
/// added in core migration 007 rejects orphan rules.
pub async fn set_entity_rules(
    pool: &PgPool,
    entity_type: &str,
    entity_id: &str,
    rules: &[AccessControlRuleInput],
) -> Result<Vec<AccessControlRule>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "INSERT INTO access_control_entities (entity_type, entity_id, default_included, source)
         VALUES ($1, $2, false, 'admin:dashboard')
         ON CONFLICT (entity_type, entity_id) DO NOTHING",
        entity_type,
        entity_id,
    )
    .execute(&mut *tx)
    .await?;

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
    entities: &[(String, String)],
    rules: &[AccessControlRuleInput],
) -> Result<usize, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let mut count = 0usize;

    for (entity_type, entity_id) in entities {
        sqlx::query!(
            "INSERT INTO access_control_entities (entity_type, entity_id, default_included, source)
             VALUES ($1, $2, false, 'admin:dashboard')
             ON CONFLICT (entity_type, entity_id) DO NOTHING",
            entity_type,
            entity_id,
        )
        .execute(&mut *tx)
        .await?;

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
