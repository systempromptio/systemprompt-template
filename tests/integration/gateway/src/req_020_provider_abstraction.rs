//! REQ-020 Model Provider Abstraction — "Applications use a provider-neutral
//! gateway/API so models/providers can be changed without application-layer
//! rewrites."
//!
//! Two providers speaking different wire protocols serve the same external
//! model name; which one serves it is decided by the model's deployment order,
//! a route pinning one deployment, and a deployment's `upstream_model`
//! rewrite, all plain configuration. Swapping the provider is therefore a
//! config edit the client never sees — the register's low switching cost,
//! demonstrated at the type level the loader enforces.

use systemprompt::identifiers::ModelId;
use systemprompt::manifest::services::ModelRequest;
use systemprompt::wire::WireProtocol;
use systemprompt::wire::canonical::{CanonicalContent, CanonicalMessage, CanonicalRequest, Role};

use crate::support::{catalog, provider};

fn request_for(model_name: &str) -> CanonicalRequest {
    CanonicalRequest::new(
        ModelId::new(model_name),
        vec![CanonicalMessage {
            role: Role::User,
            content: vec![CanonicalContent::text("Summarize the report".to_owned())],
        }],
        256,
    )
}

const MODELS: &str = "models:\n  claude-large: { clients: [] }\n";

#[test]
fn the_first_deployment_decides_the_provider() {
    let cat = catalog(
        &[
            provider("anthropic-upstream", "anthropic"),
            provider("openai-compatible", "openai-chat"),
        ],
        &format!(
            "{MODELS}deployments:\n  claude-large: [anthropic-upstream/api, openai-compatible/api]\n"
        ),
    )
    .expect("the catalog resolves");

    let request = request_for("claude-large");
    let target = cat
        .resolve(&ModelRequest::new("claude-large").with_request(&request))
        .expect("a claude request resolves");
    let primary = target.primary().expect("the chain has a primary hop");
    assert_eq!(primary.provider.as_str(), "anthropic-upstream");
    assert!(matches!(primary.wire, WireProtocol::Anthropic));
}

#[test]
fn a_route_pinning_another_deployment_substitutes_the_provider_without_touching_the_client_name() {
    let cat = catalog(
        &[
            provider("anthropic-upstream", "anthropic"),
            provider("openai-compatible", "openai-chat"),
        ],
        &format!(
            "{MODELS}deployments:\n  claude-large:\n    - anthropic-upstream/api\n    - {{ endpoint: openai-compatible/api, upstream_model: oss-large }}\ngateway:\n  enabled: false\n  routes:\n    - id: redirect\n      models: \"claude-*\"\n      deployments: [openai-compatible/api]\n"
        ),
    )
    .expect("the catalog resolves");

    let request = request_for("claude-large");
    let target = cat
        .resolve(&ModelRequest::new("claude-large").with_request(&request))
        .expect("the request still resolves");
    assert_eq!(target.requested, "claude-large");
    assert_eq!(target.route.as_ref().map(|r| r.as_str()), Some("redirect"));
    let primary = target.primary().expect("the chain has a primary hop");
    assert_eq!(
        primary.provider.as_str(),
        "openai-compatible",
        "the pinning route captures the same external name"
    );
    assert!(
        matches!(primary.wire, WireProtocol::OpenAiChat),
        "the substituted provider speaks a different wire protocol entirely"
    );
}

#[test]
fn upstream_model_rewrites_the_name_only_when_declared() {
    let cat = catalog(
        &[
            provider("anthropic-upstream", "anthropic"),
            provider("openai-compatible", "openai-chat"),
        ],
        &format!(
            "{MODELS}deployments:\n  claude-large:\n    - {{ endpoint: openai-compatible/api, upstream_model: oss-large }}\n    - anthropic-upstream/api\n"
        ),
    )
    .expect("the catalog resolves");

    let hops = cat.deployments("claude-large");
    assert_eq!(hops[0].upstream_model, "oss-large");
    assert_eq!(
        hops[1].upstream_model, "claude-large",
        "without a rewrite the requested name is forwarded as-is"
    );
}
