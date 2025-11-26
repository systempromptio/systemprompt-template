use crate::models::{SearchRequest, SearchResponse, SearchResult};
use crate::repository::{ContentRepository, SearchRepository};
use anyhow::Result;
use std::sync::Arc;
use systemprompt_core_database::DatabaseProvider;

#[derive(Debug)]
pub struct SearchService {
    search_repo: SearchRepository,
    content_repo: ContentRepository,
}

impl SearchService {
    pub fn new(db: Arc<dyn DatabaseProvider>) -> Self {
        Self {
            search_repo: SearchRepository::new(db.clone()),
            content_repo: ContentRepository::new(db),
        }
    }

    pub async fn search(&self, request: &SearchRequest) -> Result<SearchResponse> {
        let limit = request.limit.unwrap_or(10);

        // Embeddings removed - fallback to category/tag search
        let results = if let Some(filters) = &request.filters {
            if let Some(category_id) = &filters.category_id {
                self.search_repo
                    .search_by_category(category_id, limit)
                    .await?
            } else if !filters.tag_ids.is_empty() {
                self.search_repo
                    .search_by_tags(&filters.tag_ids, limit)
                    .await?
            } else {
                vec![]
            }
        } else {
            // No filters provided - list all content across all sources
            let content_list = self.content_repo.list_all(limit, 0).await?;
            content_list
                .into_iter()
                .map(Self::content_to_search_result)
                .collect()
        };

        Ok(SearchResponse {
            total: results.len(),
            results,
        })
    }

    pub async fn search_by_category(
        &self,
        category_id: &str,
        limit: i64,
    ) -> Result<Vec<SearchResult>> {
        self.search_repo
            .search_by_category(category_id, limit)
            .await
    }

    pub async fn search_by_tags(
        &self,
        tag_ids: &[String],
        limit: i64,
    ) -> Result<Vec<SearchResult>> {
        self.search_repo.search_by_tags(tag_ids, limit).await
    }

    fn content_to_search_result(content: crate::models::Content) -> SearchResult {
        SearchResult {
            id: content.id,
            title: content.title,
            slug: content.slug,
            description: content.description,
            source_id: content.source_id,
            category: None,
        }
    }
}
