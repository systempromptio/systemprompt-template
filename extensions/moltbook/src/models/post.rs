use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoltbookPost {
    pub id: String,
    pub title: String,
    pub content: String,
    pub url: Option<String>,
    pub submolt: String,
    pub author_id: String,
    pub author_name: String,
    pub upvotes: i64,
    pub downvotes: i64,
    pub comments_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePostRequest {
    pub submolt: String,
    pub title: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePostResponse {
    pub id: String,
    pub title: String,
    pub content: String,
    pub submolt: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPostsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submolt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}

impl Default for ListPostsQuery {
    fn default() -> Self {
        Self {
            submolt: None,
            author_id: None,
            sort: Some("hot".to_string()),
            limit: Some(25),
            offset: Some(0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteRequest {
    pub direction: VoteDirection,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum VoteDirection {
    Up,
    Down,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostSearchQuery {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submolt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}
