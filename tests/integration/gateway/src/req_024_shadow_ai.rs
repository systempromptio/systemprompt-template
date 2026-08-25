//! REQ-024 Shadow AI Detection & Blocking — "Unauthorized or unmanaged AI use
//! can be detected and, where policy requires, blocked or escalated through
//! the governed platform."
//!
//! The gateway's exposure decision (`GatewayConfig::is_model_exposed`) is the
//! blocking half: with `allow_unlisted_models: false` a model that is neither
//! routed nor in any provider catalog is refused, so traffic to an
//! un-governed model cannot pass through the platform. The tests also
//! document the opt-in open posture — `allow_unlisted_models: true` plus a
//! `default_provider` — so the closed default is visibly a choice, not an
//! accident.

use systemprompt::identifiers::ProviderId;
use systemprompt::models::profile::WireProtocol;
use systemprompt::models::services::ModelGovernance;

use crate::support::{config, model, provider, registry, route};

fn one_provider_registry() -> systemprompt::models::profile::ProviderRegistry {
    registry(vec![provider(
        "governed-provider",
        WireProtocol::Anthropic,
        ModelGovernance::default(),
        vec![model("governed-model")],
    )])
}

#[test]
fn an_unlisted_model_is_not_exposed_under_the_closed_posture() {
    let reg = one_provider_registry();
    let cfg = config(vec![route("governed", "governed-*", "governed-provider")]);

    assert!(
        !cfg.is_model_exposed(&reg, "shadow-model"),
        "a model with no route and no catalog entry is refused"
    );
}

#[test]
fn routed_and_cataloged_models_are_exposed() {
    let reg = one_provider_registry();
    let cfg = config(vec![route("governed", "governed-*", "governed-provider")]);

    assert!(
        cfg.is_model_exposed(&reg, "governed-anything"),
        "a route pattern match exposes the model"
    );
    assert!(
        cfg.is_model_exposed(&reg, "governed-model"),
        "a provider catalog entry exposes the model even without its own route"
    );
}

#[test]
fn allow_unlisted_models_with_a_default_provider_opens_the_allowlist() {
    let reg = one_provider_registry();
    let mut cfg = config(vec![]);
    cfg.allow_unlisted_models = true;
    cfg.default_provider = Some(ProviderId::new("governed-provider"));

    assert!(
        cfg.is_model_exposed(&reg, "shadow-model"),
        "the open posture forwards unlisted models to the default provider"
    );
}

#[test]
fn allow_unlisted_models_without_a_default_provider_still_refuses() {
    let reg = one_provider_registry();
    let mut cfg = config(vec![]);
    cfg.allow_unlisted_models = true;

    assert!(
        !cfg.is_model_exposed(&reg, "shadow-model"),
        "the flag alone opens nothing — there is nowhere to send the traffic"
    );
}
