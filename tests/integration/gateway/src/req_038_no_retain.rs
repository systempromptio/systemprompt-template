//! REQ-038 Provider No-Train / No-Retain Policy Enforcement — "Policy can
//! prevent classified requests from being routed to providers that do not
//! satisfy the configured no-train/no-retain contractual requirement."
//!
//! Mirrors the residency evidence for the `no_retain` flag, and additionally
//! proves the model-level override: a model's own `governance:` block beats
//! the provider default in `ProviderEntry::effective_governance`, which is
//! what `validate()` and dispatch both consult.

use systemprompt::models::profile::{
    GatewayProfileError, GatewayRoute, RouteRequirements, WireProtocol,
};
use systemprompt::models::services::ModelGovernance;

use crate::support::{config, model, provider, registry, route};

fn no_retain_route() -> GatewayRoute {
    let mut r = route("no-retain", "classified-*", "retaining-provider");
    r.requires = Some(RouteRequirements {
        european: false,
        no_retain: true,
    });
    r
}

#[test]
fn a_no_retain_route_over_a_retaining_provider_fails_validation() {
    let reg = registry(vec![provider(
        "retaining-provider",
        WireProtocol::Anthropic,
        ModelGovernance::default(),
        vec![model("classified-fast")],
    )]);
    let cfg = config(vec![no_retain_route()]);

    match cfg.validate(&reg) {
        Err(GatewayProfileError::RouteGovernanceUnsatisfied {
            route,
            model,
            requirements,
        }) => {
            assert_eq!(route, "no-retain");
            assert_eq!(model, "classified-fast");
            assert_eq!(requirements, "no_retain");
        },
        other => panic!("expected RouteGovernanceUnsatisfied, got {other:?}"),
    }
}

#[test]
fn the_same_route_validates_once_the_provider_contract_declares_no_retain() {
    let reg = registry(vec![provider(
        "retaining-provider",
        WireProtocol::Anthropic,
        ModelGovernance {
            european: false,
            no_retain: true,
        },
        vec![model("classified-fast")],
    )]);
    let cfg = config(vec![no_retain_route()]);

    assert!(
        cfg.validate(&reg).is_ok(),
        "a no-retain provider satisfies the no-retain route requirement"
    );
}

#[test]
fn a_model_level_governance_block_overrides_the_provider_default() {
    let mut covered = model("covered-model");
    covered.governance = Some(ModelGovernance {
        european: false,
        no_retain: true,
    });
    let entry = provider(
        "mixed-provider",
        WireProtocol::OpenAiChat,
        ModelGovernance::default(),
        vec![covered, model("uncovered-model")],
    );

    assert!(
        entry.effective_governance("covered-model").no_retain,
        "the model's own declaration wins over the provider default"
    );
    assert!(
        !entry.effective_governance("uncovered-model").no_retain,
        "a model without its own block inherits the provider default"
    );
    assert!(
        !entry.effective_governance("never-listed").no_retain,
        "an unlisted model inherits the provider default too"
    );
}

#[test]
fn validation_honours_the_model_level_override() {
    let mut covered = model("classified-covered");
    covered.governance = Some(ModelGovernance {
        european: false,
        no_retain: true,
    });
    let reg = registry(vec![provider(
        "retaining-provider",
        WireProtocol::OpenAiChat,
        ModelGovernance::default(),
        vec![covered],
    )]);
    let cfg = config(vec![no_retain_route()]);

    assert!(
        cfg.validate(&reg).is_ok(),
        "the only reachable model declares no_retain itself, so the route validates \
         even though the provider default does not"
    );
}
