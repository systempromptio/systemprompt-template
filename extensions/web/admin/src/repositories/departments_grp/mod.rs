use sqlx::PgPool;
use uuid::Uuid;

use crate::types::departments::{
    Department, DepartmentInput, DepartmentMember, DepartmentSummary,
};

pub async fn list_departments(pool: &PgPool) -> Result<Vec<DepartmentSummary>, sqlx::Error> {
    sqlx::query_as::<_, DepartmentSummary>(
        r"
        SELECT
            d.id,
            d.name,
            d.description,
            d.manager_user_id,
            mgr.email AS manager_email,
            COALESCE(mc.member_count, 0)::BIGINT AS member_count,
            COALESCE(ac.assignment_count, 0)::BIGINT AS assignment_count,
            d.created_at,
            d.updated_at
        FROM departments d
        LEFT JOIN users mgr ON mgr.id = d.manager_user_id
        LEFT JOIN (
            SELECT department, COUNT(*)::BIGINT AS member_count
            FROM users
            WHERE department IS NOT NULL AND department <> ''
            GROUP BY department
        ) mc ON mc.department = d.name
        LEFT JOIN (
            SELECT rule_value, COUNT(*)::BIGINT AS assignment_count
            FROM access_control_rules
            WHERE rule_type = 'department'
            GROUP BY rule_value
        ) ac ON ac.rule_value = d.name
        ORDER BY d.name
        ",
    )
    .fetch_all(pool)
    .await
}

pub async fn get_department(pool: &PgPool, id: &str) -> Result<Option<Department>, sqlx::Error> {
    sqlx::query_as::<_, Department>(
        "SELECT id, name, description, manager_user_id, created_at, updated_at
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
        "SELECT id, name, description, manager_user_id, created_at, updated_at
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
        r"INSERT INTO departments (id, name, description, manager_user_id)
          VALUES ($1, $2, $3, $4)
          RETURNING id, name, description, manager_user_id, created_at, updated_at",
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.description)
    .bind(&input.manager_user_id)
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
        "SELECT id, name, description, manager_user_id, created_at, updated_at
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
              manager_user_id = $4,
              updated_at = NOW()
          WHERE id = $1
          RETURNING id, name, description, manager_user_id, created_at, updated_at",
    )
    .bind(id)
    .bind(&input.name)
    .bind(&input.description)
    .bind(&input.manager_user_id)
    .fetch_one(&mut *tx)
    .await?;

    if renamed {
        sqlx::query("UPDATE users SET department = $2 WHERE department = $1")
            .bind(&existing.name)
            .bind(&updated.name)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "UPDATE access_control_rules
             SET rule_value = $2, updated_at = NOW()
             WHERE rule_type = 'department' AND rule_value = $1",
        )
        .bind(&existing.name)
        .bind(&updated.name)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(updated)
}

pub async fn delete_department(pool: &PgPool, id: &str) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    let dept: Department = sqlx::query_as::<_, Department>(
        "SELECT id, name, description, manager_user_id, created_at, updated_at
         FROM departments WHERE id = $1 FOR UPDATE",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query("UPDATE users SET department = '' WHERE department = $1")
        .bind(&dept.name)
        .execute(&mut *tx)
        .await?;

    sqlx::query(
        "DELETE FROM access_control_rules
         WHERE rule_type = 'department' AND rule_value = $1",
    )
    .bind(&dept.name)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM departments WHERE id = $1")
        .bind(id)
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
        r"SELECT id, email, display_name, status, roles
          FROM users
          WHERE department = $1
            AND NOT ('anonymous' = ANY(roles))
            AND email NOT LIKE '%@anonymous.local'
          ORDER BY email",
    )
    .bind(department_name)
    .fetch_all(pool)
    .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserManagementAggregate {
    pub user_id: String,
    pub department: String,
    pub assigned_skills_count: i64,
    pub assigned_marketplaces_count: i64,
    pub devices_count: i64,
}

pub async fn list_user_management_aggregates(
    pool: &PgPool,
) -> Result<Vec<UserManagementAggregate>, sqlx::Error> {
    sqlx::query_as::<_, UserManagementAggregate>(
        r"
        SELECT
            u.id AS user_id,
            COALESCE(u.department, '') AS department,
            (
                COALESCE((SELECT COUNT(DISTINCT base_skill_id) FROM user_skills WHERE user_id = u.id), 0)
                + COALESCE((
                    SELECT COUNT(DISTINCT acr.entity_id)
                    FROM access_control_rules acr
                    WHERE acr.entity_type = 'skill'
                      AND acr.access = 'allow'
                      AND ((acr.rule_type = 'department' AND acr.rule_value = u.department)
                           OR (acr.rule_type = 'user' AND acr.rule_value = u.id))
                ), 0)
            )::BIGINT AS assigned_skills_count,
            COALESCE((
                SELECT COUNT(DISTINCT acr.entity_id)
                FROM access_control_rules acr
                WHERE acr.entity_type = 'marketplace'
                  AND acr.access = 'allow'
                  AND ((acr.rule_type = 'department' AND acr.rule_value = u.department)
                       OR (acr.rule_type = 'user' AND acr.rule_value = u.id))
            ), 0)::BIGINT AS assigned_marketplaces_count,
            COALESCE((
                SELECT COUNT(*) FROM user_api_keys
                WHERE user_id = u.id AND revoked_at IS NULL
            ), 0)::BIGINT AS devices_count
        FROM users u
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
    sqlx::query("UPDATE users SET department = $2 WHERE id = $1")
        .bind(user_id)
        .bind(department_name)
        .execute(pool)
        .await?;
    Ok(())
}
