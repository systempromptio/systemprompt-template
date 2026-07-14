//! Roster query backing the Access Control page's department tree.

use sqlx::PgPool;

/// One user row for the access-control department tree.
#[derive(Debug, sqlx::FromRow)]
pub struct AccessTreeUserRow {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub roles: Vec<String>,
    pub department: String,
    pub is_active: bool,
}

/// List human users (excluding anonymous accounts) for the access-control
/// tree, ordered by department then display name.
pub async fn list_users_for_access_tree(
    pool: &PgPool,
) -> Result<Vec<AccessTreeUserRow>, sqlx::Error> {
    sqlx::query_as!(
        AccessTreeUserRow,
        r#"SELECT
              u.id AS "id!",
              u.email AS "email!",
              COALESCE(u.display_name, u.full_name, u.name) AS "display_name?",
              u.roles AS "roles!",
              COALESCE(upe.department, '') AS "department!",
              (u.status = 'active') AS "is_active!"
           FROM users u
           LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
           WHERE NOT ('anonymous' = ANY(u.roles))
             AND u.email NOT LIKE '%@anonymous.local'
           ORDER BY COALESCE(upe.department, ''), COALESCE(u.display_name, u.email)"#,
    )
    .fetch_all(pool)
    .await
}
