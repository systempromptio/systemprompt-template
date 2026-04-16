mod config;
pub mod demo_scanner;
pub mod prerenderer;
pub mod provider;

pub use config::{
    ComparisonConfig, ComparisonItem, ComparisonSide, DemoCategory, DemoPillar, DemoStep,
    DemosConfig, DifferentiatorConfig, DifferentiatorItem, ExtensionTrait, ExtensionsConfig,
    FaqConfig, FaqItem, Feature, FeatureCategory, FeaturesConfig as HomepageFeaturesConfig,
    FinalCtaConfig, HeroConfig, HomepageConfig, HowItWorksConfig, HowItWorksStep, IntegrationBrand,
    IntegrationsConfig, PricingConfig, PricingTier, QuickStartStep, TechnicalConfig,
    TechnicalStandard, UseCase, UseCasesConfig, ValueProp,
};
pub use prerenderer::HomepagePrerenderer;
pub use provider::HomepagePageDataProvider;
