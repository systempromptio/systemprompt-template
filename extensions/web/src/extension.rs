//! The `WebExtension` composition member and the configuration it loads from
//! the profile's services tree.
//!
//! Config loading lives in `systemprompt_web_site::config_loader`, beside the
//! types it deserialises; every read takes the services directory from the
//! member's [`WebSettings`], so nothing is cached process-wide.

use std::sync::Arc;

use crate::WebSettings;
use crate::config::BlogConfigValidated;

/// The template's web member: schemas, jobs, content providers, governance
/// providers and the web-root route set.
#[derive(Debug, Default, Clone, Copy)]
pub struct WebExtension;

impl WebExtension {
    pub const ID: &'static str = "web";

    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    // Why: a load failure is logged and treated as "no config", so link
    // generation and the content APIs run unconfigured rather than failing
    // boot.
    #[must_use]
    pub fn blog_config(settings: &WebSettings) -> Option<Arc<BlogConfigValidated>> {
        match BlogConfigValidated::load_for_services(&settings.services_path) {
            Ok(config) => config,
            Err(errors) => {
                tracing::error!(
                    error = %errors,
                    "Blog config error: link generation and content APIs will run unconfigured"
                );
                None
            },
        }
    }
}
