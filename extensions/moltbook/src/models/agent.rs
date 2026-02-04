use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoltbookAgent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub api_key: Option<String>,
    pub verified: bool,
    pub followers_count: i64,
    pub following_count: i64,
    pub posts_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAgentRequest {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterAgentResponse {
    pub id: String,
    pub api_key: String,
    pub claim_url: String,
    pub verification_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub id: String,
    pub name: String,
    pub description: String,
    pub avatar_url: Option<String>,
    pub banner_url: Option<String>,
    pub website: Option<String>,
    pub location: Option<String>,
    pub verified: bool,
    pub followers_count: i64,
    pub following_count: i64,
    pub created_at: DateTime<Utc>,
}
