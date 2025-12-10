pub mod api;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub use api::{CreateVariableRequest, ListVariablesQuery, UpdateVariableRequest, VariableResponse};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ConfigVariable {
    pub id: String,
    pub name: String,
    pub value: Option<String>,
    #[sqlx(rename = "type")]
    pub variable_type: String,
    pub description: Option<String>,
    pub category: String,
    pub is_secret: Option<bool>,
    pub is_required: bool,
    pub default_value: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
