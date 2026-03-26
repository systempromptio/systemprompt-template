use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BrandingConfig {
    #[serde(default)]
    pub tagline: String,
    #[serde(default)]
    pub copyright: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationConfig {
    pub header: HeaderNavConfig,
    #[serde(default)]
    pub footer: FooterConfig,
    #[serde(default)]
    pub social: Vec<SocialLink>,
    #[serde(default)]
    pub docs_sidebar: Vec<DocsSidebarSection>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FooterConfig {
    #[serde(default)]
    pub legal: Vec<FooterLink>,
    #[serde(default)]
    pub resources: Vec<FooterLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FooterLink {
    pub path: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocialLink {
    pub href: String,
    #[serde(rename = "type")]
    pub link_type: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsSidebarSection {
    pub title: String,
    pub links: Vec<NavLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderNavConfig {
    pub items: Vec<NavItem>,
    #[serde(default)]
    pub cta: Option<NavCta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavItem {
    pub id: String,
    pub label: String,
    pub href: String,
    #[serde(default)]
    pub dropdown: bool,
    #[serde(default)]
    pub external: bool,
    #[serde(default)]
    pub sections: Vec<NavSection>,
    #[serde(default)]
    pub view_all: Option<NavLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavSection {
    #[serde(default)]
    pub title: Option<String>,
    pub links: Vec<NavLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavLink {
    pub label: String,
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavCta {
    pub label: String,
    pub href: String,
    #[serde(default)]
    pub external: bool,
}
