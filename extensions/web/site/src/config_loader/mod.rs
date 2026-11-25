//! Loading of the public-site sections of the `services/web/` YAML tree.
//!
//! Lives beside the config types it deserialises, so adding a YAML-backed
//! page section is one edit in this crate. Every load takes the services
//! directory it reads from; nothing is cached process-wide. A load failure is
//! logged and served as "section absent", never a crash. The web member loads
//! each section once, in its provide fns and factories, after admission.

use std::path::Path;
use std::sync::Arc;

use systemprompt_web_shared::config_loader::{
    ConfigError, load_config_section, log_and_discard_err, parse_section,
};

#[doc(hidden)]
pub mod skills;

pub(crate) use skills::load_skills_page_config;

use crate::homepage::HomepageConfig;
use crate::navigation::NavigationConfig;
use crate::skills_page::SkillsPageConfig;

fn load_navigation_config(services: &Path) -> Result<Option<Arc<NavigationConfig>>, ConfigError> {
    let Some(nav_value) = load_config_section(services, "navigation.yaml")? else {
        return Ok(None);
    };

    let nav_config: NavigationConfig = parse_section(nav_value, "navigation.yaml")?;

    tracing::info!("Loaded navigation config from config/navigation.yaml");

    Ok(Some(Arc::new(nav_config)))
}

fn load_homepage_config(services: &Path) -> Result<Option<Arc<HomepageConfig>>, ConfigError> {
    let Some(homepage_value) = load_config_section(services, "homepage.yaml")? else {
        return Ok(None);
    };

    let homepage_config: HomepageConfig = parse_section(homepage_value, "homepage.yaml")?;

    tracing::info!("Loaded homepage config from config/homepage.yaml");

    Ok(Some(Arc::new(homepage_config)))
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
