//! Lookup of the user behind a bridge credential.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug, Clone)]
pub struct BridgeUserRow {
    pub id: String,
    pub name: String,
    pub email: String,
    pub display_name: Option<String>,
    pub roles: Vec<String>,
}

pub async fn find_bridge_user(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<BridgeUserRow>, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT id, name, email, display_name,
                  COALESCE(roles, '{}') as "roles!: Vec<String>"
           FROM users WHERE id = $1"#,
        user_id.as_str(),
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| BridgeUserRow {
        id: r.id,
        name: r.name,
        email: r.email,
        display_name: r.display_name,
        roles: r.roles,
    }))
}
