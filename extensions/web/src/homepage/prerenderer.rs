use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use systemprompt::extension::prelude::*;

use super::config::HomepageConfig;
use crate::error::MarketplaceError;

pub struct HomepagePrerenderer {
    config: Arc<HomepageConfig>,
}

impl HomepagePrerenderer {
    #[must_use]
    pub fn new(config: Arc<HomepageConfig>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl PagePrerenderer for HomepagePrerenderer {
    fn page_type(&self) -> &'static str {
        "homepage"
    }

    fn priority(&self) -> u32 {
        150
    }

    async fn prepare(
        &self,
        _ctx: &PagePrepareContext<'_>,
    ) -> anyhow::Result<Option<PageRenderSpec>> {
        let config_value = serde_json::to_value(&*self.config).map_err(MarketplaceError::Json)?;

        let base_data = serde_json::json!({
            "site": {
                "homepage": config_value,
            },
        });

        let output_path = PathBuf::from("index.html");

        Ok(Some(PageRenderSpec::new(
            "homepage",
            base_data,
            output_path,
        )))
    }
}
