//! The shipped Anthropic catalog, pinned. `services/ai/providers.yaml` is the
//! deployment's allow-list: `allow_unlisted_models` is `false` in
//! `services/ai/gateway.yaml`, so a model id that is absent from this file
//! cannot be called by anyone, and an id that is present is billed at the
//! pricing written beside it.
//!
//! That makes catalog drift expensive in two directions. A missing current
//! model is an outage for every client that names it. A stale price is a
//! silently wrong cost figure on every audit row, every budget window, and
//! every finance report derived from them — nothing downstream re-checks the
//! vendor's rate card.
//!
//! These tests pin the Anthropic id set, the Sonnet 5 rate that was wrong here
//! until 2026-09-04, and the fact that the gateway's default model resolves
//! inside the catalog. Changing any of them is then a deliberate,
//! review-visible edit rather than a one-line drift.

#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: panics are the assertion mechanism"
)]

use systemprompt::models::services::{GatewayConfigSpec, ProviderModel, ProviderRegistry};

use crate::support::repo_root;

// Rate-card figures are written as decimal literals in the YAML and parsed to
// the same `f64`, so exact comparison is the right check here; the helper keeps
// clippy's blanket float-comparison lint from firing on every assertion.
fn same(actual: f64, expected: f64) -> bool {
    (actual - expected).abs() < f64::EPSILON
}

fn registry() -> ProviderRegistry {
    let path = repo_root().join("services/ai/providers.yaml");
    let yaml = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    let root: serde_yaml::Value =
        serde_yaml::from_str(&yaml).unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()));
    let node = root
        .get("providers")
        .unwrap_or_else(|| panic!("{} has no providers: key", path.display()))
        .clone();
    serde_yaml::from_value(node)
        .unwrap_or_else(|e| panic!("parsing providers: in {}: {e}", path.display()))
}

fn gateway() -> GatewayConfigSpec {
    let path = repo_root().join("services/ai/gateway.yaml");
    let yaml = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    let root: serde_yaml::Value =
        serde_yaml::from_str(&yaml).unwrap_or_else(|e| panic!("parsing {}: {e}", path.display()));
    let node = root
        .get("gateway")
        .unwrap_or_else(|| panic!("{} has no gateway: key", path.display()))
        .clone();
    serde_yaml::from_value(node)
        .unwrap_or_else(|e| panic!("parsing gateway: in {}: {e}", path.display()))
}

fn anthropic_models() -> Vec<ProviderModel> {
    registry()
        .providers
        .into_iter()
        .find(|p| p.name.as_str() == "anthropic")
        .expect("the anthropic provider entry")
        .models
}

#[test]
fn the_anthropic_lineup_is_exactly_the_current_models() {
    let ids: Vec<String> = anthropic_models()
        .iter()
        .map(|m| m.id.as_str().to_owned())
        .collect();
    assert_eq!(
        ids,
        vec![
            "claude-opus-5",
            "claude-sonnet-5",
            "claude-fable-5-1",
            "claude-fable-5",
            "claude-opus-4-8",
            "claude-opus-4-7",
            "claude-opus-4-6",
            "claude-sonnet-4-6",
            "claude-haiku-4-5",
            "claude-opus-4-5",
            "claude-sonnet-4-5",
        ]
    );
}

#[test]
fn the_retired_dated_ids_are_gone() {
    let models = anthropic_models();
    for retired in [
        "claude-opus-4-1-20250805",
        "claude-sonnet-4-20250514",
        "claude-opus-4-20250514",
    ] {
        assert!(
            !models.iter().any(|m| m.matches(retired)),
            "{retired} is retired at the vendor — serving it prices traffic \
             against a rate card that no longer exists"
        );
    }
}

#[test]
fn the_dated_aliases_that_clients_still_pin_keep_resolving() {
    let models = anthropic_models();
    for alias in [
        "claude-haiku-4-5-20251001",
        "claude-opus-4-5-20251101",
        "claude-sonnet-4-5-20250929",
    ] {
        assert!(
            models.iter().any(|m| m.matches(alias)),
            "{alias} is pinned by shipped clients and must stay an alias"
        );
    }
}

#[test]
fn sonnet_5_carries_the_current_rate_card() {
    let models = anthropic_models();
    let sonnet = models
        .iter()
        .find(|m| m.id.as_str() == "claude-sonnet-5")
        .expect("claude-sonnet-5");
    assert!(same(sonnet.pricing.input_per_million, 2.0));
    assert!(same(sonnet.pricing.output_per_million, 10.0));
    let cache_read = sonnet
        .pricing
        .cache_read_per_million
        .expect("cache read rate");
    let cache_write = sonnet
        .pricing
        .cache_write_per_million
        .expect("cache write rate");
    assert!(same(cache_read, 0.2));
    assert!(same(cache_write, 2.5));
}

#[test]
fn fable_5_1_is_priced_and_sized_as_the_frontier_model() {
    let models = anthropic_models();
    let fable = models
        .iter()
        .find(|m| m.id.as_str() == "claude-fable-5-1")
        .expect("claude-fable-5-1");
    assert!(same(fable.pricing.input_per_million, 10.0));
    assert!(same(fable.pricing.output_per_million, 50.0));
    assert_eq!(fable.limits.context_window, 1_000_000);
    assert_eq!(fable.limits.max_output_tokens, 128_000);
}

#[test]
fn the_gateway_default_model_is_a_catalog_entry() {
    let spec = gateway();
    let default_model = spec.default_model.expect("gateway.default_model");
    assert_eq!(default_model, "claude-sonnet-5");
    assert!(
        anthropic_models().iter().any(|m| m.matches(&default_model)),
        "the default model must be listed: allow_unlisted_models is false, so \
         an unlisted default is a 403 for every request that omits a model"
    );
}
