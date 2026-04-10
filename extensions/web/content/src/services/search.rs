use sqlx::PgPool;
use std::sync::Arc;

use crate::repository::SearchRepository;
use systemprompt_web_shared::error::BlogError;
use systemprompt_web_shared::models::{SearchRequest, SearchResponse};

#[derive(Debug, Clone)]
pub struct SearchService {
    repo: SearchRepository,
}

impl SearchService {
    #[must_use]
    pub const fn new(pool: Arc<PgPool>) -> Self {
        Self {
            repo: SearchRepository::new(pool),
        }
    }

    pub async fn search(&self, request: &SearchRequest) -> Result<SearchResponse, BlogError> {
        let limit = request.limit.unwrap_or(20);
        let results = self.repo.search_by_keyword(&request.query, limit).await?;
        let total = results.len();

        Ok(SearchResponse { results, total })
    }
}
