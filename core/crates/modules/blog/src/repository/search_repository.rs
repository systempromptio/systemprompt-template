use crate::models::SearchResult;
use anyhow::{Context, Result};
use std::sync::Arc;
use systemprompt_core_database::DatabaseProvider;
use systemprompt_core_database::DatabaseQueryEnum;

#[derive(Debug)]
pub struct SearchRepository {
    db: Arc<dyn DatabaseProvider>,
}

impl SearchRepository {
    pub fn new(db: Arc<dyn DatabaseProvider>) -> Self {
        Self { db }
    }

    // FTS search currently disabled - embeddings removed
    // TODO: Implement basic FTS using SQL LIKE or full-text search indexes on markdown_fts table
    // pub async fn fts_search(&self, query_text: &str, limit: i64) -> Result<Vec<SearchResult>> {
    //     // TODO: Implement without embeddings
    // }

    pub async fn search_by_category(
        &self,
        category_id: &str,
        limit: i64,
    ) -> Result<Vec<SearchResult>> {
        let query = DatabaseQueryEnum::SearchByCategory.get(self.db.as_ref());

        let rows = self
            .db
            .fetch_all(&query, &[&category_id, &limit, &0i64])
            .await
            .context(format!("Failed to search by category: {category_id}"))?;

        rows.iter()
            .map(SearchResult::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn search_by_tags(
        &self,
        tag_ids: &[String],
        limit: i64,
    ) -> Result<Vec<SearchResult>> {
        let query = DatabaseQueryEnum::SearchByTags.get(self.db.as_ref());

        let tags_json =
            serde_json::to_string(tag_ids).context("Failed to serialize tag IDs to JSON")?;

        let rows = self
            .db
            .fetch_all(&query, &[&tags_json, &(tag_ids.len() as i64), &limit])
            .await
            .context(format!(
                "Failed to search by tags, tag count: {}",
                tag_ids.len()
            ))?;

        rows.iter()
            .map(SearchResult::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    pub async fn search_by_keyword(&self, keyword: &str, limit: i64) -> Result<Vec<SearchResult>> {
        let query = DatabaseQueryEnum::SearchContentByKeyword.get(self.db.as_ref());

        let rows = self
            .db
            .fetch_all(&query, &[&keyword, &limit])
            .await
            .context(format!("Failed to search by keyword: {keyword}"))?;

        rows.iter()
            .map(SearchResult::from_json_row)
            .collect::<Result<Vec<_>>>()
    }

    // Hybrid search currently disabled - embeddings removed
    // TODO: Implement basic search combining categories and tags
    // pub async fn hybrid_search(
    //     &self,
    //     query_text: &str,
    //     filters: &SearchFilters,
    //     limit: i64,
    // ) -> Result<Vec<SearchResult>> {
    //     // TODO: Implement without embeddings
    // }
}
