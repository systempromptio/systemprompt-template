use sqlx::PgPool;

pub async fn get_user_roles_department(
    pool: &PgPool,
    user_id: &str,
) -> Result<Option<(Vec<String>, String)>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT u.roles, COALESCE(upe.department, 'Default') AS "department!"
        FROM users u
        LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        WHERE u.id = $1
        "#,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| (r.roles, r.department)))
}
