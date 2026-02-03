use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use systemprompt::template_provider::{PageContext, PageDataProvider};

use super::config::{BrandingConfig, NavigationConfig};

pub struct NavigationPageDataProvider {
    config: Arc<NavigationConfig>,
    branding: Option<BrandingConfig>,
}

impl NavigationPageDataProvider {
    #[must_use]
    pub fn new(config: Arc<NavigationConfig>) -> Self {
        Self {
            config,
            branding: None,
        }
    }

    #[must_use]
    pub fn with_branding(mut self, branding: Option<BrandingConfig>) -> Self {
        self.branding = branding;
        self
    }
}

#[async_trait]
impl PageDataProvider for NavigationPageDataProvider {
    fn provider_id(&self) -> &'static str {
        "navigation"
    }

    fn applies_to_pages(&self) -> Vec<String> {
        vec![]
    }

    async fn provide_page_data(&self, _ctx: &PageContext<'_>) -> Result<Value> {
        Ok(serde_json::json!({
            "site": {
                "header_nav": &self.config.header,
                "docs_sidebar": &self.config.docs_sidebar,
                "branding": &self.branding,
                "navigation": {
                    "footer": &self.config.footer,
                    "social": &self.config.social
                }
            },
            "nav": {
                "app_url": "/app",
                "blog_url": "/blog",
                "docs_url": "/documentation"
            }
        }))
    }

    fn priority(&self) -> u32 {
        10
    }
}
