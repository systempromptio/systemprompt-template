//! Search service for content queries.

use sqlx::PgPool;
use std::sync::Arc;

use crate::error::BlogError;
use crate::models::{SearchRequest, SearchResponse};
use crate::repository::SearchRepository;

/// Service for searching content.
#[derive(Debug, Clone)]
pub struct SearchService {
    repo: SearchRepository,
}

impl SearchService {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            repo: SearchRepository::new(pool),
        }
    }

    /// Search content with the given request.
    pub async fn search(&self, request: &SearchRequest) -> Result<SearchResponse, BlogError> {
        let limit = request.limit.unwrap_or(20);
        let results = self.repo.search_by_keyword(&request.query, limit).await?;
        let total = results.len();

        Ok(SearchResponse { results, total })
    }
}
