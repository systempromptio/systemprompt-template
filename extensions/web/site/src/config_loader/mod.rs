//! Loading of the `services/web/` YAML tree.
//!
//! Lives beside the config types it deserialises, so adding a YAML-backed
//! page section is one edit in this crate. Every load takes the services
//! directory it reads from; nothing is cached process-wide. A load failure is
//! logged and served as "section absent", never a crash. The web member loads
//! each section once, in its provide fns and factories, after admission.

use std::path::Path;
use std::sync::Arc;

mod adfs;

use adfs::load_adfs_config;

use thiserror::Error;

#[doc(hidden)]
pub mod skills;

pub(crate) use skills::load_skills_page_config;


use crate::homepage::HomepageConfig;
use crate::navigation::{BrandingConfig, NavigationConfig};
use crate::skills_page::SkillsPageConfig;

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

fn load_navigation_config(services: &Path) -> Result<Option<Arc<NavigationConfig>>, ConfigError> {
    let Some(nav_value) = load_config_section(services, "navigation.yaml")? else {
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

fn load_homepage_config(services: &Path) -> Result<Option<Arc<HomepageConfig>>, ConfigError> {
    let Some(homepage_value) = load_config_section(services, "homepage.yaml")? else {
        return Ok(None);
    };

    let homepage_config: HomepageConfig =
        serde_yaml::from_value(homepage_value).map_err(|e| ConfigError::Parse {
            config_name: "homepage.yaml".to_owned(),
            message: e.to_string(),
        })?;

    tracing::info!("Loaded homepage config from config/homepage.yaml");

    Ok(Some(Arc::new(homepage_config)))
}

fn load_branding_config(services: &Path) -> Result<Option<BrandingConfig>, ConfigError> {
    let Some(theme_value) = load_config_section(services, "theme.yaml")? else {
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

// Why: Both router builds and the HTTP contract suite need the engine
// configured the same way; the templates read `branding.*` under strict mode,
// so an engine built without it fails to render every page that has one.
#[must_use]
pub fn branding_config(services: &Path) -> Option<BrandingConfig> {
    log_and_discard_err(load_branding_config(services), "Branding config error")
}

#[must_use]
pub fn navigation_config(services: &Path) -> Option<Arc<NavigationConfig>> {
    log_and_discard_err(load_navigation_config(services), "Navigation config error")
}

#[must_use]
pub fn homepage_config(services: &Path) -> Option<Arc<HomepageConfig>> {
    log_and_discard_err(load_homepage_config(services), "Homepage config error")
}

#[must_use]
pub fn skills_page_config(services: &Path) -> Option<Arc<SkillsPageConfig>> {
    log_and_discard_err(
        load_skills_page_config(services),
        "Skills page config error",
    )
}

#[must_use]
// Why: an `adfs.yaml` that is present but unreadable is a broken sign-in
// configuration, so the error reaches the caller and refuses the boot rather
// than leaving the instance without SSO.
pub fn adfs_config(
    services: &Path,
    api_external_url: &str,
) -> Result<Option<Arc<systemprompt_web_admin::AdfsConfig>>, ConfigError> {
    load_adfs_config(services, api_external_url)
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DevLoginConfig {
    #[serde(default)]
    enabled: bool,
}

/// Whether `services/web/config/dev-login.yaml` asks for the development
/// login link.
pub fn dev_login_requested(services: &Path) -> Result<bool, ConfigError> {
    let filename = "dev-login.yaml";
    let Some(value) = load_config_section(services, filename)? else {
        return Ok(false);
    };
    let config: DevLoginConfig = serde_yaml::from_value(value).map_err(|e| ConfigError::Parse {
        config_name: filename.to_owned(),
        message: e.to_string(),
    })?;
    Ok(config.enabled)
}

#[doc(hidden)]
pub fn log_and_discard_err<T>(loaded: Result<Option<T>, ConfigError>, msg: &str) -> Option<T> {
    match loaded {
        Ok(val) => val,
        Err(error) => {
            tracing::error!(
                error = %error,
                config = msg,
                "Config failed to load; its pages and sections will not render"
            );
            None
        },
    }
}

pub(super) fn load_config_section(
    services: &Path,
    filename: &str,
) -> Result<Option<serde_yaml::Value>, ConfigError> {
    let config_path = services.join(format!("web/config/{filename}"));

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
