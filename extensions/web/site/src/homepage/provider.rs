//! Runtime page data provider for the homepage route.

use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use systemprompt::extension::prelude::*;

use super::config::HomepageConfig;
use super::context::HomepageContext;

#[derive(Debug)]
pub struct HomepagePageDataProvider {
    config: Arc<HomepageConfig>,
}

impl HomepagePageDataProvider {
    #[must_use]
    pub const fn new(config: Arc<HomepageConfig>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl PageDataProvider for HomepagePageDataProvider {
    fn provider_id(&self) -> &'static str {
        "homepage"
    }

    fn applies_to_pages(&self) -> Vec<String> {
        vec!["homepage".to_owned()]
    }

    async fn provide_page_data(
        &self,
        _ctx: &PageContext<'_>,
    ) -> Result<Value, systemprompt::traits::ProviderError> {
        Ok(serde_json::to_value(HomepageContext::new(&self.config))?)
    }

    fn priority(&self) -> u32 {
        50
    }
}
