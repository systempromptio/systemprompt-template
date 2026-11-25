//! The runtime settings the web members read, projected once from the
//! profile's runtime configuration by `WebExtension`'s provide fn.

use std::path::PathBuf;
use std::sync::Arc;

use systemprompt::manifest::Config;

/// What the web routes and jobs need of the runtime configuration: no
/// credential, no database URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WebSettings {
    pub sitename: String,
    pub api_external_url: String,
    pub jwt_issuer: String,
    pub use_https: bool,
    pub system_path: PathBuf,
    pub services_path: PathBuf,
    pub admin_template_dir: PathBuf,
}

impl WebSettings {
    #[must_use]
    pub fn from_config(config: &Config, admin_template_dir: PathBuf) -> Arc<Self> {
        Arc::new(Self {
            sitename: config.sitename.clone(),
            api_external_url: config.api_external_url.clone(),
            jwt_issuer: config.jwt_issuer.clone(),
            use_https: config.use_https,
            system_path: PathBuf::from(&config.system_path),
            services_path: PathBuf::from(&config.services_path),
            admin_template_dir,
        })
    }
}
