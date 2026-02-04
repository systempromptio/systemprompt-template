use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Submolt {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub rules: Option<String>,
    pub subscribers_count: i64,
    pub posts_count: i64,
    pub is_nsfw: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubmoltRequest {
    pub name: String,
    pub display_name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<String>,
    #[serde(default)]
    pub is_nsfw: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmoltSearchQuery {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribeRequest {
    pub submolt: String,
}
