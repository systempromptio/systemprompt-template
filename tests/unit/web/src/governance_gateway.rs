#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::needless_pass_by_value,
    clippy::redundant_clone,
    reason = "test code: panics are the assertion mechanism and clones keep fixtures readable"
)]

use systemprompt::manifest::services::ProviderRegistry;
use systemprompt_web_admin::repositories::config::gateway::{
    create_route, find_matching_route, find_matching_route_index, find_route_index_by_id,
    get_gateway_config, glob_match, reorder_routes, retain_client_facing, slugify_pattern,
    synthesize_route_id, update_route, validate_route,
};
use systemprompt_web_admin::types::GatewayRouteView;
use systemprompt_web_shared::error::MarketplaceError;

const TWO_ROUTE_PROFILE: &str = r"gateway:
  enabled: true
  routes:
    - model_pattern: claude-*
      provider: anthropic
    - model_pattern: '*'
      provider: openai
";

#[test]
fn glob_matches_exact_and_wildcard() {
    assert!(glob_match("*", "anything"));
    assert!(glob_match("claude-*", "claude-sonnet"));
    assert!(!glob_match("claude-*", "gpt-4"));
    assert!(glob_match("gpt-4", "gpt-4"));
    assert!(glob_match("*-latest", "gpt-4-latest"));
}

#[test]
fn first_match_wins() {
    let routes = vec![
        GatewayRouteView {
            id: "claude-abc123".into(),
            model_pattern: "claude-*".into(),
            provider: "a".into(),
            ..Default::default()
        },
        GatewayRouteView {
            id: "star-def456".into(),
            model_pattern: "*".into(),
            provider: "b".into(),
            ..Default::default()
        },
    ];
    assert_eq!(find_matching_route_index(&routes, "claude-3"), Some(0));
    assert_eq!(find_matching_route_index(&routes, "gpt-4"), Some(1));
    assert_eq!(
        find_matching_route(&routes, "claude-3").map(|r| r.id.as_str()),
        Some("claude-abc123"),
    );
    assert_eq!(find_route_index_by_id(&routes, "star-def456"), Some(1));
}

#[test]
fn validate_rejects_empty_required_fields() {
    let no_pattern = GatewayRouteView {
        provider: "anthropic".into(),
        ..Default::default()
    };
    assert!(validate_route(&no_pattern).is_err());

    let no_provider = GatewayRouteView {
        model_pattern: "*".into(),
        ..Default::default()
    };
    assert!(validate_route(&no_provider).is_err());
}

#[test]
fn slugify_replaces_star_and_non_alnum() {
    assert_eq!(slugify_pattern("*"), "star");
    assert_eq!(slugify_pattern("claude-*"), "claude-star");
    assert_eq!(slugify_pattern("*-latest"), "star-latest");
    assert_eq!(slugify_pattern("GPT-4"), "gpt-4");
    assert_eq!(slugify_pattern("foo.bar/baz"), "foo-bar-baz");
    assert_eq!(slugify_pattern(""), "route");
}

#[test]
fn synthesized_id_is_stable() {
    let a = synthesize_route_id("claude-*", "anthropic");
    let b = synthesize_route_id("claude-*", "anthropic");
    assert_eq!(a, b);
    assert!(a.starts_with("claude-star-"));
    let c = synthesize_route_id("claude-*", "openai");
    assert_ne!(a, c, "provider change must produce a different id");
}

#[test]
fn ids_stable_across_reorder() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("profile.yaml");
    std::fs::write(&path, TWO_ROUTE_PROFILE)?;
    let cfg = get_gateway_config(&path)?;
    let id0 = cfg.routes[0].id.clone();
    let id1 = cfg.routes[1].id.clone();

    reorder_routes(&path, &[1, 0])?;
    let cfg2 = get_gateway_config(&path)?;
    assert_eq!(cfg2.routes[0].id, id1);
    assert_eq!(cfg2.routes[1].id, id0);
    Ok(())
}

