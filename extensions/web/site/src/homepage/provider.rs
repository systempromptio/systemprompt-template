use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use systemprompt::extension::prelude::*;

use super::config::HomepageConfig;
use systemprompt_web_shared::error::MarketplaceError;

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
        vec!["homepage".to_string()]
    }

    async fn provide_page_data(&self, _ctx: &PageContext<'_>) -> anyhow::Result<Value> {
        let config = (*self.config).clone();
        let config_value = serde_json::to_value(&config).map_err(MarketplaceError::Json)?;
        Ok(serde_json::json!({ "site": { "homepage": config_value } }))
    }

    fn priority(&self) -> u32 {
        50
    }
}
