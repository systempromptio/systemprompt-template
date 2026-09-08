#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::needless_pass_by_value,
    clippy::redundant_clone,
    reason = "test code: panics are the assertion mechanism and clones keep fixtures readable"
)]

use std::path::{Path, PathBuf};

use systemprompt_web_admin::repositories::config::gateway::{
    create_route, delete_route, get_gateway_config_from_file, glob_match, reorder_routes,
    slugify_pattern, synthesize_route_id, update_gateway_settings, update_route,
};
use systemprompt_web_admin::types::{GatewayRouteView, UpdateGatewaySettingsRequest};
use systemprompt_web_shared::error::MarketplaceError;

// The gateway block lives in services/ai/gateway.yaml since core 0.44; the
// file has the same `gateway:` root key the profile used to carry.
fn gateway_file(dir: &Path, body: &str) -> PathBuf {
    let path = dir.join("gateway.yaml");
    std::fs::write(&path, body).expect("write gateway file");
    path
}

fn route(id: &str, pattern: &str, provider: &str) -> GatewayRouteView {
    GatewayRouteView {
        id: id.to_owned(),
        model_pattern: pattern.to_owned(),
        provider: provider.to_owned(),
        ..Default::default()
    }
}

#[test]
fn glob_match_enforces_prefix_suffix_and_length() {
    assert!(glob_match("claude-*-latest", "claude-3-latest"));
    assert!(!glob_match("claude-*-latest", "claude-3-preview"));
    assert!(glob_match("*sonnet*", "claude-sonnet-4"));
    assert!(!glob_match("*sonnet*", "claude-opus-4"));
    assert!(glob_match("abc*def", "abcdef"));
    assert!(
        !glob_match("abc*bc", "abc"),
        "prefix and suffix must not overlap"
    );
    assert!(glob_match("*", ""));
    assert!(!glob_match("gpt-4", "gpt-4o"));
}

#[test]
fn glob_match_anchors_the_leading_segment() {
    assert!(!glob_match("claude*x*", "xclaudex"));
    assert!(glob_match("claude*x*", "claude-3x-end"));
}

#[test]
fn slugify_collapses_runs_and_trims_separators() {
    assert_eq!(slugify_pattern("--foo..bar--"), "foo-bar");
    assert_eq!(slugify_pattern("///"), "route");
    assert_eq!(slugify_pattern("**"), "starstar");
    assert_eq!(slugify_pattern("A_B C"), "a-b-c");
}

#[test]
fn synthesized_ids_differ_per_pattern() {
    assert_ne!(
        synthesize_route_id("claude-*", "anthropic"),
        synthesize_route_id("gpt-*", "anthropic")
    );
}

#[test]
fn create_route_bootstraps_a_missing_gateway_block() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = gateway_file(dir.path(), "name: local\n");
    let index = create_route(&path, &route("", "claude-*", "anthropic"))?;
    assert_eq!(index, 0);

    let cfg = get_gateway_config_from_file(&path)?;
    assert_eq!(cfg.routes.len(), 1);
    assert!(cfg.routes[0].id.starts_with("claude-star-"));
    assert!(!cfg.enabled, "enabled defaults to false");
    assert_eq!(cfg.auth_scheme, "bearer");
    assert_eq!(cfg.inference_path_prefix, "/v1");
    assert_eq!(cfg.config_path, path.display().to_string());
    Ok(())
}

#[test]
fn create_route_rejects_blank_pattern_or_provider() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = gateway_file(dir.path(), "gateway:\n  routes: []\n");
    assert!(matches!(
        create_route(&path, &route("a", "   ", "anthropic")),
        Err(MarketplaceError::BadRequest(_))
    ));
    assert!(matches!(
        create_route(&path, &route("a", "claude-*", "  ")),
        Err(MarketplaceError::BadRequest(_))
    ));
    assert!(get_gateway_config_from_file(&path)?.routes.is_empty());
    Ok(())
}

#[test]
fn upstream_model_and_extra_headers_survive_a_round_trip() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = gateway_file(dir.path(), "gateway:\n  routes: []\n");
    let mut r = route("cerebras", "claude-*", "cerebras");
    r.upstream_model = Some("gpt-oss-120b".to_owned());
    r.extra_headers
        .insert("x-tenant".to_owned(), "acme".to_owned());
    create_route(&path, &r)?;

    let cfg = get_gateway_config_from_file(&path)?;
    assert_eq!(
        cfg.routes[0].upstream_model.as_deref(),
        Some("gpt-oss-120b")
    );
    assert_eq!(
        cfg.routes[0]
            .extra_headers
            .get("x-tenant")
            .map(String::as_str),
        Some("acme")
    );
    Ok(())
}

