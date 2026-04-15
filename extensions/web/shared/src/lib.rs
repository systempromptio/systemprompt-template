pub mod config;
pub mod config_errors;
pub mod error;
pub mod models;
mod utils;

pub use utils::html_escape;

pub mod branding {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Default, Serialize, Deserialize)]
    pub struct BrandingConfig {
        #[serde(default)]
        pub name: String,
        #[serde(default)]
        pub domain: String,
        #[serde(default)]
        pub display_name: String,
        #[serde(default)]
        pub title: String,
        #[serde(default)]
        pub description: String,
        #[serde(default)]
        pub tagline: String,
        #[serde(default)]
        pub copyright: String,
        #[serde(default, alias = "themeColor")]
        pub theme_color: String,
        #[serde(default)]
        pub platform_name: String,
        #[serde(default)]
        pub support_email: String,
        #[serde(default)]
        pub logo_light: String,
        #[serde(default)]
        pub logo_dark: String,
        #[serde(default)]
        pub favicon: String,
        #[serde(default)]
        pub twitter_handle: String,
    }
}

pub use branding::BrandingConfig;
