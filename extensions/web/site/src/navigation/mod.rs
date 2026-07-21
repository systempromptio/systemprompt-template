//! Site navigation: config model and the provider that supplies it to every
//! page.

mod config;
mod provider;

pub use config::{
    BrandingConfig, ContactLink, DocsSidebarSection, FooterConfig, FooterLink, HeaderNavConfig,
    NavCta, NavItem, NavLink, NavSection, NavigationConfig, SocialLink,
};
pub use provider::NavigationPageDataProvider;
