use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use systemprompt::template_provider::{PagePrepareContext, PagePrerenderer, PageRenderSpec};

use super::config::FeaturePage;

pub struct FeaturePagePrerenderer {
    page: FeaturePage,
}

impl FeaturePagePrerenderer {
    #[must_use]
    pub fn new(page: FeaturePage) -> Self {
        Self { page }
    }
}

#[async_trait]
impl PagePrerenderer for FeaturePagePrerenderer {
    fn page_type(&self) -> &'static str {
        "feature-page"
    }

    fn priority(&self) -> u32 {
        50
    }

    async fn prepare(&self, ctx: &PagePrepareContext<'_>) -> Result<Option<PageRenderSpec>> {
        let page_data = serde_json::to_value(&self.page)?;

        let base_data = serde_json::json!({
            "feature": page_data,
            "site": ctx.web_config,
        });

        let output_path = PathBuf::from(format!("features/{}/index.html", self.page.slug));

        Ok(Some(PageRenderSpec::new(
            "feature-page",
            base_data,
            output_path,
        )))
    }
}
