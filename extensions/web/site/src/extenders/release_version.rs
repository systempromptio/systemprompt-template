//! Injects the running build's crate version into every template context.

use async_trait::async_trait;
// JSON: template context crosses the extender trait as a Value.
use serde_json::Value;
use systemprompt::template_provider::{ExtenderContext, TemplateDataExtender};

const RELEASE_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, Copy)]
pub struct ReleaseVersionExtender;

impl ReleaseVersionExtender {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for ReleaseVersionExtender {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TemplateDataExtender for ReleaseVersionExtender {
    fn extender_id(&self) -> &'static str {
        "release-version"
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
        if let Some(obj) = data.as_object_mut() {
            obj.insert(
                "RELEASE_VERSION".to_owned(),
                Value::String(RELEASE_VERSION.to_owned()),
            );
        }

        Ok(())
    }
}

systemprompt_web_shared::submit_extender!(ReleaseVersionExtender::new());
