use std::sync::Arc;

use sqlx::PgPool;

use super::super::types::{
    PluginRating, PluginRatingAggregate, PluginUsageAggregate, PluginUser, VisibilityRule,
    VisibilityRuleInput,
};

pub async fn get_all_plugin_usage(
    pool: &Arc<PgPool>,
) -> Result<Vec<PluginUsageAggregate>, sqlx::Error> {
    sqlx::query_as::<_, PluginUsageAggregate>(
        r"SELECT
            p.plugin_id,
            COUNT(*)::BIGINT AS total_events,
            COUNT(DISTINCT p.user_id)::BIGINT AS unique_users,
            COUNT(DISTINCT p.user_id) FILTER (
                WHERE p.created_at >= NOW() - INTERVAL '7 days'
            )::BIGINT AS active_users_7d,
            COUNT(DISTINCT p.user_id) FILTER (
                WHERE p.created_at >= NOW() - INTERVAL '30 days'
            )::BIGINT AS active_users_30d
        FROM plugin_usage_events p
        JOIN users u ON u.id = p.user_id
        WHERE p.plugin_id IS NOT NULL
          AND NOT ('anonymous' = ANY(u.roles))
          AND u.email NOT LIKE '%@anonymous.local'
        GROUP BY p.plugin_id",
    )
    .fetch_all(pool.as_ref())
    .await
}

pub async fn get_plugin_users(
    pool: &Arc<PgPool>,
    plugin_id: &str,
) -> Result<Vec<PluginUser>, sqlx::Error> {
    sqlx::query_as::<_, PluginUser>(
        r"SELECT
            p.user_id,
            COALESCE(u.display_name, u.full_name, u.name, u.email, p.user_id) AS display_name,
            u.email,
            u.department,
            COUNT(*)::BIGINT AS event_count,
            MAX(p.created_at) AS last_used
        FROM plugin_usage_events p
        JOIN users u ON u.id = p.user_id
        WHERE p.plugin_id = $1
          AND NOT ('anonymous' = ANY(u.roles))
          AND u.email NOT LIKE '%@anonymous.local'
        GROUP BY p.user_id, u.display_name, u.full_name, u.name, u.email, u.department
        ORDER BY event_count DESC
        LIMIT 50",
    )
    .bind(plugin_id)
    .fetch_all(pool.as_ref())
    .await
}

pub async fn get_all_plugin_ratings(
    pool: &Arc<PgPool>,
) -> Result<Vec<PluginRatingAggregate>, sqlx::Error> {
    sqlx::query_as::<_, PluginRatingAggregate>(
        r"SELECT
            plugin_id,
            COALESCE(AVG(rating)::FLOAT8, 0.0) AS avg_rating,
            COUNT(*)::BIGINT AS rating_count
        FROM plugin_ratings
        GROUP BY plugin_id",
    )
    .fetch_all(pool.as_ref())
    .await
}

pub async fn get_all_visibility_rules(
    pool: &Arc<PgPool>,
) -> Result<Vec<VisibilityRule>, sqlx::Error> {
    sqlx::query_as::<_, VisibilityRule>(
        "SELECT id, plugin_id, rule_type, rule_value, access, created_at
         FROM plugin_visibility_rules
         ORDER BY plugin_id, rule_type, rule_value",
    )
    .fetch_all(pool.as_ref())
    .await
}

pub async fn upsert_rating(
    pool: &Arc<PgPool>,
    plugin_id: &str,
    user_id: &str,
    rating: i16,
    review: Option<&str>,
) -> Result<PluginRating, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query_as::<_, PluginRating>(
        r"INSERT INTO plugin_ratings (id, plugin_id, user_id, rating, review)
          VALUES ($1, $2, $3, $4, $5)
          ON CONFLICT (plugin_id, user_id) DO UPDATE
            SET rating = EXCLUDED.rating,
                review = EXCLUDED.review,
                updated_at = NOW()
          RETURNING id, plugin_id, user_id, rating, review, created_at, updated_at",
    )
    .bind(&id)
    .bind(plugin_id)
    .bind(user_id)
    .bind(rating)
    .bind(review)
    .fetch_one(pool.as_ref())
    .await
}

pub async fn set_visibility_rules(
    pool: &Arc<PgPool>,
    plugin_id: &str,
    rules: &[VisibilityRuleInput],
) -> Result<Vec<VisibilityRule>, sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query("DELETE FROM plugin_visibility_rules WHERE plugin_id = $1")
        .bind(plugin_id)
        .execute(&mut *tx)
        .await?;

    let mut results = Vec::new();
    for rule in rules {
        let id = uuid::Uuid::new_v4().to_string();
        let row = sqlx::query_as::<_, VisibilityRule>(
            r"INSERT INTO plugin_visibility_rules (id, plugin_id, rule_type, rule_value, access)
              VALUES ($1, $2, $3, $4, $5)
              RETURNING id, plugin_id, rule_type, rule_value, access, created_at",
        )
        .bind(&id)
        .bind(plugin_id)
        .bind(&rule.rule_type)
        .bind(&rule.rule_value)
        .bind(&rule.access)
        .fetch_one(&mut *tx)
        .await?;
        results.push(row);
    }

    tx.commit().await?;
    Ok(results)
}
