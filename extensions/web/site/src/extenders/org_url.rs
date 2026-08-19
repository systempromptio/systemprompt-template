//! Injects the deployment's canonical organisation URL into template context.

use async_trait::async_trait;
// JSON: template context crosses the extender trait as a Value.
use serde_json::Value;
use systemprompt::models::Config;
use systemprompt::template_provider::{ExtenderContext, TemplateDataExtender};

#[derive(Debug, Clone, Copy)]
pub struct OrgUrlExtender;

impl OrgUrlExtender {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for OrgUrlExtender {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TemplateDataExtender for OrgUrlExtender {
    fn extender_id(&self) -> &'static str {
        "org-url"
    }

    fn applies_to(&self) -> Vec<String> {
        vec![]
    }

    fn priority(&self) -> u32 {
        10
    }

    async fn extend(
        &self,
        _ctx: &ExtenderContext<'_>,
        // JSON: required by trait contract
        data: &mut Value,
    ) -> Result<(), systemprompt::traits::ProviderError> {
        // Why: lint-ok: error-adapt — core's ProviderError::Internal(String) is the
        // trait's only failure channel; the nearby format! builds URLs, not errors.
        let config = Config::get()
            .map_err(|e| systemprompt::traits::ProviderError::Internal(e.to_string()))?;
        let org_url = &config.api_external_url;

        let default_image = format!("{org_url}/files/images/logo.png");
        let org_logo = format!("{org_url}/files/images/logo.svg");

        if let Some(obj) = data.as_object_mut() {
            obj.insert("ORG_URL".to_owned(), Value::String(org_url.clone()));
            obj.insert("ORG_LOGO".to_owned(), Value::String(org_logo));
            obj.insert("DEFAULT_IMAGE".to_owned(), Value::String(default_image));
        }

        Ok(())
    }
}

systemprompt_web_shared::submit_extender!(OrgUrlExtender::new());
