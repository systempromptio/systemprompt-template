use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub display_name: Option<String>,
    pub status: String,
    pub roles: Vec<String>,
    pub total_sessions: i64,
    pub created_at: String,
}
