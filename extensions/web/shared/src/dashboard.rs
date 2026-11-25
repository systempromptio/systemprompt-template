//! The console dashboard config, `services/web/config/dashboard.yaml`.
//!
//! Names the console's landing page, its home cards and its sidebar
//! sections, each by nav item id. A flavour overrides the file under
//! `flavours/<flavour>/services/web/config/dashboard.yaml`; the console
//! resolves the ids against the pages this build compiles and refuses boot on
//! one it does not.

use serde::Deserialize;

/// The parsed dashboard config; every field is required and unknown keys are
/// refused.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardConfig {
    pub landing: String,
    pub home: Vec<HomeCardConfig>,
    pub nav: Vec<NavSectionConfig>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HomeCardConfig {
    pub item: String,
    pub title: String,
    pub note: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NavSectionConfig {
    pub id: String,
    pub title: String,
    pub items: Vec<String>,
}
