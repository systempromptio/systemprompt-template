use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use systemprompt::extension::prelude::*;

use super::config::HomepageConfig;
use super::context::HomepageContext;

#[derive(Debug)]
pub struct HomepagePrerenderer {
    config: Arc<HomepageConfig>,
}

impl HomepagePrerenderer {
    #[must_use]
    pub const fn new(config: Arc<HomepageConfig>) -> Self {
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
    ) -> Result<Option<PageRenderSpec>, systemprompt::traits::ProviderError> {
        let base_data = serde_json::to_value(HomepageContext::new(&self.config))?;

        let output_path = PathBuf::from("index.html");

        Ok(Some(PageRenderSpec::new(
            "homepage",
            base_data,
            output_path,
        )))
    }
}
