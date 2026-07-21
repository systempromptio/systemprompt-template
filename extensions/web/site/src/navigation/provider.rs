//! Supplies header, footer, and branding navigation context to every public
//! page.

use std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;
use serde_json::Value;
use systemprompt::extension::prelude::*;

use super::config::{
    BrandingConfig, DocsSidebarSection, FooterConfig, HeaderNavConfig, NavigationConfig, SocialLink,
};

/// Template context injected on every page: header/footer navigation, branding,
/// and the fixed top-level app/blog/docs links consumed by the site partials.
#[derive(Debug, Serialize)]
struct NavigationContext<'a> {
    site: NavigationSite<'a>,
    nav: NavLinks,
}

#[derive(Debug, Serialize)]
struct NavigationSite<'a> {
    header_nav: &'a HeaderNavConfig,
    docs_sidebar: &'a [DocsSidebarSection],
    branding: &'a Option<BrandingConfig>,
    navigation: FooterNavigation<'a>,
}

#[derive(Debug, Serialize)]
struct FooterNavigation<'a> {
    footer: &'a FooterConfig,
    social: &'a [SocialLink],
}

#[derive(Debug, Serialize)]
struct NavLinks {
    #[serde(rename = "app_url")]
    app: &'static str,
    #[serde(rename = "blog_url")]
    blog: &'static str,
    #[serde(rename = "docs_url")]
    docs: &'static str,
}

#[derive(Debug)]
pub struct NavigationPageDataProvider {
    config: Arc<NavigationConfig>,
    branding: Option<BrandingConfig>,
}

impl NavigationPageDataProvider {
    #[must_use]
    pub const fn new(config: Arc<NavigationConfig>) -> Self {
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

    async fn provide_page_data(
        &self,
        _ctx: &PageContext<'_>,
    ) -> Result<Value, systemprompt::traits::ProviderError> {
        let context = NavigationContext {
            site: NavigationSite {
                header_nav: &self.config.header,
                docs_sidebar: &self.config.docs_sidebar,
                branding: &self.branding,
                navigation: FooterNavigation {
                    footer: &self.config.footer,
                    social: &self.config.social,
                },
            },
            nav: NavLinks {
                app: "/app",
                blog: "/blog",
                docs: "/documentation",
            },
        };
        Ok(serde_json::to_value(context)?)
    }

    fn priority(&self) -> u32 {
        10
    }
}
