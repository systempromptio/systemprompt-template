use std::path::PathBuf;

use anyhow::Result;
use async_trait::async_trait;
use systemprompt::extension::prelude::*;

use super::config::FeaturePage;

pub struct FeaturePagePrerenderer {
    page: FeaturePage,
    page_type_id: String,
}

impl FeaturePagePrerenderer {
    #[must_use]
    pub fn new(page: FeaturePage) -> Self {
        let page_type_id = format!("feature-page:{}", page.slug);
        Self { page, page_type_id }
    }
}

#[async_trait]
impl PagePrerenderer for FeaturePagePrerenderer {
    fn page_type(&self) -> &str {
        &self.page_type_id
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
