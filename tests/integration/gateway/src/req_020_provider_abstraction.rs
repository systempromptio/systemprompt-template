//! REQ-020 Model Provider Abstraction — "Applications use a provider-neutral
//! gateway/API so models/providers can be changed without application-layer
//! rewrites."
//!
//! Two providers speaking different wire protocols sit behind the same
//! external model name; which one serves it is decided by route order and the
//! `upstream_model` rewrite, both plain configuration. Swapping the provider
//! is therefore a config edit the client never sees — the register's low
//! switching cost, demonstrated at the type level the loader enforces.

use systemprompt::models::profile::WireProtocol;
use systemprompt::models::services::ModelGovernance;
use systemprompt::models::wire::canonical::CanonicalRequest;

use crate::support::{config, model, provider, registry, route};

fn request_for(model_name: &str) -> CanonicalRequest {
    CanonicalRequest {
        model: model_name.to_owned(),
        ..CanonicalRequest::default()
    }
}

#[test]
fn the_first_matching_route_decides_the_provider() {
    let reg = registry(vec![
        provider(
            "anthropic-upstream",
            WireProtocol::Anthropic,
            ModelGovernance::default(),
            vec![model("claude-large")],
        ),
        provider(
            "openai-compatible",
            WireProtocol::OpenAiChat,
            ModelGovernance::default(),
            vec![model("oss-large")],
        ),
    ]);
    let cfg = config(vec![
        route("primary", "claude-*", "anthropic-upstream"),
        route("fallback", "*", "openai-compatible"),
    ]);

    let resolved = cfg
        .resolve_route(&reg, &request_for("claude-large"))
        .expect("a claude request resolves");
    assert_eq!(resolved.provider.as_str(), "anthropic-upstream");

    let entry = resolved.resolve(&reg).expect("the provider is registered");
    assert!(matches!(entry.wire, WireProtocol::Anthropic));
}

#[test]
fn reordering_routes_substitutes_the_provider_without_touching_the_client_name() {
    let reg = registry(vec![
        provider(
            "anthropic-upstream",
            WireProtocol::Anthropic,
            ModelGovernance::default(),
            vec![model("claude-large")],
        ),
        provider(
            "openai-compatible",
            WireProtocol::OpenAiChat,
            ModelGovernance::default(),
            vec![model("oss-large")],
        ),
    ]);
    let mut redirect = route("redirect", "claude-*", "openai-compatible");
    redirect.upstream_model = Some("oss-large".to_owned());
    let cfg = config(vec![
        redirect,
        route("primary", "claude-*", "anthropic-upstream"),
    ]);

    let resolved = cfg
        .resolve_route(&reg, &request_for("claude-large"))
        .expect("the request still resolves");
    assert_eq!(
        resolved.provider.as_str(),
        "openai-compatible",
        "first match wins, so the redirect route captures the same external name"
    );
    let entry = resolved.resolve(&reg).expect("the provider is registered");
    assert!(
        matches!(entry.wire, WireProtocol::OpenAiChat),
        "the substituted provider speaks a different wire protocol entirely"
    );
}

#[test]
fn upstream_model_rewrites_the_name_only_when_declared() {
    let mut rewriting = route("rewrite", "claude-*", "openai-compatible");
    rewriting.upstream_model = Some("oss-large".to_owned());
    assert_eq!(rewriting.effective_upstream_model("claude-large"), "oss-large");

    let passthrough = route("passthrough", "claude-*", "anthropic-upstream");
    assert_eq!(
        passthrough.effective_upstream_model("claude-large"),
        "claude-large",
        "without a rewrite the requested name is forwarded as-is"
    );
}
