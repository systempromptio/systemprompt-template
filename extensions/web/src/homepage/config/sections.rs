use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseCasesConfig {
    pub title: String,
    pub cases: Vec<UseCase>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UseCase {
    pub title: String,
    pub description: String,
    pub icon: String,
    pub code: String,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalConfig {
    pub title: String,
    pub subtitle: String,
    #[serde(default)]
    pub tagline: Option<String>,
    #[serde(default)]
    pub standards: Vec<TechnicalStandard>,
    #[serde(default)]
    pub extensions: Option<ExtensionsConfig>,
    #[serde(default)]
    pub differentiator: Option<DifferentiatorConfig>,
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
pub struct TechnicalStandard {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub logo: String,
    pub tagline: String,
    pub description: String,
    #[serde(default)]
    pub benefits: Vec<String>,
    #[serde(default)]
    pub link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionsConfig {
    pub title: String,
    pub subtitle: String,
    #[serde(default)]
    pub ownership_message: Option<String>,
    #[serde(default)]
    pub traits: Vec<ExtensionTrait>,
    #[serde(default)]
    pub architecture_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionTrait {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferentiatorConfig {
    pub title: String,
    #[serde(default)]
    pub items: Vec<DifferentiatorItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferentiatorItem {
    pub icon: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonConfig {
    pub title: String,
    pub superagent: ComparisonSide,
    pub harness: ComparisonSide,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonSide {
    pub title: String,
    pub items: Vec<ComparisonItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonItem {
    pub text: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingConfig {
    pub title: String,
    #[serde(default)]
    pub subtitle: Option<String>,
    #[serde(default)]
    pub docs_url: Option<String>,
    pub tiers: Vec<PricingTier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingTier {
    pub name: String,
    pub price: String,
    pub description: String,
    pub features: Vec<String>,
    pub cta: String,
    pub cta_url: String,
    #[serde(default)]
    pub highlight: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaqConfig {
    pub title: String,
    pub items: Vec<FaqItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaqItem {
    pub question: String,
    pub answer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinalCtaConfig {
    pub title: String,
    pub subtitle: String,
    pub button: String,
}
