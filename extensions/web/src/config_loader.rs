use std::sync::Arc;

use systemprompt::models::AppPaths;

use crate::features::{FeaturePage, FeaturesConfig};
use crate::homepage::HomepageConfig;
use crate::navigation::{BrandingConfig, NavigationConfig};

pub fn load_navigation_config() -> Option<Arc<NavigationConfig>> {
    let nav_value = load_config_section("navigation.yaml")?;

    let nav_config: NavigationConfig = match serde_yaml::from_value(nav_value) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                error = %e,
                "Failed to deserialize navigation config"
            );
            return None;
        }
    };

    tracing::info!("Loaded navigation config from config/navigation.yaml");

    Some(Arc::new(nav_config))
}

pub fn load_homepage_config() -> Option<Arc<HomepageConfig>> {
    let homepage_value = load_config_section("homepage.yaml")?;

    let homepage_config: HomepageConfig = match serde_yaml::from_value(homepage_value) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                error = %e,
                "Failed to deserialize homepage config"
            );
            return None;
        }
    };

    tracing::info!("Loaded homepage config from config/homepage.yaml");

    Some(Arc::new(homepage_config))
}

pub fn load_branding_config() -> Option<BrandingConfig> {
    let theme_value = load_config_section("theme.yaml")?;

    let branding_value = theme_value.get("branding")?;

    let branding_config: BrandingConfig = match serde_yaml::from_value(branding_value.clone()) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(
                error = %e,
                "Failed to deserialize branding config from theme.yaml"
            );
            return None;
        }
    };

    tracing::info!("Loaded branding config from config/theme.yaml");

    Some(branding_config)
}

pub fn load_features_config() -> Option<Arc<FeaturesConfig>> {
    let paths = match AppPaths::get() {
        Ok(p) => p,
        Err(e) => {
            tracing::debug!("AppPaths not available for features config: {e}");
            return None;
        }
    };

    let features_dir = paths.system().services().join("web/config/features");

    let entries = match std::fs::read_dir(&features_dir) {
        Ok(entries) => entries,
        Err(e) => {
            tracing::debug!(
                path = %features_dir.display(),
                error = %e,
                "Failed to read features config directory"
            );
            return None;
        }
    };

    let mut pages: Vec<FeaturePage> = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();

        if path.extension().is_none_or(|ext| ext != "yaml") {
            continue;
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = %e,
                    "Failed to read feature config file"
                );
                continue;
            }
        };

        let page: FeaturePage = match serde_yaml::from_str(&content) {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(
                    path = %path.display(),
                    error = %e,
                    "Failed to deserialize feature config"
                );
                continue;
            }
        };

        pages.push(page);
    }

    if pages.is_empty() {
        tracing::debug!("No feature pages loaded");
        return None;
    }

    pages.sort_by(|a, b| a.slug.cmp(&b.slug));

    tracing::info!(
        page_count = pages.len(),
        "Loaded features config from config/features/"
    );

    Some(Arc::new(FeaturesConfig { pages }))
}

fn load_config_section(filename: &str) -> Option<serde_yaml::Value> {
    let paths = match AppPaths::get() {
        Ok(p) => p,
        Err(e) => {
            tracing::debug!("AppPaths not available for config section: {e}");
            return None;
        }
    };

    let config_path = paths
        .system()
        .services()
        .join(format!("web/config/{filename}"));

    let yaml_content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::debug!(
                path = %config_path.display(),
                error = %e,
                "Failed to read config section"
            );
            return None;
        }
    };

    match serde_yaml::from_str(&yaml_content) {
        Ok(v) => Some(v),
        Err(e) => {
            tracing::warn!(
                path = %config_path.display(),
                error = %e,
                "Failed to parse config section"
            );
            None
        }
    }
}
