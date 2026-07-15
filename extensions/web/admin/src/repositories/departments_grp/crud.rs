//! Department record lifecycle: lookup, create, rename-aware update, guarded
//! delete, and the user→department assignment write.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use uuid::Uuid;

use crate::types::departments::{Department, DepartmentInput};

pub async fn get_department(pool: &PgPool, id: &str) -> Result<Option<Department>, sqlx::Error> {
    sqlx::query_as!(
        Department,
        r#"SELECT id, name, description as "description!", created_at, updated_at
           FROM departments WHERE id = $1"#,
        id,
    )
    .fetch_optional(pool)
    .await
}

pub async fn get_department_by_name(
    pool: &PgPool,
    name: &str,
) -> Result<Option<Department>, sqlx::Error> {
    sqlx::query_as!(
        Department,
        r#"SELECT id, name, description as "description!", created_at, updated_at
           FROM departments WHERE name = $1"#,
        name,
    )
    .fetch_optional(pool)
    .await
}

pub async fn create_department(
    pool: &PgPool,
    input: &DepartmentInput,
) -> Result<Department, sqlx::Error> {
    let id = Uuid::new_v4().to_string();
    sqlx::query_as!(
        Department,
        r#"INSERT INTO departments (id, name, description)
           VALUES ($1, $2, $3)
           RETURNING id, name, description as "description!", created_at, updated_at"#,
        id,
        input.name,
        input.description,
    )
    .fetch_one(pool)
    .await
}

pub async fn update_department(
    pool: &PgPool,
    id: &str,
    input: &DepartmentInput,
) -> Result<Department, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let existing = sqlx::query_as!(
        Department,
        r#"SELECT id, name, description as "description!", created_at, updated_at
           FROM departments WHERE id = $1 FOR UPDATE"#,
        id,
    )
    .fetch_one(&mut *tx)
    .await?;

    let renamed = existing.name != input.name;

    let updated = sqlx::query_as!(
        Department,
        r#"UPDATE departments
           SET name = $2,
               description = $3,
               updated_at = NOW()
           WHERE id = $1
           RETURNING id, name, description as "description!", created_at, updated_at"#,
        id,
        input.name,
        input.description,
    )
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

    let dept = sqlx::query_as!(
        Department,
        r#"SELECT id, name, description as "description!", created_at, updated_at
           FROM departments WHERE id = $1 FOR UPDATE"#,
        id,
    )
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

pub async fn assign_user_to_department(
    pool: &PgPool,
    user_id: &UserId,
    department_name: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"INSERT INTO user_profile_ext (user_id, department)
          VALUES ($1, $2)
          ON CONFLICT (user_id) DO UPDATE SET department = EXCLUDED.department",
        user_id.as_str(),
        department_name,
    )
    .execute(pool)
    .await?;
    Ok(())
}
