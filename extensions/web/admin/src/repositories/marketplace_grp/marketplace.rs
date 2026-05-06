use sqlx::PgPool;

use crate::types::{PluginRatingAggregate, VisibilityRule, VisibilityRuleInput};

pub async fn list_plugin_ratings(pool: &PgPool) -> Result<Vec<PluginRatingAggregate>, sqlx::Error> {
    sqlx::query_as!(
        PluginRatingAggregate,
        r#"SELECT
            plugin_id,
            COALESCE(AVG(rating)::FLOAT8, 0.0) AS "avg_rating!",
            COUNT(*)::BIGINT AS "rating_count!"
        FROM plugin_ratings
        GROUP BY plugin_id"#,
    )
    .fetch_all(pool)
    .await
}

pub async fn list_visibility_rules(pool: &PgPool) -> Result<Vec<VisibilityRule>, sqlx::Error> {
    sqlx::query_as!(
        VisibilityRule,
        "SELECT id, plugin_id, rule_type, rule_value, access, created_at
         FROM plugin_visibility_rules
         ORDER BY plugin_id, rule_type, rule_value",
    )
    .fetch_all(pool)
    .await
}

pub async fn set_visibility_rules(
    pool: &PgPool,
    plugin_id: &str,
    rules: &[VisibilityRuleInput],
) -> Result<Vec<VisibilityRule>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query!(
        "DELETE FROM plugin_visibility_rules WHERE plugin_id = $1",
        plugin_id
    )
    .execute(&mut *tx)
    .await?;

    let mut results = Vec::new();
    for rule in rules {
        let id = uuid::Uuid::new_v4().to_string();
        let row = sqlx::query_as!(
            VisibilityRule,
            r"INSERT INTO plugin_visibility_rules (id, plugin_id, rule_type, rule_value, access)
              VALUES ($1, $2, $3, $4, $5)
              RETURNING id, plugin_id, rule_type, rule_value, access, created_at",
            id,
            plugin_id,
            rule.rule_type,
            rule.rule_value,
            rule.access,
        )
        .fetch_one(&mut *tx)
        .await?;
        results.push(row);
    }

    tx.commit().await?;
    Ok(results)
}
