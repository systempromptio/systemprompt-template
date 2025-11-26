use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentConfig {
    pub content_sources: HashMap<String, ContentSourceConfig>,
    #[serde(default)]
    pub metadata: Metadata,
    #[serde(default)]
    pub categories: HashMap<String, Category>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSourceConfig {
    pub path: String,
    pub source_id: String,
    pub category_id: String,
    pub enabled: bool,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub allowed_content_types: Vec<String>,
    #[serde(default)]
    pub indexing: Option<IndexingConfig>,
    #[serde(default)]
    pub sitemap: Option<SitemapConfig>,
    #[serde(default)]
    pub branding: Option<SourceBranding>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceBranding {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub keywords: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct IndexingConfig {
    #[serde(default)]
    pub clear_before: bool,
    #[serde(default)]
    pub recursive: bool,
    #[serde(default)]
    pub preserve_revisions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SitemapConfig {
    pub enabled: bool,
    pub url_pattern: String,
    pub priority: f32,
    pub changefreq: String,
    #[serde(default)]
    pub fetch_from: String,
    #[serde(default)]
    pub parent_route: Option<ParentRoute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParentRoute {
    pub enabled: bool,
    pub url: String,
    pub priority: f32,
    pub changefreq: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Metadata {
    #[serde(default)]
    pub default_author: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub structured_data: StructuredData,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StructuredData {
    #[serde(default)]
    pub organization: OrganizationData,
    #[serde(default)]
    pub article: ArticleDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrganizationData {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub logo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ArticleDefaults {
    #[serde(default)]
    pub article_type: String,
    #[serde(default)]
    pub article_section: String,
    #[serde(default)]
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Category {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
}

impl ContentConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub async fn load_async(path: &str) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let config: Self = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    pub fn is_html_page(&self, path: &str) -> bool {
        if path == "/" {
            return true;
        }

        for source in self.content_sources.values() {
            if !source.enabled {
                continue;
            }

            let Some(sitemap) = &source.sitemap else {
                continue;
            };

            if !sitemap.enabled {
                continue;
            }

            if Self::matches_url_pattern(&sitemap.url_pattern, path) {
                return true;
            }
        }

        false
    }

    pub fn matches_url_pattern(pattern: &str, path: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
        let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        if pattern_parts.len() != path_parts.len() {
            return false;
        }

        for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
            if *pattern_part == "{slug}" {
                continue;
            }

            if pattern_part != path_part {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_url_pattern() {
        assert!(ContentConfig::matches_url_pattern(
            "/blog/{slug}",
            "/blog/test-post"
        ));
        assert!(ContentConfig::matches_url_pattern("/{slug}", "/about"));
        assert!(!ContentConfig::matches_url_pattern(
            "/blog/{slug}",
            "/about"
        ));
        assert!(!ContentConfig::matches_url_pattern("/{slug}", "/blog/post"));
        assert!(!ContentConfig::matches_url_pattern(
            "/blog/{slug}",
            "/api/content"
        ));
    }

    #[test]
    fn test_config_deserialization() {
        let yaml = r#"
content_sources:
  blog:
    path: crates/services/content/blog
    source_id: blog
    category_id: blog
    enabled: true
    description: "Blog posts"
    allowed_content_types:
      - article
      - blog
    sitemap:
      enabled: true
      url_pattern: /blog/{slug}
      priority: 0.8
      changefreq: weekly
      parent_route:
        enabled: true
        url: /blog
        priority: 0.9
        changefreq: daily
    branding:
      name: Blog
      description: Technical blog
metadata:
  default_author: Default Author
  language: en
  structured_data:
    organization:
      name: Example Org
      url: https://example.com
      logo: https://example.com/logo.png
"#;

        let config: ContentConfig = serde_yaml::from_str(yaml).expect("Config parse failed");
        assert!(config.content_sources.contains_key("blog"));
        let blog = &config.content_sources["blog"];
        assert_eq!(blog.source_id, "blog");
        assert!(blog.enabled);
        assert!(blog.sitemap.as_ref().unwrap().enabled);
        assert!(blog.branding.as_ref().unwrap().name.is_some());
    }
}
