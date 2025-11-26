use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt_core_database::JsonRow;
use systemprompt_models::ContentLink;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub category_id: String,
    pub source_id: String,
    pub version_hash: String,
    pub public: bool,
    pub parent_content_id: Option<String>,
    pub links: Vec<ContentLink>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Content {
    pub fn from_json_row(row: &JsonRow) -> Result<Self> {
        let id = row
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing id"))?
            .to_string();

        let slug = row
            .get("slug")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing slug"))?
            .to_string();

        let title = row
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing title"))?
            .to_string();

        let body = row
            .get("body")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing body"))?
            .to_string();

        let author = row
            .get("author")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing author"))?
            .to_string();

        let published_at = row
            .get("published_at")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .ok_or_else(|| anyhow!("Missing or invalid published_at"))?;

        let category_id = row
            .get("category_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing category_id"))?
            .to_string();

        let source_id = row
            .get("source_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing source_id"))?
            .to_string();

        let description = row
            .get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing description"))?
            .to_string();

        let keywords = row
            .get("keywords")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing keywords"))?
            .to_string();

        let kind = row
            .get("kind")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing kind"))?
            .to_string();

        let image = row
            .get("image")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let version_hash = row
            .get("version_hash")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("Missing version_hash"))?
            .to_string();

        let created_at = row
            .get("created_at")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .ok_or_else(|| anyhow!("Missing or invalid created_at"))?;

        let updated_at = row
            .get("updated_at")
            .and_then(systemprompt_core_database::parse_database_datetime)
            .ok_or_else(|| anyhow!("Missing or invalid updated_at"))?;

        let public = row
            .get("public")
            .and_then(serde_json::Value::as_bool)
            .ok_or_else(|| anyhow!("Missing or invalid public"))?;

        let parent_content_id = row
            .get("parent_content_id")
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string);

        let links = row
            .get("links")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow!("Missing or invalid links"))?
            .iter()
            .map(|item| {
                serde_json::from_value::<ContentLink>(item.clone())
                    .map_err(|e| anyhow!("Invalid link data: {e}"))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            id,
            slug,
            title,
            description,
            body,
            author,
            published_at,
            keywords,
            kind,
            image,
            category_id,
            source_id,
            version_hash,
            public,
            parent_content_id,
            links,
            created_at,
            updated_at,
        })
    }
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
    #[serde(default = "default_public")]
    pub public: bool,
    #[serde(default)]
    pub links: Vec<ContentLinkMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentLinkMetadata {
    pub title: String,
    pub url: String,
}

impl ContentMetadata {
    pub fn validate(&self) -> Result<()> {
        self.validate_with_allowed_types(&["article", "post", "page", "blog", "documentation"])
    }

    pub fn validate_with_allowed_types(&self, allowed_types: &[&str]) -> Result<()> {
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

const fn default_public() -> bool {
    true
}
