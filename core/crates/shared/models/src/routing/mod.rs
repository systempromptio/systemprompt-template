use crate::ContentConfig;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventMetadata {
    pub event_type: &'static str,
    pub event_category: &'static str,
    pub log_module: &'static str,
}

impl EventMetadata {
    pub const HTML_CONTENT: Self = Self {
        event_type: "page_view",
        event_category: "content",
        log_module: "page_view",
    };

    pub const API_REQUEST: Self = Self {
        event_type: "http_request",
        event_category: "api",
        log_module: "http_request",
    };

    pub const STATIC_ASSET: Self = Self {
        event_type: "asset_request",
        event_category: "static",
        log_module: "asset_request",
    };

    pub const NOT_FOUND: Self = Self {
        event_type: "not_found",
        event_category: "error",
        log_module: "not_found",
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteType {
    HtmlContent { source: String },
    ApiEndpoint { category: ApiCategory },
    StaticAsset { asset_type: AssetType },
    NotFound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiCategory {
    Content,
    Core,
    Agents,
    OAuth,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    JavaScript,
    Stylesheet,
    Image,
    Font,
    SourceMap,
    Other,
}

#[derive(Debug)]
pub struct RouteClassifier {
    content_config: Option<Arc<ContentConfig>>,
}

impl RouteClassifier {
    pub const fn new(content_config: Option<Arc<ContentConfig>>) -> Self {
        Self { content_config }
    }

    pub fn classify(&self, path: &str, _method: &str) -> RouteType {
        if Self::is_static_asset_path(path) {
            return RouteType::StaticAsset {
                asset_type: Self::determine_asset_type(path),
            };
        }

        if path.starts_with("/api/") {
            return RouteType::ApiEndpoint {
                category: Self::determine_api_category(path),
            };
        }

        if let Some(config) = &self.content_config {
            if config.is_html_page(path) {
                return RouteType::HtmlContent {
                    source: Self::determine_source(config, path),
                };
            }
        } else if !Self::is_static_asset_path(path) && !path.starts_with("/api/") {
            return RouteType::HtmlContent {
                source: "unknown".to_string(),
            };
        }

        RouteType::NotFound
    }

    pub fn should_track_analytics(&self, path: &str, method: &str) -> bool {
        if method == "OPTIONS" {
            return false;
        }

        match self.classify(path, method) {
            RouteType::HtmlContent { .. } => true,
            RouteType::ApiEndpoint { category } => {
                matches!(category, ApiCategory::Core | ApiCategory::Content)
            },
            RouteType::StaticAsset { .. } | RouteType::NotFound => false,
        }
    }

    pub fn is_html(&self, path: &str) -> bool {
        matches!(self.classify(path, "GET"), RouteType::HtmlContent { .. })
    }

    pub fn get_event_metadata(&self, path: &str, method: &str) -> EventMetadata {
        match self.classify(path, method) {
            RouteType::HtmlContent { .. } => EventMetadata::HTML_CONTENT,
            RouteType::ApiEndpoint { .. } => EventMetadata::API_REQUEST,
            RouteType::StaticAsset { .. } => EventMetadata::STATIC_ASSET,
            RouteType::NotFound => EventMetadata::NOT_FOUND,
        }
    }

    fn is_static_asset_path(path: &str) -> bool {
        if path.starts_with("/assets/")
            || path.starts_with("/.well-known/")
            || path.starts_with("/generated/")
            || path.starts_with("/files/")
        {
            return true;
        }

        matches!(
            Path::new(path).extension().and_then(|e| e.to_str()),
            Some(
                "js" | "css"
                    | "map"
                    | "ttf"
                    | "woff"
                    | "woff2"
                    | "otf"
                    | "png"
                    | "jpg"
                    | "jpeg"
                    | "svg"
                    | "ico"
                    | "webp"
            )
        ) || path == "/vite.svg"
            || path == "/favicon.ico"
    }

    fn determine_asset_type(path: &str) -> AssetType {
        match Path::new(path).extension().and_then(|e| e.to_str()) {
            Some("js") => AssetType::JavaScript,
            Some("css") => AssetType::Stylesheet,
            Some("png" | "jpg" | "jpeg" | "svg" | "ico" | "webp") => AssetType::Image,
            Some("ttf" | "woff" | "woff2" | "otf") => AssetType::Font,
            Some("map") => AssetType::SourceMap,
            _ => AssetType::Other,
        }
    }

    fn determine_api_category(path: &str) -> ApiCategory {
        if path.starts_with("/api/v1/content/") {
            ApiCategory::Content
        } else if path.starts_with("/api/v1/core/") {
            ApiCategory::Core
        } else if path.starts_with("/api/v1/agents/") {
            ApiCategory::Agents
        } else if path.starts_with("/api/v1/oauth/") {
            ApiCategory::OAuth
        } else {
            ApiCategory::Other
        }
    }

    fn determine_source(config: &ContentConfig, path: &str) -> String {
        if path == "/" {
            return "web".to_string();
        }

        for (source_name, source_config) in &config.content_sources {
            if !source_config.enabled {
                continue;
            }

            if let Some(sitemap) = &source_config.sitemap {
                if sitemap.enabled && ContentConfig::matches_url_pattern(&sitemap.url_pattern, path)
                {
                    return source_name.clone();
                }
            }
        }

        "unknown".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_homepage() {
        let classifier = RouteClassifier::new(None);
        assert!(matches!(
            classifier.classify("/", "GET"),
            RouteType::HtmlContent { .. }
        ));
    }

    #[test]
    fn test_classify_static_asset_js() {
        let classifier = RouteClassifier::new(None);
        assert!(matches!(
            classifier.classify("/assets/main.js", "GET"),
            RouteType::StaticAsset {
                asset_type: AssetType::JavaScript
            }
        ));
    }

    #[test]
    fn test_classify_static_asset_css() {
        let classifier = RouteClassifier::new(None);
        assert!(matches!(
            classifier.classify("/assets/style.css", "GET"),
            RouteType::StaticAsset {
                asset_type: AssetType::Stylesheet
            }
        ));
    }

    #[test]
    fn test_classify_api_content() {
        let classifier = RouteClassifier::new(None);
        assert!(matches!(
            classifier.classify("/api/v1/content/list", "GET"),
            RouteType::ApiEndpoint {
                category: ApiCategory::Content
            }
        ));
    }

    #[test]
    fn test_classify_api_core() {
        let classifier = RouteClassifier::new(None);
        assert!(matches!(
            classifier.classify("/api/v1/core/agents", "GET"),
            RouteType::ApiEndpoint {
                category: ApiCategory::Core
            }
        ));
    }

    #[test]
    fn test_should_track_html() {
        let classifier = RouteClassifier::new(None);
        assert!(classifier.should_track_analytics("/", "GET"));
        assert!(classifier.should_track_analytics("/blog/post", "GET"));
    }

    #[test]
    fn test_should_not_track_assets() {
        let classifier = RouteClassifier::new(None);
        assert!(!classifier.should_track_analytics("/assets/main.js", "GET"));
        assert!(!classifier.should_track_analytics("/favicon.ico", "GET"));
    }

    #[test]
    fn test_should_not_track_options() {
        let classifier = RouteClassifier::new(None);
        assert!(!classifier.should_track_analytics("/", "OPTIONS"));
    }

    #[test]
    fn test_should_track_core_api() {
        let classifier = RouteClassifier::new(None);
        assert!(classifier.should_track_analytics("/api/v1/core/agents", "GET"));
    }

    #[test]
    fn test_should_track_content_api() {
        let classifier = RouteClassifier::new(None);
        assert!(classifier.should_track_analytics("/api/v1/content/list", "GET"));
    }

    #[test]
    fn test_should_not_track_oauth_api() {
        let classifier = RouteClassifier::new(None);
        assert!(!classifier.should_track_analytics("/api/v1/oauth/token", "POST"));
    }
}