#[test]
fn routes_missing_provider_are_dropped_from_the_read_view() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = gateway_file(
        dir.path(),
        "gateway:\n  routes:\n    - model_pattern: broken\n    - provider: alone\n    - model_pattern: ok\n      provider: anthropic\n",
    );
    let cfg = get_gateway_config_from_file(&path)?;
    assert_eq!(cfg.routes.len(), 1);
    assert_eq!(cfg.routes[0].model_pattern, "ok");
    Ok(())
}

#[test]
fn update_route_replaces_in_place_and_reports_out_of_range() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = gateway_file(dir.path(), "gateway:\n  routes: []\n");
    create_route(&path, &route("first", "claude-*", "anthropic"))?;

    assert!(update_route(
        &path,
        0,
        &route("first", "claude-*", "openai")
    )?);
    let cfg = get_gateway_config_from_file(&path)?;
    assert_eq!(cfg.routes[0].provider, "openai");

    assert!(
        !update_route(&path, 9, &route("first", "claude-*", "openai"))?,
        "an out-of-range index is a miss, not an error"
    );
    Ok(())
}

#[test]
fn update_route_validates_before_touching_disk() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = gateway_file(dir.path(), "gateway:\n  routes: []\n");
    create_route(&path, &route("first", "claude-*", "anthropic"))?;
    assert!(matches!(
        update_route(&path, 0, &route("first", "", "openai")),
        Err(MarketplaceError::BadRequest(_))
    ));
    assert_eq!(
        get_gateway_config_from_file(&path)?.routes[0].provider,
        "anthropic"
    );
    Ok(())
}

#[test]
fn delete_route_removes_by_index() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = gateway_file(dir.path(), "gateway:\n  routes: []\n");
    create_route(&path, &route("a", "a-*", "anthropic"))?;
    create_route(&path, &route("b", "b-*", "openai"))?;

    assert!(delete_route(&path, 0)?);
    let cfg = get_gateway_config_from_file(&path)?;
    assert_eq!(cfg.routes.len(), 1);
    assert_eq!(cfg.routes[0].id, "b");

    assert!(!delete_route(&path, 5)?);
    Ok(())
}

#[test]
fn reorder_rejects_wrong_length_and_non_permutations() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = gateway_file(dir.path(), "gateway:\n  routes: []\n");
    create_route(&path, &route("a", "a-*", "anthropic"))?;
    create_route(&path, &route("b", "b-*", "openai"))?;

    assert!(matches!(
        reorder_routes(&path, &[0]),
        Err(MarketplaceError::BadRequest(_))
    ));
    assert!(matches!(
        reorder_routes(&path, &[0, 0]),
        Err(MarketplaceError::BadRequest(_))
    ));
    assert!(matches!(
        reorder_routes(&path, &[0, 7]),
        Err(MarketplaceError::BadRequest(_))
    ));

    let ids: Vec<String> = get_gateway_config_from_file(&path)?
        .routes
        .into_iter()
        .map(|r| r.id)
        .collect();
    assert_eq!(ids, vec!["a".to_owned(), "b".to_owned()]);
    Ok(())
}

#[test]
fn update_gateway_settings_writes_each_supplied_field() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = gateway_file(dir.path(), "name: local\n");
    let cfg = update_gateway_settings(
        &path,
        &UpdateGatewaySettingsRequest {
            enabled: Some(true),
            auth_scheme: Some("oauth".to_owned()),
            inference_path_prefix: Some("/api/v2".to_owned()),
        },
    )?;
    assert!(cfg.enabled);
    assert_eq!(cfg.auth_scheme, "oauth");
    assert_eq!(cfg.inference_path_prefix, "/api/v2");

    let unchanged = update_gateway_settings(
        &path,
        &UpdateGatewaySettingsRequest {
            enabled: None,
            auth_scheme: None,
            inference_path_prefix: None,
        },
    )?;
    assert!(unchanged.enabled);
    assert_eq!(unchanged.auth_scheme, "oauth");
    Ok(())
}

#[test]
fn inference_path_prefix_must_be_absolute() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = gateway_file(dir.path(), "gateway:\n  enabled: true\n");
    assert!(matches!(
        update_gateway_settings(
            &path,
            &UpdateGatewaySettingsRequest {
                enabled: None,
                auth_scheme: None,
                inference_path_prefix: Some("v1".to_owned()),
            },
        ),
        Err(MarketplaceError::BadRequest(_))
    ));
    assert_eq!(
        get_gateway_config_from_file(&path)?.inference_path_prefix,
        "/v1"
    );
    Ok(())
}

#[test]
fn a_non_mapping_profile_root_is_an_internal_error() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let path = gateway_file(dir.path(), "- just\n- a list\n");
    assert!(matches!(
        create_route(&path, &route("a", "a-*", "anthropic")),
        Err(MarketplaceError::Internal(_))
    ));
    Ok(())
}
