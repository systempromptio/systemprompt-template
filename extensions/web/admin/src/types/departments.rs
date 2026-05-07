use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Department {
    pub id: String,
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentInput {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DepartmentMember {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub status: String,
    pub roles: Vec<String>,
    #[serde(default)]
    pub input_tokens: i64,
    #[serde(default)]
    pub output_tokens: i64,
    #[serde(default)]
    pub requests: i64,
    #[serde(default)]
    pub cost_microdollars: i64,
    #[serde(default)]
    pub last_active: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DepartmentSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub member_count: i64,
    pub assignment_count: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub requests: i64,
    pub cost_microdollars: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DepartmentTopTool {
    pub tool_name: String,
    pub invocations: i64,
}
