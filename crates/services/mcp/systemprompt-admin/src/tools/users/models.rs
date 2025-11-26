use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use systemprompt_core_database::JsonRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub roles: Vec<String>,
    pub total_sessions: i32,
}

impl User {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let id = row
            .get("uuid")
            .or_else(|| row.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing id or uuid"))?
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

        let roles_str = row.get("roles").and_then(|v| v.as_str()).unwrap_or("[]");
        let roles: Vec<String> = serde_json::from_str(roles_str)?;

        let total_sessions = row
            .get("total_sessions")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;

        Ok(Self {
            id,
            name,
            email,
            roles,
            total_sessions,
        })
    }
}
