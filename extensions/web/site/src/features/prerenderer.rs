use std::path::PathBuf;

use async_trait::async_trait;
use serde::Serialize;
use systemprompt::extension::prelude::*;
use systemprompt::models::WebConfig;

use super::config::FeaturePage;

/// Template context for a prerendered feature page (`feature-page.html`): the
/// feature definition under `feature.*` and the site-wide web config under
/// `site.*`.
#[derive(Debug, Serialize)]
struct FeaturePageContext<'a> {
    feature: &'a FeaturePage,
    site: &'a WebConfig,
}

#[derive(Debug)]
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

    async fn prepare(
        &self,
        ctx: &PagePrepareContext<'_>,
    ) -> Result<Option<PageRenderSpec>, systemprompt::traits::ProviderError> {
        let base_data = serde_json::to_value(FeaturePageContext {
            feature: &self.page,
            site: ctx.web_config,
        })?;

        let output_path = PathBuf::from(format!("features/{}/index.html", self.page.slug));

        Ok(Some(PageRenderSpec::new(
            "feature-page",
            base_data,
            output_path,
        )))
    }
}
