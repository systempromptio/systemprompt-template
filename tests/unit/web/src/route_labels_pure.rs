#![allow(
    clippy::expect_used,
    clippy::panic,
    reason = "test code: panics are the assertion mechanism"
)]

//! The route and provider labels every console page names a route by.
//!
//! A declared name wins; without one the label is derived from the pattern
//! and the provider's display name, and the generated id never leaks into
//! a label on its own.

use systemprompt::models::ServicesConfig;
use systemprompt::models::services::{GatewayConfigSpec, GatewayState, ProviderEntry};
use systemprompt_web_admin::repositories::config::gateway::{
    derive_provider_label, derive_route_label, get_route_labels,
};

fn provider(name: &str, display: Option<&str>) -> ProviderEntry {
    let mut entry: ProviderEntry = serde_yaml::from_str(&format!(
        "name: {name}\nwire: anthropic\nsurface: anthropic\nendpoint: https://x.test/v1\napi_key_secret: {name}\n"
    ))
    .expect("provider parses");
    entry.display_name = display.map(str::to_owned);
    entry
}

#[test]
fn provider_label_prefers_display_name_then_known_spelling_then_title_case() {
    assert_eq!(
        derive_provider_label(
            "vertex",
            Some(&provider("vertex", Some("Google Vertex AI")))
        ),
        "Google Vertex AI"
    );
    assert_eq!(
        derive_provider_label("vertex", Some(&provider("vertex", Some("  ")))),
        "Vertex AI"
    );
    assert_eq!(
        derive_provider_label("openai-responses", None),
        "OpenAI (Responses)"
    );
    assert_eq!(derive_provider_label("my-lab.host", None), "My Lab Host");
}

#[test]
fn route_label_reads_the_pattern_as_a_model_family() {
    assert_eq!(
        derive_route_label("claude-*", "Anthropic"),
        "Claude models · Anthropic"
    );
    assert_eq!(
        derive_route_label("*", "Anthropic"),
        "Everything else · Anthropic"
    );
    assert_eq!(derive_route_label("o4-mini", "OpenAI"), "O4 Mini · OpenAI");
    assert_eq!(
        derive_route_label("openai.gpt-oss-*", "Vertex AI MaaS"),
        "Openai Gpt Oss models · Vertex AI MaaS"
    );
}

#[test]
fn declared_name_wins_and_catch_all_is_labelled() {
    let spec: GatewayConfigSpec = serde_yaml::from_str(
        r"
enabled: true
routes:
- name: Claude (Anthropic)
  description: Every claude-* request.
  model_pattern: claude-*
  provider: anthropic
default_provider: anthropic
default_model: claude-sonnet-5
",
    )
    .expect("gateway spec parses");
    let mut services = ServicesConfig::default();
    services
        .providers
        .providers
        .push(provider("anthropic", Some("Anthropic")));
    let mut config = spec.resolve();
    for route in &mut config.routes {
        route.ensure_id();
    }
    services.gateway = Some(GatewayState::Resolved(config));
    let labels = get_route_labels(&services).expect("labels resolve");
    let claude = labels
        .routes
        .iter()
        .find(|r| r.model_pattern == "claude-*")
        .expect("claude route");
    assert_eq!(claude.label, "Claude (Anthropic)");
    assert!(claude.declared_name);
    assert_eq!(claude.subtitle(), "claude-* → Anthropic");
    assert_eq!(
        claude.description.as_deref(),
        Some("Every claude-* request.")
    );
    let catch_all = labels
        .routes
        .iter()
        .find(|r| r.model_pattern == "*")
        .expect("synthesized catch-all");
    assert_eq!(catch_all.label, "Everything else · Anthropic");
    assert!(!catch_all.declared_name);
    assert_eq!(labels.label_of("no-such-route"), "no-such-route");
    assert_eq!(
        labels.find_provider("anthropic").map(|p| p.label.as_str()),
        Some("Anthropic")
    );
}
