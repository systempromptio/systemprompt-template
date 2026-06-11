//! Shared types for the web extension siblings.
//!
//! No business logic — only the cross-cutting primitives the other web
//! crates need to agree on:
//!
//! - [`config`] / [`config_errors`] — typed YAML schemas with
//!   `#[serde(deny_unknown_fields)]` plus the error type that surfaces
//!   misconfigurations at startup.
//! - [`error::MarketplaceError`] — the unified error sink for admin/jobs
//!   code; implements `IntoResponse` for axum handlers.
//! - [`ids`] — newtype wrappers around `String` for entity IDs that escape
//!   the web layer (`MarketplaceId`, `PluginId`, `TraceId`, etc.). For
//!   identifiers that originate in `systemprompt_identifiers`, prefer those
//!   directly.
//! - [`models`] — content/link/search wire types shared between the
//!   content extension and the admin dashboard.
//! - [`BrandingConfig`] — branding fields parsed from `services/web/config.yaml`.
//! - [`html_escape`] — escape helper re-exported as `utils::html_escape`.

pub mod config;
pub mod config_errors;
pub mod error;
pub mod ids;
pub mod models;
mod utils;

pub use ids::{MarketplaceId, PluginId, RankTier, RequestId, TierLevel, TraceId, UserId};

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
        #[serde(default)]
        pub image: String,
    }
}

pub use branding::BrandingConfig;
