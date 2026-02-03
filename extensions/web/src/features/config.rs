use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    pub pages: Vec<FeaturePage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturePage {
    pub slug: String,
    pub title: String,
    pub headline: String,
    #[serde(default)]
    pub headline_highlight: Option<String>,
    pub subtitle: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub highlights: Vec<Highlight>,
    #[serde(default)]
    pub cta: Option<FeatureCta>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub sections: Vec<FeatureSection>,
    #[serde(default)]
    pub related: Vec<RelatedFeature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Highlight {
    pub text: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedFeature {
    pub title: String,
    pub description: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureCta {
    pub text: String,
    pub url: String,
    #[serde(default)]
    pub secondary_text: Option<String>,
    #[serde(default)]
    pub secondary_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSection {
    #[serde(default)]
    pub id: Option<String>,
    pub title: String,
    pub content: String,
    pub visual_desktop: String,
    pub visual_mobile: String,
    #[serde(default)]
    pub visual_position: Option<String>,
    #[serde(default)]
    pub items: Vec<FeatureSectionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSectionItem {
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub icon: Option<String>,
}
