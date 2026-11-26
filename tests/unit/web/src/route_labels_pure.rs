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

use systemprompt::manifest::services::catalog::{CatalogContext, CatalogDocument};
use systemprompt_web_admin::repositories::config::gateway::{
    derive_provider_label, derive_route_label, get_route_labels,
};

#[test]
fn provider_label_prefers_display_name_then_known_spelling_then_title_case() {
    assert_eq!(
        derive_provider_label("vertex", Some("Google Vertex AI")),
        "Google Vertex AI"
    );
    assert_eq!(derive_provider_label("vertex", Some("  ")), "Vertex AI");
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
    let catalog = CatalogDocument::from_yaml_str(
        "route-labels",
        r#"
credentials:
  anthropic: { kind: platform, secret: anthropic_key }
providers:
  anthropic:
    credential: anthropic
    display_name: Anthropic
    endpoints:
      api: { wire: anthropic, url: "https://anthropic.example.com/v1" }
models:
  claude-sonnet-5: { clients: [] }
deployments:
  claude-sonnet-5: [anthropic/api]
gateway:
  enabled: false
  routes:
    - id: claude
      label: Claude (Anthropic)
      description: Every claude-* request.
      models: "claude-*"
      deployments: [anthropic/api]
    - id: everything-else
      models: "*"
      deployments: [anthropic/api]
"#,
    )
    .expect("the catalog parses")
    .resolve(&CatalogContext::default())
    .expect("the catalog resolves");
    let labels = get_route_labels(&catalog).expect("labels resolve");
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
        .expect("catch-all route");
    assert_eq!(catch_all.label, "Everything else · Anthropic");
    assert!(!catch_all.declared_name);
    assert_eq!(labels.label_of("no-such-route"), "no-such-route");
    assert_eq!(
        labels.find_provider("anthropic").map(|p| p.label.as_str()),
        Some("Anthropic")
    );
}
