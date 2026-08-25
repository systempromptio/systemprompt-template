//! REQ-037 Data Residency Routing — "Routing rules can enforce that
//! classified workloads remain within the configured jurisdiction/region/
//! sovereign endpoint."
//!
//! A route declaring `requires: { european: true }` refuses to validate when
//! any model it can reach lacks a `european` governance declaration, and the
//! same `requires:`/`governance:` blocks round-trip from profile YAML — so
//! the boot-time check the register asks for is enforced by configuration the
//! operator actually writes.

use systemprompt::models::profile::{
    GatewayConfigSpec, GatewayProfileError, GatewayRoute, ProviderEntry, RouteRequirements,
    WireProtocol,
};
use systemprompt::models::services::ModelGovernance;

use crate::support::{config, model, provider, registry, route};

fn european_route() -> GatewayRoute {
    let mut r = route("eu-classified", "classified-*", "eu-provider");
    r.requires = Some(RouteRequirements {
        european: true,
        no_retain: false,
    });
    r
}

#[test]
fn a_european_route_over_a_non_european_provider_fails_validation() {
    let reg = registry(vec![provider(
        "eu-provider",
        WireProtocol::OpenAiChat,
        ModelGovernance {
            european: false,
            no_retain: false,
        },
        vec![model("classified-large")],
    )]);
    let cfg = config(vec![european_route()]);

    match cfg.validate(&reg) {
        Err(GatewayProfileError::RouteGovernanceUnsatisfied {
            route,
            model,
            requirements,
        }) => {
            assert_eq!(route, "eu-classified");
            assert_eq!(model, "classified-large");
            assert_eq!(requirements, "european");
        },
        other => panic!("expected RouteGovernanceUnsatisfied, got {other:?}"),
    }
}

#[test]
fn the_same_route_validates_once_the_provider_declares_european_residency() {
    let reg = registry(vec![provider(
        "eu-provider",
        WireProtocol::OpenAiChat,
        ModelGovernance {
            european: true,
            no_retain: false,
        },
        vec![model("classified-large")],
    )]);
    let cfg = config(vec![european_route()]);

    assert!(
        cfg.validate(&reg).is_ok(),
        "a european provider satisfies the european route requirement"
    );
}

#[test]
fn unmet_names_exactly_the_missing_residency_guarantee() {
    let requires = RouteRequirements {
        european: true,
        no_retain: false,
    };
    let unmet = requires.unmet(ModelGovernance {
        european: false,
        no_retain: true,
    });
    assert_eq!(unmet, vec!["european"]);
    assert!(
        requires
            .unmet(ModelGovernance {
                european: true,
                no_retain: false,
            })
            .is_empty(),
        "a compliant posture leaves nothing unmet"
    );
}

#[test]
fn residency_blocks_round_trip_from_profile_yaml() {
    let spec: GatewayConfigSpec = serde_yaml::from_str(
        r#"
enabled: false
routes:
  - id: eu-classified
    model_pattern: "classified-*"
    provider: eu-provider
    requires:
      european: true
"#,
    )
    .expect("the requires block parses from YAML");
    let requires = spec.routes[0]
        .requires
        .expect("the parsed route carries its requirements");
    assert!(requires.european);
    assert!(!requires.no_retain);

    let entry: ProviderEntry = serde_yaml::from_str(
        r#"
name: eu-provider
wire: openai-chat
surface: backend
endpoint: https://eu.example.com/v1
api_key_secret: eu_key
governance:
  european: true
"#,
    )
    .expect("the governance block parses from YAML");
    assert!(entry.governance.european);
    assert!(!entry.governance.no_retain);
}
