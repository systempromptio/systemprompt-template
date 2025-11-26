use anyhow::Result;
use systemprompt_models::ContentConfig;

#[derive(Debug, Clone)]
pub struct StaticContentMatcher {
    patterns: Vec<(String, String)>,
}

impl StaticContentMatcher {
    pub fn from_config(config_path: &str) -> Result<Self> {
        let config = ContentConfig::load_from_file(config_path)?;

        let mut patterns = Vec::new();

        for (source_id, source) in config.content_sources {
            if source.enabled {
                if let Some(sitemap) = &source.sitemap {
                    if sitemap.enabled {
                        patterns.push((sitemap.url_pattern.clone(), source_id));
                    }
                }
            }
        }

        Ok(Self { patterns })
    }

    pub fn empty() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    pub fn matches(&self, path: &str) -> Option<(String, String)> {
        for (pattern, source_id) in &self.patterns {
            if let Some(slug) = extract_slug(path, pattern) {
                return Some((slug, source_id.clone()));
            }
        }
        None
    }
}

fn extract_slug(path: &str, pattern: &str) -> Option<String> {
    let pattern_parts: Vec<&str> = pattern.split('{').collect();
    if pattern_parts.len() != 2 {
        return None;
    }

    let prefix = pattern_parts[0];
    if !path.starts_with(prefix) {
        return None;
    }

    let slug = path.trim_start_matches(prefix).trim_end_matches('/');
    if !slug.is_empty() {
        Some(slug.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_slug() {
        assert_eq!(
            extract_slug("/blog/my-post/", "/blog/{slug}"),
            Some("my-post".to_string())
        );
        assert_eq!(extract_slug("/about", "/{slug}"), Some("about".to_string()));
        assert_eq!(extract_slug("/api/v1/something", "/blog/{slug}"), None);
    }

    #[test]
    fn test_url_pattern_matching() {
        let matcher = StaticContentMatcher {
            patterns: vec![
                ("/blog/{slug}".to_string(), "blog".to_string()),
                ("/{slug}".to_string(), "web".to_string()),
            ],
        };

        assert_eq!(
            matcher.matches("/blog/my-post/"),
            Some(("my-post".to_string(), "blog".to_string()))
        );

        assert_eq!(
            matcher.matches("/about"),
            Some(("about".to_string(), "web".to_string()))
        );

        assert_eq!(matcher.matches("/api/v1/something"), None);
    }
}
