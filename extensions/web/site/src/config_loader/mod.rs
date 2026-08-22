//! Bootstrap-time loading of the `services/web/` YAML tree.
//!
//! Lives beside the config types it deserialises, so adding a YAML-backed
//! page section is one edit in this crate. Each section is loaded once into
//! a `OnceLock`; a load failure is logged and served as "section absent",
//! never a crash. Runs at extension construction, before any request is
//! served, so the file-system reads here are not on a hot path.

use std::sync::{Arc, OnceLock};

use systemprompt::config::ProfileBootstrap;
use systemprompt::models::AppPaths;
use thiserror::Error;

mod features;
use features::load_features_config;

static BRANDING_CONFIG: OnceLock<Result<Option<BrandingConfig>, String>> = OnceLock::new();

fn load_app_paths() -> Result<AppPaths, ConfigError> {
    let profile =
        ProfileBootstrap::get().map_err(|e| ConfigError::PathsUnavailable(e.to_string()))?;
    AppPaths::from_profile(&profile.paths, profile.path_resolution())
        .map_err(|e| ConfigError::PathsUnavailable(e.to_string()))
}

use crate::features::FeaturePagesConfig;
use crate::homepage::HomepageConfig;
use crate::navigation::{BrandingConfig, NavigationConfig};

#[derive(Debug, Clone, Error)]
pub enum ConfigError {
    #[error("Failed to parse {config_name}: {message}")]
    Parse {
        config_name: String,
        message: String,
    },

    #[error("Application paths unavailable: {0}")]
    PathsUnavailable(String),
}

fn load_navigation_config() -> Result<Option<Arc<NavigationConfig>>, ConfigError> {
    let Some(nav_value) = load_config_section("navigation.yaml")? else {
        return Ok(None);
    };

    let nav_config: NavigationConfig =
        serde_yaml::from_value(nav_value).map_err(|e| ConfigError::Parse {
            config_name: "navigation.yaml".to_owned(),
            message: e.to_string(),
        })?;

    tracing::info!("Loaded navigation config from config/navigation.yaml");

    Ok(Some(Arc::new(nav_config)))
}

fn load_homepage_config() -> Result<Option<Arc<HomepageConfig>>, ConfigError> {
    let Some(homepage_value) = load_config_section("homepage.yaml")? else {
        return Ok(None);
    };

    let mut homepage_config: HomepageConfig =
        serde_yaml::from_value(homepage_value).map_err(|e| ConfigError::Parse {
            config_name: "homepage.yaml".to_owned(),
            message: e.to_string(),
        })?;

    if let Ok(paths) = load_app_paths() {
        populate_demo_showcase(
            &mut homepage_config,
            paths.system().root().join("demo").as_path(),
        );
    }

    tracing::info!("Loaded homepage config from config/homepage.yaml");

    Ok(Some(Arc::new(homepage_config)))
}

fn populate_demo_showcase(homepage_config: &mut HomepageConfig, demo_root: &std::path::Path) {
    match crate::homepage::demo_scanner::scan_demos(demo_root) {
        Ok(mut scanned) => {
            if let Some(existing) = homepage_config.demos.as_ref() {
                if existing.title.is_some() {
                    scanned.title.clone_from(&existing.title);
                }
                if existing.subtitle.is_some() {
                    scanned.subtitle.clone_from(&existing.subtitle);
                }
            }
            let total_categories: usize = scanned.pillars.iter().map(|p| p.categories.len()).sum();
            tracing::info!(
                pillars = scanned.pillars.len(),
                categories = total_categories,
                "Scanned demo/ for homepage showcase"
            );
            homepage_config.demos = Some(scanned);
        },
        Err(e) => {
            tracing::warn!(
                error = %e,
                path = %demo_root.display(),
                "Failed to scan demo/ directory — homepage will render without demo cards"
            );
        },
    }
}

fn load_branding_config() -> Result<Option<BrandingConfig>, ConfigError> {
    let Some(theme_value) = load_config_section("theme.yaml")? else {
        return Ok(None);
    };

    let Some(branding_value) = theme_value.get("branding") else {
        return Ok(None);
    };

    let branding_config: BrandingConfig =
        serde_yaml::from_value(branding_value.clone()).map_err(|e| ConfigError::Parse {
            config_name: "theme.yaml (branding section)".to_owned(),
            message: e.to_string(),
        })?;

    tracing::info!("Loaded branding config from config/theme.yaml");

    Ok(Some(branding_config))
}

// Why: Branding as the server resolves it, with a failure to load treated as
// "no branding" rather than fatal.
//
// Both router builds and the HTTP contract suite need the engine configured
// the same way; the templates read `branding.*` under strict mode, so an
// engine built without it fails to render every page that has one.
//
// Cached: the router build and each prerender context ask for branding
// independently, and re-reading theme.yaml per caller is pure waste.
#[must_use]
pub fn branding_config() -> Option<BrandingConfig> {
    log_and_discard_err(
        &BRANDING_CONFIG,
        load_branding_config,
        "Branding config error",
    )
}

static NAVIGATION_CONFIG: OnceLock<Result<Option<Arc<NavigationConfig>>, String>> = OnceLock::new();
static HOMEPAGE_CONFIG: OnceLock<Result<Option<Arc<HomepageConfig>>, String>> = OnceLock::new();
static FEATURES_CONFIG: OnceLock<Result<Option<Arc<FeaturePagesConfig>>, String>> = OnceLock::new();

#[must_use]
pub fn navigation_config() -> Option<Arc<NavigationConfig>> {
    log_and_discard_err(
        &NAVIGATION_CONFIG,
        load_navigation_config,
        "Navigation config error",
    )
}

#[must_use]
pub fn homepage_config() -> Option<Arc<HomepageConfig>> {
    log_and_discard_err(
        &HOMEPAGE_CONFIG,
        load_homepage_config,
        "Homepage config error",
    )
}

#[must_use]
pub fn features_config() -> Option<Arc<FeaturePagesConfig>> {
    log_and_discard_err(
        &FEATURES_CONFIG,
        load_features_config,
        "Features config error",
    )
}

#[doc(hidden)]
pub fn log_and_discard_err<T: Clone>(
    lock: &OnceLock<Result<Option<T>, String>>,
    init: fn() -> Result<Option<T>, ConfigError>,
    msg: &str,
) -> Option<T> {
    match lock.get_or_init(|| init().map_err(|e| e.to_string())) {
        Ok(val) => val.clone(),
        Err(message) => {
            tracing::error!(
                error = %message,
                "{msg}: config failed to load; its pages and sections will not render"
            );
            None
        },
    }
}

fn load_config_section(filename: &str) -> Result<Option<serde_yaml::Value>, ConfigError> {
    let paths = match load_app_paths() {
        Ok(p) => p,
        Err(e) => {
            tracing::debug!(error = %e, "AppPaths not available for config section");
            return Ok(None);
        },
    };

    let config_path = paths
        .system()
        .services()
        .join(format!("web/config/{filename}"));

    let yaml_content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::debug!(
                path = %config_path.display(),
                "Config file does not exist"
            );
            return Ok(None);
        },
        Err(e) => {
            return Err(ConfigError::Parse {
                config_name: filename.to_owned(),
                message: format!("Failed to read file: {e}"),
            });
        },
    };

    serde_yaml::from_str(&yaml_content)
        .map(Some)
        .map_err(|e| ConfigError::Parse {
            config_name: filename.to_owned(),
            message: e.to_string(),
        })
}
