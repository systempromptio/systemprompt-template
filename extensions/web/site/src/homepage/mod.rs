//! Homepage section: config model, demo scanner, prerenderer, and data
//! provider.

mod config;
mod context;
pub mod demo_scanner;
pub mod prerenderer;
pub mod provider;

pub use config::{
    ComparisonConfig, ComparisonItem, ComparisonSide, DemoCategory, DemoPillar, DemoStep,
    DemosConfig, DifferentiatorConfig, DifferentiatorItem, ExtensionTrait, ExtensionsConfig,
    FaqConfig, FaqItem, Feature, FeatureCategory, FinalCtaConfig, HeroConfig, HomepageConfig,
    HomepageFeaturesSection as HomepageFeaturesConfig, HowItWorksConfig, HowItWorksStep,
    IntegrationBrand, IntegrationsConfig, PricingConfig, PricingTier, QuickStartStep,
    TechnicalConfig, TechnicalStandard, UseCase, UseCasesConfig, ValueProp,
};
pub use prerenderer::HomepagePrerenderer;
pub use provider::HomepagePageDataProvider;