#[test]
fn create_route_rejects_duplicate_id() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("profile.yaml");
    std::fs::write(
        &path,
        r"gateway:
  enabled: true
  routes: []
",
    )?;
    let route = GatewayRouteView {
        id: "fixed-id".into(),
        model_pattern: "claude-*".into(),
        provider: "anthropic".into(),
        ..Default::default()
    };
    create_route(&path, &route)?;
    assert!(matches!(
        create_route(&path, &route),
        Err(MarketplaceError::BadRequest(_))
    ));
    Ok(())
}

const COMMENTED_PROFILE: &str = r"# Gateway routes - which model patterns map onto which provider.
# Edit by hand or with: systemprompt admin config gateway ...
gateway:
  enabled: true
  routes:
  - model_pattern: claude-*
    provider: anthropic
  - id: hand-written
    model_pattern: gpt-*
    provider: openai
";

#[test]
fn reading_the_config_never_writes_the_file() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("gateway.yaml");
    std::fs::write(&path, COMMENTED_PROFILE)?;

    let cfg = get_gateway_config(&path)?;
    assert_eq!(cfg.routes.len(), 2);
    assert_eq!(cfg.routes[1].id, "hand-written");
    assert!(
        !cfg.routes[0].id.is_empty(),
        "a route without an id: key still reports a synthesized one"
    );

    assert_eq!(
        std::fs::read_to_string(&path)?,
        COMMENTED_PROFILE,
        "a read must leave the operator's file byte-identical"
    );
    Ok(())
}

#[test]
fn updating_one_route_changes_only_that_field() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("gateway.yaml");
    std::fs::write(&path, COMMENTED_PROFILE)?;

    let cfg = get_gateway_config(&path)?;
    let mut edited = cfg.routes[0].clone();
    edited.provider = "vertex".into();
    edited.id = synthesize_route_id("claude-*", "anthropic");
    assert!(update_route(&path, 0, &edited)?);

    let after = std::fs::read_to_string(&path)?;
    assert!(
        after.starts_with("# Gateway routes"),
        "the comment header must survive the write: {after}"
    );
    assert!(
        !after.contains("id: claude-star-"),
        "a synthesized id must not be written into the file: {after}"
    );
    assert!(
        after.contains("id: hand-written"),
        "a hand-chosen id must survive: {after}"
    );

    let expected: Vec<String> = COMMENTED_PROFILE
        .lines()
        .map(|line| line.replace("provider: anthropic", "provider: vertex"))
        .collect();
    let actual: Vec<String> = after.lines().map(str::to_owned).collect();
    assert_eq!(actual, expected, "only the edited field may differ");
    Ok(())
}

const TWO_SURFACE_CATALOG: &str = r"- name: anthropic
  wire: anthropic
  surface: anthropic
  endpoint: https://api.anthropic.com/v1
  api_key_secret: anthropic
- name: e2e-mock
  wire: anthropic
  surface: backend
  endpoint: https://mock.invalid/v1
  api_key_secret: e2e_mock
";

#[test]
fn a_backend_surface_provider_is_never_client_facing() -> anyhow::Result<()> {
    let providers: ProviderRegistry = serde_yaml::from_str(TWO_SURFACE_CATALOG)?;
    let routes = vec![
        GatewayRouteView {
            id: "claude".into(),
            model_pattern: "claude-*".into(),
            provider: "anthropic".into(),
            ..Default::default()
        },
        GatewayRouteView {
            id: "e2e-mock-route".into(),
            model_pattern: "e2e-mock-*".into(),
            provider: "e2e-mock".into(),
            ..Default::default()
        },
        GatewayRouteView {
            id: "unknown".into(),
            model_pattern: "ghost-*".into(),
            provider: "not-in-the-catalog".into(),
            ..Default::default()
        },
    ];

    let kept = retain_client_facing(routes, &providers);
    assert_eq!(
        kept.iter().map(|r| r.provider.as_str()).collect::<Vec<_>>(),
        vec!["anthropic", "not-in-the-catalog"],
        "only a provider that declares `surface: backend` is hidden; an unknown one passes through"
    );
    Ok(())
}
