use sqlx::PgPool;
use uuid::Uuid;

use crate::types::departments::{
    Department, DepartmentInput, DepartmentMember, DepartmentSummary, DepartmentTopTool,
};

pub async fn list_departments(pool: &PgPool) -> Result<Vec<DepartmentSummary>, sqlx::Error> {
    sqlx::query_as::<_, DepartmentSummary>(
        r"
        SELECT
            d.id,
            d.name,
            d.description,
            COALESCE(mc.member_count, 0)::BIGINT  AS member_count,
            COALESCE(ac.assignment_count, 0)::BIGINT AS assignment_count,
            COALESCE(usg.input_tokens, 0)::BIGINT  AS input_tokens,
            COALESCE(usg.output_tokens, 0)::BIGINT AS output_tokens,
            COALESCE(usg.requests, 0)::BIGINT      AS requests,
            COALESCE(usg.cost_microdollars, 0)::BIGINT AS cost_microdollars,
            d.created_at,
            d.updated_at
        FROM departments d
        LEFT JOIN (
            SELECT department, COUNT(*)::BIGINT AS member_count
            FROM user_profile_ext
            WHERE department IS NOT NULL AND department <> ''
            GROUP BY department
        ) mc ON mc.department = d.name
        LEFT JOIN (
            SELECT rule_value, COUNT(*)::BIGINT AS assignment_count
            FROM access_control_rules
            WHERE rule_type = 'department'
            GROUP BY rule_value
        ) ac ON ac.rule_value = d.name
        LEFT JOIN (
            SELECT
                upe.department AS dept,
                COALESCE(SUM(ar.input_tokens), 0)::BIGINT  AS input_tokens,
                COALESCE(SUM(ar.output_tokens), 0)::BIGINT AS output_tokens,
                COUNT(ar.id)::BIGINT                       AS requests,
                COALESCE(SUM(ar.cost_microdollars), 0)::BIGINT AS cost_microdollars
            FROM ai_requests ar
            JOIN user_profile_ext upe ON upe.user_id = ar.user_id
            WHERE ar.created_at >= NOW() - INTERVAL '30 days'
            GROUP BY upe.department
        ) usg ON usg.dept = d.name
        ORDER BY d.name
        ",
    )
    .fetch_all(pool)
    .await
}

