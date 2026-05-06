use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Department {
    pub id: String,
    pub name: String,
    pub description: String,
    pub manager_user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentInput {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub manager_user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DepartmentMember {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub status: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DepartmentSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub manager_user_id: Option<String>,
    pub manager_email: Option<String>,
    pub member_count: i64,
    pub assignment_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
