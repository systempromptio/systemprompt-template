use anyhow::Result;
use async_trait::async_trait;
use serde_json::{json, Value};
use systemprompt::models::Config;
use systemprompt::template_provider::{ExtenderContext, TemplateDataExtender};

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

    async fn extend(&self, _ctx: &ExtenderContext<'_>, data: &mut Value) -> Result<()> {
        let config = Config::get()?;
        let org_url = &config.api_external_url;

        let default_image = format!("{org_url}/files/images/logo.png");
        let org_logo = format!("{org_url}/files/images/logo.svg");

        if let Some(obj) = data.as_object_mut() {
            obj.insert("ORG_URL".to_string(), json!(org_url));
            obj.insert("ORG_LOGO".to_string(), json!(org_logo));
            obj.insert("DEFAULT_IMAGE".to_string(), json!(default_image));
        }

        Ok(())
    }
}
