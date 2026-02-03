use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use systemprompt::template_provider::{PagePrepareContext, PagePrerenderer, PageRenderSpec};

use super::config::HomepageConfig;

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
        150 // Must be higher than core's default (100)
    }

    async fn prepare(&self, _ctx: &PagePrepareContext<'_>) -> Result<Option<PageRenderSpec>> {
        let config_value = serde_json::to_value(&*self.config)?;

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
