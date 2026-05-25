use sqlx::PgPool;

#[derive(Debug, Clone)]
pub struct CoworkUserRow {
    pub id: String,
    pub name: String,
    pub email: String,
    pub display_name: Option<String>,
    pub roles: Vec<String>,
}

pub async fn find_cowork_user(
    pool: &PgPool,
    user_id: &str,
) -> Result<Option<CoworkUserRow>, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT id, name, email, display_name,
                  COALESCE(roles, '{}') as "roles!: Vec<String>"
           FROM users WHERE id = $1"#,
        user_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| CoworkUserRow {
        id: r.id,
        name: r.name,
        email: r.email,
        display_name: r.display_name,
        roles: r.roles,
    }))
}
