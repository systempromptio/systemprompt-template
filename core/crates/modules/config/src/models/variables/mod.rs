pub mod api;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use systemprompt_core_database::JsonRow;

pub use api::{CreateVariableRequest, ListVariablesQuery, UpdateVariableRequest, VariableResponse};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Variable {
    pub id: i32,
    pub name: String,
    pub value: Option<String>,
    pub r#type: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_secret: bool,
    pub is_required: bool,
    pub default_value: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Variable {
    pub fn from_json_row(row: &JsonRow) -> anyhow::Result<Self> {
        use anyhow::anyhow;

        let id = row
            .get("id")
            .and_then(serde_json::Value::as_i64)
            .ok_or_else(|| anyhow!("Missing id"))? as i32;

        let name = row
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing name"))?
            .to_string();

        let value = row.get("value").and_then(|v| v.as_str()).map(String::from);

        let r#type = row
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing type"))?
            .to_string();

        let description = row
            .get("description")
            .and_then(|v| v.as_str())
            .map(String::from);

        let category = row
            .get("category")
            .and_then(|v| v.as_str())
            .map(String::from);

        let is_secret = row
            .get("is_secret")
            .and_then(|v| v.as_bool().or_else(|| v.as_i64().map(|i| i != 0)))
            .ok_or_else(|| anyhow!("Missing is_secret"))?;

        let is_required = row
            .get("is_required")
            .and_then(|v| v.as_bool().or_else(|| v.as_i64().map(|i| i != 0)))
            .ok_or_else(|| anyhow!("Missing is_required"))?;

        let default_value = row
            .get("default_value")
            .and_then(|v| v.as_str())
            .map(String::from);

        let created_at = row
            .get("created_at")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .ok_or_else(|| anyhow!("Invalid created_at"))?;

        let updated_at = row
            .get("updated_at")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .ok_or_else(|| anyhow!("Invalid updated_at"))?;

        Ok(Self {
            id,
            name,
            value,
            r#type,
            description,
            category,
            is_secret,
            is_required,
            default_value,
            created_at,
            updated_at,
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VariableType {
    String,
    Integer,
    Boolean,
    Json,
}

impl VariableType {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Integer => "integer",
            Self::Boolean => "boolean",
            Self::Json => "json",
        }
    }

    pub fn try_from_str(s: &str) -> Option<Self> {
        match s {
            "string" => Some(Self::String),
            "integer" => Some(Self::Integer),
            "boolean" => Some(Self::Boolean),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VariableCategory {
    System,
    Database,
    Web,
    Security,
    Logging,
    Module,
}

impl VariableCategory {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Database => "database",
            Self::Web => "web",
            Self::Security => "security",
            Self::Logging => "logging",
            Self::Module => "module",
        }
    }

    pub fn try_from_str(s: &str) -> Option<Self> {
        match s {
            "system" => Some(Self::System),
            "database" => Some(Self::Database),
            "web" => Some(Self::Web),
            "security" => Some(Self::Security),
            "logging" => Some(Self::Logging),
            "module" => Some(Self::Module),
            _ => None,
        }
    }
}
