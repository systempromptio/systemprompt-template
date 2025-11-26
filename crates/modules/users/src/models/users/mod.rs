pub mod api;
pub mod rows;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::fmt;
use std::str::FromStr;

pub use api::{CreateUserRequest, ListUsersQuery, UpdateUserRequest};
pub use rows::{UserRowPostgres, UserRowSqlite};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserResponse {
    pub uuid: String,
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

impl UserResponse {
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.contains(&role.to_string())
    }

    pub fn from_json_row(
        row: &std::collections::HashMap<String, serde_json::Value>,
    ) -> anyhow::Result<Self> {
        use anyhow::anyhow;

        let uuid = row
            .get("uuid")
            .or_else(|| row.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing uuid"))?
            .to_string();

        let name = row
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing name"))?
            .to_string();

        let email = row
            .get("email")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing email"))?
            .to_string();

        let full_name = row
            .get("full_name")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let display_name = row
            .get("display_name")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let status = row
            .get("status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing status"))?
            .to_string();

        let email_verified = row
            .get("email_verified")
            .and_then(|v| v.as_bool().or_else(|| v.as_i64().map(|i| i != 0)))
            .unwrap_or(false);

        let roles = row
            .get("roles")
            .ok_or_else(|| anyhow!("Missing roles"))
            .and_then(|v| {
                if let Some(arr) = v.as_array() {
                    arr.iter()
                        .map(|item| {
                            item.as_str()
                                .map(std::string::ToString::to_string)
                                .ok_or_else(|| anyhow!("Role item is not a string"))
                        })
                        .collect::<anyhow::Result<Vec<String>>>()
                } else if let Some(s) = v.as_str() {
                    serde_json::from_str::<Vec<String>>(s)
                        .map_err(|e| anyhow!("Invalid roles JSON: {e}"))
                } else {
                    Err(anyhow!("Roles is neither array nor JSON string"))
                }
            })?;

        let avatar_url = row
            .get("avatar_url")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let created_at = row
            .get("created_at")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .ok_or_else(|| anyhow!("Invalid created_at"))?;

        let updated_at = row
            .get("updated_at")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .ok_or_else(|| anyhow!("Invalid updated_at"))?;

        Ok(Self {
            uuid,
            name,
            email,
            full_name,
            display_name,
            status,
            email_verified,
            roles,
            avatar_url,
            created_at,
            updated_at,
        })
    }
}

impl systemprompt_core_database::FromDatabaseRow for UserResponse {
    fn from_postgres_row(row: &sqlx::postgres::PgRow) -> anyhow::Result<Self> {
        let row = UserRowPostgres::from_row(row)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize Postgres row: {e}"))?;
        Ok(row.into())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    Pending,
    Deleted,
}

impl fmt::Display for UserStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Active => "active",
                Self::Inactive => "inactive",
                Self::Suspended => "suspended",
                Self::Pending => "pending",
                Self::Deleted => "deleted",
            }
        )
    }
}

impl FromStr for UserStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            "suspended" => Ok(Self::Suspended),
            "pending" => Ok(Self::Pending),
            "deleted" => Ok(Self::Deleted),
            _ => Err(format!("Invalid user status: {s}")),
        }
    }
}
