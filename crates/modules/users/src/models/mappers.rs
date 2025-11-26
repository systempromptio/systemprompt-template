use crate::models::users::UserResponse;
use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, FromRow)]
pub struct UserRow {
    pub uuid: String,
    pub name: String,
    pub email: String,
    pub full_name: Option<String>,
    pub display_name: Option<String>,
    pub status: String,
    pub email_verified: bool,
    pub roles: String,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserRow> for UserResponse {
    fn from(row: UserRow) -> Self {
        let roles = serde_json::from_str::<Vec<String>>(&row.roles)
            .unwrap_or_else(|_| vec!["user".to_string()]);

        Self {
            uuid: row.uuid,
            name: row.name,
            email: row.email,
            full_name: row.full_name,
            display_name: row.display_name,
            status: row.status,
            email_verified: row.email_verified,
            roles,
            avatar_url: row.avatar_url,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
