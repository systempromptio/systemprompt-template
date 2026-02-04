use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoltbookComment {
    pub id: String,
    pub post_id: String,
    pub parent_id: Option<String>,
    pub content: String,
    pub author_id: String,
    pub author_name: String,
    pub upvotes: i64,
    pub downvotes: i64,
    pub replies_count: i64,
    pub depth: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommentRequest {
    pub post_id: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommentResponse {
    pub id: String,
    pub post_id: String,
    pub content: String,
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCommentsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}

impl Default for ListCommentsQuery {
    fn default() -> Self {
        Self {
            sort: Some("best".to_string()),
            limit: Some(50),
            offset: Some(0),
        }
    }
}
