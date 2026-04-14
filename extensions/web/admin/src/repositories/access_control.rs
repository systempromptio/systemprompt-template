use sqlx::PgPool;

use super::super::types::access_control::{AccessControlRule, AccessControlRuleInput};

pub async fn list_all_rules(pool: &PgPool) -> Result<Vec<AccessControlRule>, sqlx::Error> {
    sqlx::query_as::<_, AccessControlRule>(
        "SELECT id, entity_type, entity_id, rule_type, rule_value, access, default_included, created_at, updated_at
         FROM access_control_rules
         ORDER BY entity_type, entity_id, rule_type, rule_value",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_rules_for_entity(
    pool: &PgPool,
    entity_type: &str,
    entity_id: &str,
) -> Result<Vec<AccessControlRule>, sqlx::Error> {
    sqlx::query_as::<_, AccessControlRule>(
        "SELECT id, entity_type, entity_id, rule_type, rule_value, access, default_included, created_at, updated_at
         FROM access_control_rules
         WHERE entity_type = $1 AND entity_id = $2
         ORDER BY rule_type, rule_value",
    )
    .bind(entity_type)
    .bind(entity_id)
    .fetch_all(pool)
    .await
}

pub async fn set_entity_rules(
    pool: &PgPool,
    entity_type: &str,
    entity_id: &str,
    rules: &[AccessControlRuleInput],
) -> Result<Vec<AccessControlRule>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query!("DELETE FROM access_control_rules WHERE entity_type = $1 AND entity_id = $2", entity_type, entity_id)
        .execute(&mut *tx)
        .await?;

    let mut results = Vec::new();
    for rule in rules {
        let id = uuid::Uuid::new_v4().to_string();
        let rule_type_str = rule.rule_type.to_string();
        let access_str = rule.access.to_string();
        let row = sqlx::query_as::<_, AccessControlRule>(
            r"INSERT INTO access_control_rules (id, entity_type, entity_id, rule_type, rule_value, access, default_included)
              VALUES ($1, $2, $3, $4, $5, $6, $7)
              RETURNING id, entity_type, entity_id, rule_type, rule_value, access, default_included, created_at, updated_at",
        )
        .bind(&id)
        .bind(entity_type)
        .bind(entity_id)
        .bind(&rule_type_str)
        .bind(&rule.rule_value)
        .bind(&access_str)
        .bind(rule.default_included)
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
        sqlx::query!("DELETE FROM access_control_rules WHERE entity_type = $1 AND entity_id = $2", entity_type, entity_id)
            .execute(&mut *tx)
            .await?;

        for rule in rules {
            let id = uuid::Uuid::new_v4().to_string();
            let rule_type_str = rule.rule_type.to_string();
            let access_str = rule.access.to_string();
            sqlx::query!(
                r"INSERT INTO access_control_rules (id, entity_type, entity_id, rule_type, rule_value, access, default_included)
                  VALUES ($1, $2, $3, $4, $5, $6, $7)",
                id,
                entity_type,
                entity_id,
                rule_type_str,
                rule.rule_value,
                access_str,
                rule.default_included,
            )
            .execute(&mut *tx)
            .await?;
        }
        count += 1;
    }

    tx.commit().await?;
    Ok(count)
}
