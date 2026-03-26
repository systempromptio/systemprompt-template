use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use systemprompt::identifiers::{CategoryId, ContentId, SourceId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub filters: Option<SearchFilters>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub category_id: Option<CategoryId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SearchResult {
    pub id: ContentId,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub image: Option<String>,
    pub view_count: i64,
    pub source_id: SourceId,
    pub category_id: Option<CategoryId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: usize,
}
