//! REQ-024 Shadow AI Detection & Blocking — "Unauthorized or unmanaged AI use
//! can be detected and, where policy requires, blocked or escalated through
//! the governed platform."
//!
//! The gateway's dispatch decision (`ResolvedCatalog::resolve`) is the
//! blocking half: a model that is not in the catalog is refused, and a route
//! glob never exposes a name the catalog does not declare, so traffic to an
//! un-governed model cannot pass through the platform. There is no open
//! posture to opt into.

use systemprompt::manifest::services::ModelRequest;
use systemprompt::manifest::services::catalog::{ResolveError, ResolvedCatalog};

use crate::support::{catalog, provider};

fn governed_catalog() -> ResolvedCatalog {
    catalog(
        &[provider("governed-provider", "anthropic")],
        "models:\n  governed-model: { clients: [] }\ndeployments:\n  governed-model: [governed-provider/api]\ngateway:\n  enabled: false\n  routes:\n    - id: governed\n      models: \"governed-*\"\n",
    )
    .expect("the catalog resolves")
}

#[test]
fn an_unlisted_model_is_refused() {
    let cat = governed_catalog();

    match cat.resolve(&ModelRequest::new("shadow-model")) {
        Err(ResolveError::UnknownModel { requested }) => assert_eq!(requested, "shadow-model"),
        other => panic!("a model with no catalog entry is refused, got {other:?}"),
    }
}

#[test]
fn a_cataloged_model_is_dispatched() {
    let cat = governed_catalog();

    let target = cat
        .resolve(&ModelRequest::new("governed-model"))
        .expect("a catalog model resolves");
    assert_eq!(target.route.as_ref().map(|r| r.as_str()), Some("governed"));
}

#[test]
fn a_route_glob_does_not_expose_an_uncataloged_model() {
    let cat = governed_catalog();

    assert!(
        matches!(
            cat.resolve(&ModelRequest::new("governed-anything")),
            Err(ResolveError::UnknownModel { .. })
        ),
        "a route pattern match alone exposes nothing"
    );
}
