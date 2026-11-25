//! Loading of the `services/web/config/` YAML the console needs on every
//! flavour: the branding section of `theme.yaml`, the ADFS SSO config and
//! the dashboard.
//!
//! Every load takes the services directory it reads from; nothing is cached
//! process-wide. A load failure is logged and served as "section absent",
//! never a crash. The public-site sections are loaded by
//! `systemprompt_web_site::config_loader` on top of [`load_config_section`].

use std::path::{Path, PathBuf};
use std::sync::Arc;

mod adfs;

use thiserror::Error;

use crate::BrandingConfig;
use crate::adfs::AdfsConfig;
use crate::dashboard::DashboardConfig;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("required config {} is missing", path.display())]
    Missing { path: PathBuf },
    #[error("failed to parse {config_name}")]
    Parse {
        config_name: String,
        #[source]
        source: serde_yaml::Error,
    },
}

fn load_branding_config(services: &Path) -> Result<Option<BrandingConfig>, ConfigError> {
    let Some(theme_value) = load_config_section(services, "theme.yaml")? else {
        return Ok(None);
    };
    let Some(branding_value) = theme_value.get("branding") else {
        return Ok(None);
    };
    let branding_config: BrandingConfig =
        parse_section(branding_value.clone(), "theme.yaml (branding section)")?;
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
pub fn adfs_config(services: &Path, api_external_url: &str) -> Option<Arc<AdfsConfig>> {
    log_and_discard_err(
        adfs::load_adfs_config(services, api_external_url),
        "ADFS config error",
    )
}

pub fn dashboard_config(services: &Path) -> Result<DashboardConfig, ConfigError> {
    const FILE: &str = "dashboard.yaml";
    let value = load_config_section(services, FILE)?.ok_or_else(|| ConfigError::Missing {
        path: services.join("web/config").join(FILE),
    })?;
    parse_section(value, FILE)
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

pub fn parse_section<T: serde::de::DeserializeOwned>(
    value: serde_yaml::Value,
    config_name: &str,
) -> Result<T, ConfigError> {
    serde_yaml::from_value(value).map_err(|source| ConfigError::Parse {
        config_name: config_name.to_owned(),
        source,
    })
}

pub fn load_config_section(
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
        Err(source) => {
            return Err(ConfigError::Read {
                path: config_path,
                source,
            });
        },
    };
    serde_yaml::from_str(&yaml_content)
        .map(Some)
        .map_err(|source| ConfigError::Parse {
            config_name: filename.to_owned(),
            source,
        })
}
