mod config;
mod provider;

pub use config::{
    BrandingConfig, ContactLink, DocsSidebarSection, FooterConfig, FooterLink, HeaderNavConfig,
    NavCta, NavItem, NavLink, NavSection, NavigationConfig, SocialLink,
};
pub use provider::NavigationPageDataProvider;
