//! Cross-user aggregates used by the user-management views: marketplace
//! overrides and per-user skill / device counts keyed by department.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserManagementAggregate {
    pub user_id: UserId,
    pub department: String,
    pub assigned_skills_count: i64,
    pub devices_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserMarketplaceOverride {
    pub user_id: UserId,
    pub department: String,
    // Why: polymorphic entity reference (gateway_route/mcp_server), no single typed-ID equivalent
    pub entity_id: String,
    pub access: String,
}

/// A user receives overrides from rules matching either their own id or their
/// department, so the same entity can appear under both scopes.
pub async fn list_user_marketplace_overrides(
    pool: &PgPool,
) -> Result<Vec<UserMarketplaceOverride>, sqlx::Error> {
    sqlx::query_as!(
        UserMarketplaceOverride,
        r#"
        SELECT
            u.id AS "user_id!: UserId",
            COALESCE(upe.department, '') AS "department!",
            acr.entity_id,
            acr.access
        FROM users u
        LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        JOIN access_control_rules acr
          ON acr.entity_type = 'marketplace'
         AND ((acr.rule_type = 'user' AND acr.rule_value = u.id)
              OR (acr.rule_type = 'department' AND acr.rule_value = COALESCE(upe.department, '')))
        WHERE NOT ('anonymous' = ANY(u.roles))
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn list_user_management_aggregates(
    pool: &PgPool,
) -> Result<Vec<UserManagementAggregate>, sqlx::Error> {
    sqlx::query_as!(
        UserManagementAggregate,
        r#"
        SELECT
            u.id AS "user_id!: UserId",
            COALESCE(upe.department, '') AS "department!",
            COALESCE((
                SELECT COUNT(DISTINCT acr.entity_id)
                FROM access_control_rules acr
                WHERE acr.entity_type = 'skill'
                  AND acr.access = 'allow'
                  AND ((acr.rule_type = 'department' AND acr.rule_value = COALESCE(upe.department, ''))
                       OR (acr.rule_type = 'user' AND acr.rule_value = u.id))
            ), 0)::BIGINT AS "assigned_skills_count!",
            COALESCE((
                SELECT COUNT(*) FROM user_api_keys
                WHERE user_id = u.id AND revoked_at IS NULL
            ), 0)::BIGINT AS "devices_count!",
            u.created_at
        FROM users u
        LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        WHERE NOT ('anonymous' = ANY(u.roles))
        "#,
    )
    .fetch_all(pool)
    .await
}
