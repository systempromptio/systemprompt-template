use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use systemprompt_core_database::JsonRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub filters: Option<SearchFilters>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    pub category_id: Option<String>,
    pub tag_ids: Vec<String>,
}

impl SearchFilters {
    pub fn tags_json(&self) -> Option<String> {
        if self.tag_ids.is_empty() {
            None
        } else {
            serde_json::to_string(&self.tag_ids).ok()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub description: String,
    pub source_id: String,
    pub category: Option<String>,
}

impl SearchResult {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing id"))?
            .to_string();

        let title = row
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing title"))?
            .to_string();

        let slug = row
            .get("slug")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing slug"))?
            .to_string();

        let description = row
            .get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing description"))?
            .to_string();

        let source_id = row
            .get("source_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing source_id"))?
            .to_string();

        let category = row
            .get("category")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(Self {
            id,
            title,
            slug,
            description,
            source_id,
            category,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: usize,
}
