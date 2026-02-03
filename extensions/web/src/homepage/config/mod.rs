mod sections;

pub use sections::{
    ComparisonConfig, ComparisonItem, ComparisonSide, DifferentiatorConfig, DifferentiatorItem,
    ExtensionTrait, ExtensionsConfig, FaqConfig, FaqItem, FinalCtaConfig, PricingConfig,
    PricingTier, TechnicalConfig, TechnicalStandard, UseCase, UseCasesConfig,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomepageConfig {
    pub hero: HeroConfig,
    #[serde(default)]
    pub value_props: Vec<ValueProp>,
    pub integrations: IntegrationsConfig,
    pub features: FeaturesConfig,
    pub how_it_works: HowItWorksConfig,
    #[serde(default)]
    pub playbooks: Option<PlaybooksConfig>,
    pub use_cases: UseCasesConfig,
    pub technical: TechnicalConfig,
    pub comparison: ComparisonConfig,
    pub pricing: PricingConfig,
    pub faq: FaqConfig,
    pub final_cta: FinalCtaConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueProp {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub title_highlight: Option<String>,
    pub subtitle: String,
    pub icon: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeroConfig {
    pub title: String,
    #[serde(default)]
    pub title_highlight: Option<String>,
    pub subtitle: String,
    pub cta: String,
    #[serde(default)]
    pub cta_url: Option<String>,
    pub cta_secondary: String,
    #[serde(default)]
    pub cta_secondary_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationsConfig {
    pub label: String,
    pub brands: Vec<IntegrationBrand>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationBrand {
    pub name: String,
    pub logo: String,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesConfig {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub subtitle: Option<String>,
    pub categories: Vec<FeatureCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureCategory {
    pub name: String,
    pub features: Vec<Feature>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub title: String,
    pub description: String,
    pub icon: String,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HowItWorksConfig {
    pub title: String,
    pub steps: Vec<HowItWorksStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HowItWorksStep {
    pub number: String,
    pub title: String,
    pub description: String,
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybooksConfig {
    pub title: String,
    #[serde(default)]
    pub title_highlight: Option<String>,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub flow: Option<PlaybooksFlowConfig>,
    #[serde(default)]
    pub featured: Option<PlaybooksFeaturedConfig>,
    #[serde(default)]
    pub categories: Vec<PlaybooksCategoryConfig>,
    #[serde(default)]
    pub differentiators: Vec<PlaybooksDifferentiatorConfig>,
    #[serde(default)]
    pub cta_primary: Option<String>,
    #[serde(default)]
    pub cta_primary_url: Option<String>,
    #[serde(default)]
    pub cta_secondary: Option<String>,
    #[serde(default)]
    pub cta_secondary_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybooksFlowConfig {
    pub steps: Vec<PlaybooksFlowStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybooksFlowStep {
    pub number: String,
    pub label: String,
    #[serde(default)]
    pub icon: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybooksFeaturedConfig {
    #[serde(default)]
    pub badge: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybooksCategoryConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub count: Option<i32>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybooksDifferentiatorConfig {
    #[serde(default)]
    pub icon: Option<String>,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
}
