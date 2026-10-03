//! Shared builders for the gateway evidence modules: a provider entry with a
//! chosen governance posture and a gateway config wrapped around a route list.
//!
//! `GatewayConfig` is deliberately not `Deserialize` — direct struct-literal
//! construction is its sanctioned test path (see the type's docs) — so these
//! builders are the whole fixture layer.

use std::collections::HashMap;

use systemprompt::identifiers::{ModelId, ProviderId, RouteId, SecretName};
use systemprompt::manifest::services::{
    GatewayConfig, GatewayRoute, ModelCapabilities, ModelGovernance, ModelPricing, ProviderEntry,
    ProviderModel, ProviderRegistry, QuotaFaultMode,
};
use systemprompt::models::providers::ApiSurface;
use systemprompt::wire::{ModelLimits, WireProtocol};

pub(crate) fn model(id: &str) -> ProviderModel {
    ProviderModel {
        id: ModelId::new(id),
        aliases: Vec::new(),
        upstream_model: None,
        pricing: ModelPricing::default(),
        capabilities: ModelCapabilities::default(),
        limits: ModelLimits::default(),
        governance: None,
        hidden: false,
    }
}

pub(crate) fn provider(
    name: &str,
    wire: WireProtocol,
    governance: ModelGovernance,
    models: Vec<ProviderModel>,
) -> ProviderEntry {
    ProviderEntry {
        name: ProviderId::new(name),
        display_name: None,
        description: None,
        wire,
        surface: ApiSurface::Backend,
        endpoint: "https://api.example.com/v1".to_owned(),
        api_key_secret: SecretName::new("test_api_key"),
        extra_headers: HashMap::new(),
        accepted_betas: None,
        models,
        governance,
    }
}

pub(crate) fn registry(providers: Vec<ProviderEntry>) -> ProviderRegistry {
    ProviderRegistry { providers }
}

pub(crate) fn route(id: &str, pattern: &str, provider_name: &str) -> GatewayRoute {
    GatewayRoute {
        id: Some(RouteId::new(id)),
        name: None,
        description: None,
        model_pattern: pattern.to_owned(),
        provider: ProviderId::new(provider_name),
        upstream_model: None,
        extra_headers: HashMap::new(),
        pricing: None,
        when: None,
        requires: None,
        fallback_provider: None,
        fallback_upstream_model: None,
    }
}

// `enabled: false` keeps `validate()` off the route-pricing tier, which is
// billing evidence, not the residency/exposure evidence these modules are
// about; the governance tier runs regardless of `enabled`.
pub(crate) fn config(routes: Vec<GatewayRoute>) -> GatewayConfig {
    GatewayConfig {
        enabled: false,
        routes,
        default_provider: None,
        default_model: None,
        allow_unlisted_models: false,
        auth_scheme: "bearer".to_owned(),
        inference_path_prefix: "/v1".to_owned(),
        system_prompt_overrides: Vec::new(),
        bridge_releases: None,
        quota_fault_mode: QuotaFaultMode::default(),
        automatic_prompt_caching: true,
    }
}
