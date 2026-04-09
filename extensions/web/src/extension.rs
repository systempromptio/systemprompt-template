use std::sync::{Arc, OnceLock};

use crate::config::BlogConfigValidated;
use crate::config_loader::{self, ConfigError};
use crate::features::FeaturesConfig;
use crate::homepage::HomepageConfig;
use crate::jobs::ContentIngestionJob;
use crate::navigation::NavigationConfig;

use systemprompt::extension::prelude::*;

static NAVIGATION_CONFIG: OnceLock<Option<Arc<NavigationConfig>>> = OnceLock::new();
static HOMEPAGE_CONFIG: OnceLock<Option<Arc<HomepageConfig>>> = OnceLock::new();
static FEATURES_CONFIG: OnceLock<Option<Arc<FeaturesConfig>>> = OnceLock::new();

#[derive(Debug, Default, Clone)]
pub struct WebExtension {
    pub(crate) validated_config: Option<Arc<BlogConfigValidated>>,
}

impl WebExtension {
    pub const PREFIX: &'static str = "web";

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn with_validated_config(config: Arc<BlogConfigValidated>) -> Self {
        Self {
            validated_config: Some(config),
        }
    }

    #[must_use]
    pub const fn validated_config(&self) -> Option<&Arc<BlogConfigValidated>> {
        self.validated_config.as_ref()
    }

    #[must_use]
    pub const fn base_path() -> &'static str {
        "/api/v1/links"
    }

    #[must_use]
    pub const fn ingestion_job() -> ContentIngestionJob {
        ContentIngestionJob
    }

    #[must_use]
    pub fn navigation_config() -> Option<Arc<NavigationConfig>> {
        log_and_discard_err(
            &NAVIGATION_CONFIG,
            config_loader::load_navigation_config,
            "Navigation config error",
        )
    }

    #[must_use]
    pub fn homepage_config() -> Option<Arc<HomepageConfig>> {
        log_and_discard_err(
            &HOMEPAGE_CONFIG,
            config_loader::load_homepage_config,
            "Homepage config error",
        )
    }

    #[must_use]
    pub fn features_config() -> Option<Arc<FeaturesConfig>> {
        log_and_discard_err(
            &FEATURES_CONFIG,
            config_loader::load_features_config,
            "Features config error",
        )
    }
}

fn log_and_discard_err<T: Clone>(
    lock: &OnceLock<Option<T>>,
    init: fn() -> Result<Option<T>, ConfigError>,
    msg: &str,
) -> Option<T> {
    lock.get_or_init(|| match init() {
        Ok(val) => val,
        Err(e) => {
            tracing::error!(error = %e, "{msg}");
            None
        }
    })
    .clone()
}

register_extension!(WebExtension);

pub type BlogExtension = WebExtension;
