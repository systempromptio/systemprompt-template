//! Homepage section: config model, prerenderer, and data provider.

mod config;
mod context;
pub mod prerenderer;
pub mod provider;

pub use config::{
    ComparisonConfig, ComparisonItem, ComparisonSide, DifferentiatorConfig, DifferentiatorItem,
    ExtensionTrait, ExtensionsConfig, FaqConfig, FaqItem, Feature, FeatureCategory, FinalCtaConfig,
    HeroConfig, HomepageConfig, HomepageFeaturesSection as HomepageFeaturesConfig,
    HowItWorksConfig, HowItWorksStep, IntegrationBrand, IntegrationsConfig, PricingConfig,
    PricingTier, TechnicalConfig, TechnicalStandard, UseCase, UseCasesConfig, ValueProp,
};
pub use prerenderer::HomepagePrerenderer;
pub use provider::HomepagePageDataProvider;
