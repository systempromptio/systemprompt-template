use anyhow::anyhow;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Content {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub author: String,
    pub published_at: DateTime<Utc>,
    pub keywords: String,
    pub kind: String,
    pub image: Option<String>,
    pub category_id: Option<String>,
    pub source_id: String,
    pub version_hash: String,
    #[serde(default)]
    pub links: JsonValue,
    pub updated_at: Option<DateTime<Utc>>,
}

impl Content {
    pub fn links_metadata(&self) -> Vec<ContentLinkMetadata> {
        serde_json::from_value(self.links.clone()).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSummary {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub published_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    pub published_at: String,
    pub slug: String,
    #[serde(default)]
    pub keywords: String,
    pub kind: String,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub links: Vec<ContentLinkMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentLinkMetadata {
    pub title: String,
    pub url: String,
}

impl ContentMetadata {
    pub fn validate_with_allowed_types(&self, allowed_types: &[&str]) -> anyhow::Result<()> {
        if self.title.trim().is_empty() {
            return Err(anyhow!("title cannot be empty"));
        }

        if self.slug.trim().is_empty() {
            return Err(anyhow!("slug cannot be empty"));
        }

        if self.author.trim().is_empty() {
            return Err(anyhow!("author cannot be empty"));
        }

        if self.published_at.trim().is_empty() {
            return Err(anyhow!("published_at cannot be empty"));
        }

        if !self
            .slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(anyhow!(
                "slug must be lowercase alphanumeric with hyphens only (got: {})",
                self.slug
            ));
        }

        if !is_valid_date_format(&self.published_at) {
            return Err(anyhow!(
                "published_at must be in YYYY-MM-DD format (got: {})",
                self.published_at
            ));
        }

        if !allowed_types.contains(&self.kind.as_str()) {
            return Err(anyhow!(
                "invalid kind '{}'. must be one of: {}",
                self.kind,
                allowed_types.join(", ")
            ));
        }

        Ok(())
    }
}

fn is_valid_date_format(date_str: &str) -> bool {
    if date_str.len() != 10 {
        return false;
    }

    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return false;
    }

    parts[0].len() == 4
        && parts[0].chars().all(char::is_numeric)
        && parts[1].len() == 2
        && parts[1].chars().all(char::is_numeric)
        && parts[2].len() == 2
        && parts[2].chars().all(char::is_numeric)
}
