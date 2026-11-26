//! The console's dashboard: the landing page, the home cards and the sidebar,
//! resolved once at boot from `services/web/config/dashboard.yaml` against
//! the [`registry`] of nav items this build compiles.
//!
//! The effective nav is the configured nav intersected with the compiled
//! pages, and the intersection is never silent: a configured item whose page
//! this build leaves out refuses boot, naming the console feature (and so the
//! flavour) that would compile it.

mod registry;

use std::path::Path;

use serde::Serialize;
use systemprompt_web_shared::config_loader::{ConfigError, dashboard_config};
use systemprompt_web_shared::dashboard::DashboardConfig;
use thiserror::Error;

use registry::NavItemDef;

#[derive(Debug, Error)]
pub enum DashboardError {
    #[error("dashboard config could not be loaded")]
    Load(#[from] ConfigError),
    #[error("dashboard {place} names nav item `{item}`, which no console page has")]
    UnknownItem { place: &'static str, item: String },
    #[error(
        "dashboard {place} names nav item `{item}`, which this build does not compile: it needs the `{feature}` console feature, which a flavour that includes it enables"
    )]
    NotCompiled {
        place: &'static str,
        item: String,
        feature: &'static str,
    },
    #[error("dashboard nav lists item `{item}` more than once")]
    Duplicate { item: String },
    #[error("dashboard nav section `{section}` has no items")]
    EmptySection { section: String },
    #[error("dashboard nav section id `{section}` is used more than once")]
    DuplicateSection { section: String },
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct NavItemView {
    href: &'static str,
    text: &'static str,
    leaf: &'static str,
    child: &'static str,
    child2: &'static str,
    scope_nav: bool,
    external: bool,
    attention: bool,
    icon: &'static str,
    #[serde(skip)]
    console: bool,
}

impl NavItemView {
    #[must_use]
    pub const fn console(&self) -> bool {
        self.console
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NavSectionView {
    id: String,
    title: String,
    items: Vec<NavItemView>,
}

impl NavSectionView {
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    #[must_use]
    pub fn items(&self) -> &[NavItemView] {
        &self.items
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HomeCardView {
    href: &'static str,
    label: String,
    note: String,
}

/// The resolved dashboard every console page renders its sidebar from.
#[derive(Debug, Clone, Default)]
pub struct ConsoleDashboard {
    landing: &'static str,
    home: Vec<HomeCardView>,
    nav: Vec<NavSectionView>,
}

impl ConsoleDashboard {
    pub fn load(services: &Path) -> Result<Self, DashboardError> {
        Self::resolve(&dashboard_config(services)?)
    }

    pub fn resolve(config: &DashboardConfig) -> Result<Self, DashboardError> {
        let landing = compiled("landing", &config.landing)?.href;
        let home = config
            .home
            .iter()
            .map(|card| {
                Ok(HomeCardView {
                    href: compiled("home card", &card.item)?.href,
                    label: card.title.clone(),
                    note: card.note.clone(),
                })
            })
            .collect::<Result<Vec<_>, DashboardError>>()?;
        let mut seen_items: Vec<&str> = Vec::new();
        let mut seen_sections: Vec<&str> = Vec::new();
        let mut nav = Vec::with_capacity(config.nav.len());
        for section in &config.nav {
            if seen_sections.contains(&section.id.as_str()) {
                return Err(DashboardError::DuplicateSection {
                    section: section.id.clone(),
                });
            }
            seen_sections.push(&section.id);
            if section.items.is_empty() {
                return Err(DashboardError::EmptySection {
                    section: section.id.clone(),
                });
            }
            let mut items = Vec::with_capacity(section.items.len());
            for id in &section.items {
                if seen_items.contains(&id.as_str()) {
                    return Err(DashboardError::Duplicate { item: id.clone() });
                }
                seen_items.push(id);
                items.push(view(compiled("nav", id)?));
            }
            nav.push(NavSectionView {
                id: section.id.clone(),
                title: section.title.clone(),
                items,
            });
        }
        Ok(Self { landing, home, nav })
    }

    #[must_use]
    pub const fn landing(&self) -> &'static str {
        self.landing
    }

    #[must_use]
    pub fn home(&self) -> &[HomeCardView] {
        &self.home
    }

    #[must_use]
    pub fn nav(&self) -> &[NavSectionView] {
        &self.nav
    }
}

fn compiled(place: &'static str, id: &str) -> Result<&'static NavItemDef, DashboardError> {
    let item = registry::find(id).ok_or_else(|| DashboardError::UnknownItem {
        place,
        item: id.to_owned(),
    })?;
    match item.needs {
        Some(feature) if !item.compiled => Err(DashboardError::NotCompiled {
            place,
            item: id.to_owned(),
            feature,
        }),
        _ => Ok(item),
    }
}

const fn view(item: &'static NavItemDef) -> NavItemView {
    NavItemView {
        href: item.href,
        text: item.text,
        leaf: item.leaf,
        child: item.child,
        child2: item.child2,
        scope_nav: item.scope_nav,
        external: item.external,
        attention: item.attention,
        icon: item.icon,
        console: item.console,
    }
}