pub async fn get_department(pool: &PgPool, id: &str) -> Result<Option<Department>, sqlx::Error> {
    sqlx::query_as::<_, Department>(
        "SELECT id, name, description, created_at, updated_at
         FROM departments WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn get_department_by_name(
    pool: &PgPool,
    name: &str,
) -> Result<Option<Department>, sqlx::Error> {
    sqlx::query_as::<_, Department>(
        "SELECT id, name, description, created_at, updated_at
         FROM departments WHERE name = $1",
    )
    .bind(name)
    .fetch_optional(pool)
    .await
}

pub async fn create_department(
    pool: &PgPool,
    input: &DepartmentInput,
) -> Result<Department, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    sqlx::query_as::<_, Department>(
        r"INSERT INTO departments (id, name, description)
          VALUES ($1, $2, $3)
          RETURNING id, name, description, created_at, updated_at",
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.description)
    .fetch_one(pool)
    .await
}

pub async fn update_department(
    pool: &PgPool,
    id: &str,
    input: &DepartmentInput,
) -> Result<Department, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let existing: Department = sqlx::query_as::<_, Department>(
        "SELECT id, name, description, created_at, updated_at
         FROM departments WHERE id = $1 FOR UPDATE",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    let renamed = existing.name != input.name;

    let updated: Department = sqlx::query_as::<_, Department>(
        r"UPDATE departments
          SET name = $2,
              description = $3,
              updated_at = NOW()
          WHERE id = $1
          RETURNING id, name, description, created_at, updated_at",
    )
    .bind(id)
    .bind(&input.name)
    .bind(&input.description)
    .fetch_one(&mut *tx)
    .await?;

    if renamed {
        sqlx::query!(
            "UPDATE user_profile_ext SET department = $2 WHERE department = $1",
            existing.name,
            updated.name,
        )
        .execute(&mut *tx)
        .await?;
        sqlx::query!(
            "UPDATE access_control_rules
             SET rule_value = $2, updated_at = NOW()
             WHERE rule_type = 'department' AND rule_value = $1",
            existing.name,
            updated.name,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(updated)
}

pub async fn delete_department(pool: &PgPool, id: &str) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    let dept: Department = sqlx::query_as::<_, Department>(
        "SELECT id, name, description, created_at, updated_at
         FROM departments WHERE id = $1 FOR UPDATE",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    if dept.name == "Default" {
        tx.rollback().await?;
        return Err(sqlx::Error::Protocol(
            "the 'Default' department cannot be deleted".into(),
        ));
    }

    sqlx::query!(
        "UPDATE user_profile_ext SET department = 'Default' WHERE department = $1",
        dept.name,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!(
        "DELETE FROM access_control_rules
         WHERE rule_type = 'department' AND rule_value = $1",
        dept.name,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query!("DELETE FROM departments WHERE id = $1", id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn list_department_members(
    pool: &PgPool,
    department_name: &str,
) -> Result<Vec<DepartmentMember>, sqlx::Error> {
    sqlx::query_as::<_, DepartmentMember>(
        r"
        SELECT
            u.id,
            u.email,
            u.display_name,
            u.status,
            u.roles,
            COALESCE(ar.input_tokens, 0)::BIGINT     AS input_tokens,
            COALESCE(ar.output_tokens, 0)::BIGINT    AS output_tokens,
            COALESCE(ar.requests, 0)::BIGINT         AS requests,
            COALESCE(ar.cost_microdollars, 0)::BIGINT AS cost_microdollars,
            ar.last_active                           AS last_active
        FROM users u
        LEFT JOIN (
            SELECT
                user_id,
                COALESCE(SUM(input_tokens), 0)::BIGINT  AS input_tokens,
                COALESCE(SUM(output_tokens), 0)::BIGINT AS output_tokens,
                COUNT(*)::BIGINT                        AS requests,
                COALESCE(SUM(cost_microdollars), 0)::BIGINT AS cost_microdollars,
                MAX(created_at)                         AS last_active
            FROM ai_requests
            WHERE created_at >= NOW() - INTERVAL '30 days'
            GROUP BY user_id
        ) ar ON ar.user_id = u.id
        JOIN user_profile_ext upe ON upe.user_id = u.id
        WHERE upe.department = $1
          AND NOT ('anonymous' = ANY(u.roles))
          AND u.email NOT LIKE '%@anonymous.local'
        ORDER BY (COALESCE(ar.input_tokens, 0) + COALESCE(ar.output_tokens, 0)) DESC, u.email
        ",
    )
    .bind(department_name)
    .fetch_all(pool)
    .await
}

/// Top tools used by members of a department in the last 30 days.
pub async fn list_department_top_tools(
    pool: &PgPool,
    department_name: &str,
    limit: i64,
) -> Result<Vec<DepartmentTopTool>, sqlx::Error> {
    sqlx::query_as::<_, DepartmentTopTool>(
        r"
        SELECT
            COALESCE(p.tool_name, 'unknown') AS tool_name,
            COALESCE(SUM(p.event_count), 0)::BIGINT AS invocations
        FROM plugin_usage_daily p
        JOIN user_profile_ext upe ON upe.user_id = p.user_id
        WHERE upe.department = $1
          AND p.tool_name IS NOT NULL
          AND p.date >= CURRENT_DATE - INTERVAL '30 days'
        GROUP BY p.tool_name
        ORDER BY invocations DESC
        LIMIT $2
        ",
    )
    .bind(department_name)
    .bind(limit)
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserManagementAggregate {
    pub user_id: String,
    pub department: String,
    pub assigned_skills_count: i64,
    pub devices_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserMarketplaceOverride {
    pub user_id: String,
    pub department: String,
    pub entity_id: String,
    pub access: String,
}

/// Loads marketplace-scoped overrides for users.
///
/// Joins `access_control_rules` to users so each row carries the `user_id` for
/// direct mapping. A user receives overrides from rules that match either their
/// own id or their department.
pub async fn list_user_marketplace_overrides(
    pool: &PgPool,
) -> Result<Vec<UserMarketplaceOverride>, sqlx::Error> {
    sqlx::query_as::<_, UserMarketplaceOverride>(
        r"
        SELECT
            u.id AS user_id,
            COALESCE(upe.department, '') AS department,
            acr.entity_id,
            acr.access
        FROM users u
        LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        JOIN access_control_rules acr
          ON acr.entity_type = 'marketplace'
         AND ((acr.rule_type = 'user' AND acr.rule_value = u.id)
              OR (acr.rule_type = 'department' AND acr.rule_value = COALESCE(upe.department, '')))
        WHERE NOT ('anonymous' = ANY(u.roles))
        ",
    )
    .fetch_all(pool)
    .await
}

pub async fn list_user_management_aggregates(
    pool: &PgPool,
) -> Result<Vec<UserManagementAggregate>, sqlx::Error> {
    sqlx::query_as::<_, UserManagementAggregate>(
        r"
        SELECT
            u.id AS user_id,
            COALESCE(upe.department, '') AS department,
            COALESCE((
                SELECT COUNT(DISTINCT acr.entity_id)
                FROM access_control_rules acr
                WHERE acr.entity_type = 'skill'
                  AND acr.access = 'allow'
                  AND ((acr.rule_type = 'department' AND acr.rule_value = COALESCE(upe.department, ''))
                       OR (acr.rule_type = 'user' AND acr.rule_value = u.id))
            ), 0)::BIGINT AS assigned_skills_count,
            COALESCE((
                SELECT COUNT(*) FROM user_api_keys
                WHERE user_id = u.id AND revoked_at IS NULL
            ), 0)::BIGINT AS devices_count,
            u.created_at
        FROM users u
        LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        WHERE NOT ('anonymous' = ANY(u.roles))
        ",
    )
    .fetch_all(pool)
    .await
}

pub async fn assign_user_to_department(
    pool: &PgPool,
    user_id: &str,
    department_name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"INSERT INTO user_profile_ext (user_id, department)
          VALUES ($1, $2)
          ON CONFLICT (user_id) DO UPDATE SET department = EXCLUDED.department",
        user_id,
        department_name,
    )
    .execute(pool)
    .await?;
    Ok(())
}
