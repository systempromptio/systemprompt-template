mod config;
pub mod prerenderer;
pub mod provider;

pub use config::{
    ComparisonConfig, ComparisonItem, ComparisonSide, DifferentiatorConfig, DifferentiatorItem,
    ExtensionTrait, ExtensionsConfig, FaqConfig, FaqItem, Feature, FeatureCategory,
    FeaturesConfig as HomepageFeaturesConfig, FinalCtaConfig, HeroConfig, HomepageConfig,
    HowItWorksConfig, HowItWorksStep, IntegrationBrand, IntegrationsConfig, PricingConfig,
    PricingTier, TechnicalConfig, TechnicalStandard, UseCase, UseCasesConfig, ValueProp,
};
pub use prerenderer::HomepagePrerenderer;
pub use provider::HomepagePageDataProvider;
