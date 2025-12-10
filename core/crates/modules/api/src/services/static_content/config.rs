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

    pub const fn empty() -> Self {
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
    if slug.is_empty() {
        None
    } else {
        Some(slug.to_string())
    }
}
