use systemprompt_web_admin::repositories::governance_grp::gateway::{
    create_route, ensure_route_ids, find_matching_route, find_matching_route_index,
    find_route_index_by_id, get_gateway_config, glob_match, reorder_routes, slugify_pattern,
    synthesize_route_id, validate_route,
};
use systemprompt_web_admin::types::GatewayRouteView;
use systemprompt_web_shared::error::MarketplaceError;

const TWO_ROUTE_PROFILE: &str = r"gateway:
  enabled: true
  routes:
    - model_pattern: claude-*
      provider: anthropic
      endpoint: https://api.anthropic.com
      api_key_secret: anthropic_key
    - model_pattern: '*'
      provider: openai
      endpoint: https://api.openai.com
      api_key_secret: openai_key
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
            endpoint: "https://a".into(),
            api_key_secret: "k".into(),
            ..Default::default()
        },
        GatewayRouteView {
            id: "star-def456".into(),
            model_pattern: "*".into(),
            provider: "b".into(),
            endpoint: "https://b".into(),
            api_key_secret: "k".into(),
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
fn rejects_inline_secret() {
    let route = GatewayRouteView {
        model_pattern: "*".into(),
        provider: "anthropic".into(),
        endpoint: "https://api.anthropic.com".into(),
        api_key_secret: "sk-abc123".into(),
        ..Default::default()
    };
    assert!(validate_route(&route).is_err());
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
    let a = synthesize_route_id("claude-*", "anthropic", "https://api.anthropic.com");
    let b = synthesize_route_id("claude-*", "anthropic", "https://api.anthropic.com");
    assert_eq!(a, b);
    assert!(a.starts_with("claude-star-"));
    let c = synthesize_route_id("claude-*", "anthropic", "https://other.example");
    assert_ne!(a, c);
}

#[test]
fn ensure_route_ids_backfills_missing_then_idempotent() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("profile.yaml");
    std::fs::write(&path, TWO_ROUTE_PROFILE)?;

    let changed = ensure_route_ids(&path)?;
    assert!(changed, "first call should backfill ids");

    let cfg = get_gateway_config(&path)?;
    assert_eq!(cfg.routes.len(), 2);
    assert!(!cfg.routes[0].id.is_empty());
    assert!(!cfg.routes[1].id.is_empty());
    assert_ne!(cfg.routes[0].id, cfg.routes[1].id);

    let id0_before = cfg.routes[0].id.clone();
    let id1_before = cfg.routes[1].id.clone();

    let changed_again = ensure_route_ids(&path)?;
    assert!(!changed_again, "second call should be a no-op");

    let cfg2 = get_gateway_config(&path)?;
    assert_eq!(cfg2.routes[0].id, id0_before);
    assert_eq!(cfg2.routes[1].id, id1_before);
    Ok(())
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
        endpoint: "https://api.anthropic.com".into(),
        api_key_secret: "anthropic_key".into(),
        ..Default::default()
    };
    create_route(&path, &route)?;
    assert!(matches!(
        create_route(&path, &route),
        Err(MarketplaceError::BadRequest(_))
    ));
    Ok(())
}
