//! REQ-038 Provider No-Train / No-Retain Policy Enforcement — "Policy can
//! prevent classified requests from being routed to providers that do not
//! satisfy the configured no-train/no-retain contractual requirement."
//!
//! Mirrors the residency evidence for the `no_retain` flag, and additionally
//! proves the model-level override: a model's own `governance:` block beats
//! the provider default on every resolved deployment hop, which is what
//! catalog validation and dispatch both consult.

use systemprompt::manifest::services::catalog::CatalogError;

use crate::support::{Provider, catalog, provider};

const ROUTE: &str = "gateway:\n  enabled: false\n  routes:\n    - id: no-retain\n      models: \"classified-*\"\n      requires: { no_retain: true }\n";

fn body(models: &str, deployments: &str) -> String {
    format!("models:\n{models}deployments:\n{deployments}{ROUTE}")
}

#[test]
fn a_no_retain_route_over_a_retaining_provider_fails_validation() {
    let yaml = body(
        "  classified-fast: { clients: [] }\n",
        "  classified-fast: [retaining-provider/api]\n",
    );
    match catalog(&[provider("retaining-provider", "anthropic")], &yaml) {
        Err(CatalogError::RouteGovernanceUnsatisfied {
            route,
            model,
            requirements,
            ..
        }) => {
            assert_eq!(route.as_str(), "no-retain");
            assert_eq!(model.as_str(), "classified-fast");
            assert_eq!(requirements, "no_retain");
        },
        other => panic!("expected RouteGovernanceUnsatisfied, got {other:?}"),
    }
}

#[test]
fn the_same_route_validates_once_the_provider_contract_declares_no_retain() {
    let yaml = body(
        "  classified-fast: { clients: [] }\n",
        "  classified-fast: [retaining-provider/api]\n",
    );
    assert!(
        catalog(
            &[Provider {
                european: false,
                no_retain: true,
                ..provider("retaining-provider", "anthropic")
            }],
            &yaml
        )
        .is_ok(),
        "a no-retain provider satisfies the no-retain route requirement"
    );
}

#[test]
fn a_model_level_governance_block_overrides_the_provider_default() {
    let cat = catalog(
        &[provider("mixed-provider", "openai-chat")],
        "models:\n  covered-model: { clients: [], governance: { no_retain: true } }\n  uncovered-model: { clients: [] }\ndeployments:\n  covered-model: [mixed-provider/api]\n  uncovered-model: [mixed-provider/api]\n",
    )
    .expect("the catalog resolves");

    assert!(
        cat.deployments("covered-model")[0].governance.no_retain,
        "the model's own declaration wins over the provider default"
    );
    assert!(
        !cat.deployments("uncovered-model")[0].governance.no_retain,
        "a model without its own block inherits the provider default"
    );
}

#[test]
fn validation_honours_the_model_level_override() {
    let yaml = body(
        "  classified-covered: { clients: [], governance: { no_retain: true } }\n",
        "  classified-covered: [retaining-provider/api]\n",
    );
    assert!(
        catalog(&[provider("retaining-provider", "openai-chat")], &yaml).is_ok(),
        "the only reachable model declares no_retain itself, so the route validates \
         even though the provider default does not"
    );
}
