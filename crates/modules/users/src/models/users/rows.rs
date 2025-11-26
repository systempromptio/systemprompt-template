use anyhow::{anyhow, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::FromRow;

use super::UserResponse;

/// PostgreSQL-specific row struct for User data.
///
/// Maps directly to `PostgreSQL` column types using sqlx's `FromRow` derive.
/// - Timestamps are `DateTime`<Utc>
/// - Roles are stored as TEXT[] and automatically convert to Vec<String>
#[derive(Debug, Clone, FromRow)]
pub struct UserRowPostgres {
    #[sqlx(rename = "uuid")]
    pub id: String,
    pub name: String,
    pub email: String,
    pub full_name: Option<String>,
    pub display_name: Option<String>,
    pub status: String,
    pub email_verified: bool,
    pub roles: Vec<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// SQLite-specific row struct for User data.
///
/// Maps directly to `SQLite` column types using sqlx's `FromRow` derive.
/// - Timestamps are `NaiveDateTime` (no timezone info in `SQLite`)
/// - Roles are stored as JSON string and need manual parsing
#[derive(Debug, Clone, FromRow)]
pub struct UserRowSqlite {
    #[sqlx(rename = "uuid")]
    pub id: String,
    pub name: String,
    pub email: String,
    pub full_name: Option<String>,
    pub display_name: Option<String>,
    pub status: String,
    pub email_verified: bool,
    pub roles: String,
    pub avatar_url: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<UserRowPostgres> for UserResponse {
    fn from(row: UserRowPostgres) -> Self {
        Self {
            uuid: row.id,
            name: row.name,
            email: row.email,
            full_name: row.full_name,
            display_name: row.display_name,
            status: row.status,
            email_verified: row.email_verified,
            roles: row.roles,
            avatar_url: row.avatar_url,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl TryFrom<UserRowSqlite> for UserResponse {
    type Error = anyhow::Error;

    fn try_from(row: UserRowSqlite) -> Result<Self> {
        let roles: Vec<String> = serde_json::from_str(&row.roles)
            .map_err(|e| anyhow!("Failed to parse roles JSON: {e}"))?;

        Ok(Self {
            uuid: row.id,
            name: row.name,
            email: row.email,
            full_name: row.full_name,
            display_name: row.display_name,
            status: row.status,
            email_verified: row.email_verified,
            roles,
            avatar_url: row.avatar_url,
            created_at: row.created_at.and_utc(),
            updated_at: row.updated_at.and_utc(),
        })
    }
}
