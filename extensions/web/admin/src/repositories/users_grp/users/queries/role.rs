use sqlx::PgPool;

pub async fn get_user_roles_department(
    pool: &PgPool,
    user_id: &str,
) -> Result<Option<(Vec<String>, String)>, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT roles, department FROM users WHERE id = $1"#,
        user_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| (r.roles, r.department)))
}
