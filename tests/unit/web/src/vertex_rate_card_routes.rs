//! The embedded Vertex rate card, cross-checked against this deployment's
//! gateway routes.
//!
//! Vertex publishes and retires MaaS models without an operator edit, so the
//! served catalog is not only what `services/ai/providers.yaml` names: at boot,
//! every model Vertex publishes that the core rate card prices is added to the
//! `vertex-maas` provider. A card id with no matching route in
//! `services/ai/gateway.yaml` is therefore priced but unreachable — the model
//! is installed into the registry and no request can ever select it, which is
//! invisible in the YAML because neither file mentions the other.
//!
//! This test makes that pairing explicit: the augmented registry must still
//! validate (so nothing the card prices can break the boot-time pricing gate),
//! and every card id that is not on its way out must match a route.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: panics are the assertion mechanism"
)]

use systemprompt::loader::config_loader::gateway::backfill_route_ids;
use systemprompt::models::services::{GatewayConfigSpec, ProviderRegistry, VertexRateCard};

use crate::support::repo_root;

fn yaml_section<T: serde::de::DeserializeOwned>(file: &str, key: &str) -> T {
    let path = repo_root().join(file);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    let root: serde_yaml::Value =
        serde_yaml::from_str(&text).unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()));
    let node = root
        .get(key)
        .unwrap_or_else(|| panic!("{} has no {key}: key", path.display()))
        .clone();
    serde_yaml::from_value(node)
        .unwrap_or_else(|e| panic!("parsing {key}: in {}: {e}", path.display()))
}

fn registry() -> ProviderRegistry {
    yaml_section("services/ai/providers.yaml", "providers")
}

// Why: three routes in gateway.yaml carry no `id:`; the loader synthesises
// one at boot, so the test runs the same backfill before validating.
fn gateway() -> GatewayConfigSpec {
    let mut spec: GatewayConfigSpec = yaml_section("services/ai/gateway.yaml", "gateway");
    backfill_route_ids(&mut spec);
    spec
}

// The registry as it looks after a boot-time discovery pass in which Vertex
// publishes everything the rate card prices — the widest catalog this
// deployment can ever serve.
fn card() -> VertexRateCard {
    VertexRateCard::embedded().expect("the embedded rate card parses")
}

// Why: discovery appends a card entry to the provider the card names, so the
// registry is augmented the same way, provider by provider. Entries with a
// documented retirement are left out: discovery withholds them once inside
// the notice window, their catalog declarations are gone, and a route that
// reached only them would itself be refused at boot as reaching no priced
// model — so they are the one class of card id that may have no route.
fn augmented_registry() -> ProviderRegistry {
    let mut registry = registry();
    let card = card();
    for provider in &mut registry.providers {
        for entry in card.entries_for(provider.name.as_str()) {
            if entry.retires_on.is_some() || provider.models.iter().any(|m| m.id == entry.id) {
                continue;
            }
            provider.models.push(entry.to_provider_model());
        }
    }
    registry
}

#[test]
fn the_full_rate_card_still_validates_against_the_gateway() {
    let registry = augmented_registry();
    gateway()
        .resolve()
        .validate(&registry)
        .expect("every rate-card model must be priced and routable");
}

#[test]
fn every_rate_card_id_matches_a_gateway_route() {
    let config = gateway().resolve();
    let configured_registry = registry();
    let configured_ids: std::collections::HashSet<&str> = configured_registry
        .providers
        .iter()
        .flat_map(|provider| provider.models.iter().map(|model| model.id.as_str()))
        .collect();
    let unroutable: Vec<String> = card()
        .entries
        .iter()
        .filter(|entry| entry.retires_on.is_none() && configured_ids.contains(entry.id.as_str()))
        .map(|entry| entry.id.as_str().to_owned())
        .filter(|id| config.find_route(id).is_none())
        .collect();
    assert!(
        unroutable.is_empty(),
        "rate-card models with no route in services/ai/gateway.yaml (they would be priced but \
         unreachable): {}",
        unroutable.join(", ")
    );
}
