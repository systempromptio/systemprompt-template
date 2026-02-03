use std::sync::Arc;

use systemprompt::models::AppPaths;
use thiserror::Error;

use crate::features::{FeaturePage, FeaturesConfig};
use crate::homepage::HomepageConfig;
use crate::navigation::{BrandingConfig, NavigationConfig};

#[derive(Debug, Clone, Error)]
pub enum ConfigError {
    #[error("Failed to parse {config_name}: {message}")]
    Parse { config_name: String, message: String },
}

pub fn load_navigation_config() -> Result<Option<Arc<NavigationConfig>>, ConfigError> {
    let Some(nav_value) = load_config_section("navigation.yaml")? else {
        return Ok(None);
    };

    let nav_config: NavigationConfig = serde_yaml::from_value(nav_value).map_err(|e| {
        ConfigError::Parse {
            config_name: "navigation.yaml".to_string(),
            message: e.to_string(),
        }
    })?;

    tracing::info!("Loaded navigation config from config/navigation.yaml");

    Ok(Some(Arc::new(nav_config)))
}

pub fn load_homepage_config() -> Result<Option<Arc<HomepageConfig>>, ConfigError> {
    let Some(homepage_value) = load_config_section("homepage.yaml")? else {
        return Ok(None);
    };

    let homepage_config: HomepageConfig =
        serde_yaml::from_value(homepage_value).map_err(|e| ConfigError::Parse {
            config_name: "homepage.yaml".to_string(),
            message: e.to_string(),
        })?;

    tracing::info!("Loaded homepage config from config/homepage.yaml");

    Ok(Some(Arc::new(homepage_config)))
}

pub fn load_branding_config() -> Result<Option<BrandingConfig>, ConfigError> {
    let Some(theme_value) = load_config_section("theme.yaml")? else {
        return Ok(None);
    };

    let Some(branding_value) = theme_value.get("branding") else {
        return Ok(None);
    };

    let branding_config: BrandingConfig =
        serde_yaml::from_value(branding_value.clone()).map_err(|e| ConfigError::Parse {
            config_name: "theme.yaml (branding section)".to_string(),
            message: e.to_string(),
        })?;

    tracing::info!("Loaded branding config from config/theme.yaml");

    Ok(Some(branding_config))
}

pub fn load_features_config() -> Result<Option<Arc<FeaturesConfig>>, ConfigError> {
    let paths = match AppPaths::get() {
        Ok(p) => p,
        Err(e) => {
            tracing::debug!("AppPaths not available for features config: {e}");
            return Ok(None);
        }
    };

    let features_dir = paths.system().services().join("web/config/features");

    let entries = match std::fs::read_dir(&features_dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::debug!(
                path = %features_dir.display(),
                "Features config directory does not exist"
            );
            return Ok(None);
        }
        Err(e) => {
            return Err(ConfigError::Parse {
                config_name: features_dir.display().to_string(),
                message: format!("Failed to read directory: {e}"),
            });
        }
    };

    let mut pages: Vec<FeaturePage> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        if path.extension().is_none_or(|ext| ext != "yaml") {
            continue;
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("{}: failed to read: {e}", path.display()));
                continue;
            }
        };

        let page: FeaturePage = match serde_yaml::from_str(&content) {
            Ok(p) => p,
            Err(e) => {
                errors.push(format!("{}: failed to parse: {e}", path.display()));
                continue;
            }
        };

        pages.push(page);
    }

    if !errors.is_empty() {
        return Err(ConfigError::Parse {
            config_name: "features".to_string(),
            message: errors.join("; "),
        });
    }

    if pages.is_empty() {
        tracing::debug!("No feature pages loaded");
        return Ok(None);
    }

    pages.sort_by(|a, b| a.slug.cmp(&b.slug));

    tracing::info!(
        page_count = pages.len(),
        "Loaded features config from config/features/"
    );

    Ok(Some(Arc::new(FeaturesConfig { pages })))
}

fn load_config_section(filename: &str) -> Result<Option<serde_yaml::Value>, ConfigError> {
    let paths = match AppPaths::get() {
        Ok(p) => p,
        Err(e) => {
            tracing::debug!("AppPaths not available for config section: {e}");
            return Ok(None);
        }
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
        }
        Err(e) => {
            return Err(ConfigError::Parse {
                config_name: filename.to_string(),
                message: format!("Failed to read file: {e}"),
            });
        }
    };

    serde_yaml::from_str(&yaml_content)
        .map(Some)
        .map_err(|e| ConfigError::Parse {
            config_name: filename.to_string(),
            message: e.to_string(),
        })
}
