//! `gateway.yaml` against `gateway_routes`: the declaration parses every
//! field core's `GatewayRoute` carries, the drift names what moved and
//! sees order, and the render puts the rows back into the file without
//! touching the settings above them.

use chrono::Utc;
use systemprompt_web_admin::repositories::gateway_routes::declared::{
    DeclaredRoutes, parse_declared_routes,
};
use systemprompt_web_admin::repositories::gateway_routes::drift::compute_route_drift;
use systemprompt_web_admin::repositories::gateway_routes::render::render_routes_export;
use systemprompt_web_admin::repositories::gateway_routes::rows::{
    RouteRow, SOURCE_CODE, SOURCE_DASHBOARD,
};

const DECLARED: &str = r"# Gateway routes — header comment.
gateway:
  enabled: true
  routes:
  - id: claude-star
    model_pattern: claude-*
    provider: anthropic
    fallback_provider: vertex
    fallback_upstream_model: claude-sonnet-5
  - model_pattern: gemini-*
    provider: gemini
  - id: gpt
    model_pattern: gpt-4.1*
    provider: openai
    requires:
      governance: [european]
  default_provider: anthropic
  quota_fault_mode: closed
";

fn rows_from(declared: &DeclaredRoutes, source: &str) -> Vec<RouteRow> {
    declared
        .routes
        .iter()
        .enumerate()
        .map(|(i, d)| RouteRow {
            route: d.route.clone(),
            position: i32::try_from(i).expect("small index"),
            explicit_id: d.explicit_id,
            source: source.to_owned(),
            updated_at: Utc::now(),
        })
        .collect()
}

#[test]
fn declaration_carries_every_route_field_and_synthesises_missing_ids() {
    let declared = parse_declared_routes(DECLARED).expect("parses");
    assert_eq!(declared.routes.len(), 3);
    let claude = &declared.routes[0];
    assert!(claude.explicit_id);
    assert_eq!(claude.route.fallback_provider.as_deref(), Some("vertex"));
    assert_eq!(
        claude.route.fallback_upstream_model.as_deref(),
        Some("claude-sonnet-5")
    );
    let gemini = &declared.routes[1];
    assert!(!gemini.explicit_id);
    assert!(!gemini.route.id.is_empty(), "synthesised id");
    assert!(declared.routes[2].route.requires.is_some());
}

#[test]
fn declared_hash_is_blind_to_comments_and_sees_order() {
    let a = parse_declared_routes(DECLARED).expect("parses");
    let b = parse_declared_routes(&DECLARED.replace("# Gateway routes — header comment.\n", ""))
        .expect("parses");
    assert_eq!(a.declared_hash(), b.declared_hash());
    let mut reordered = a.clone();
    reordered.routes.swap(0, 1);
    assert_ne!(a.declared_hash(), reordered.declared_hash());
}

#[test]
fn parse_refuses_a_repeated_id_and_a_route_without_a_provider() {
    let dup = DECLARED.replace("  - id: gpt\n", "  - id: claude-star\n");
    assert!(
        parse_declared_routes(&dup)
            .expect_err("refused")
            .contains("repeats id")
    );
    let bare = "gateway:\n  routes:\n  - model_pattern: x-*\n";
    assert!(
        parse_declared_routes(bare)
            .expect_err("refused")
            .contains("routes[0]")
    );
}

#[test]
fn in_step_rows_are_clean() {
    let declared = parse_declared_routes(DECLARED).expect("parses");
    let drift = compute_route_drift(&declared, &rows_from(&declared, SOURCE_CODE));
    assert!(drift.is_clean(), "{drift:?}");
}

#[test]
fn drift_names_missing_orphan_changed_fields_and_console_origin() {
    let declared = parse_declared_routes(DECLARED).expect("parses");
    let mut rows = rows_from(&declared, SOURCE_CODE);
    rows.remove(2);
    rows[0].route.fallback_provider = Some("cerebras".to_owned());
    rows[0].source = SOURCE_DASHBOARD.to_owned();
    rows.push(RouteRow {
        route: systemprompt_web_admin::types::GatewayRouteView {
            id: "console-only".to_owned(),
            model_pattern: "o4-*".to_owned(),
            provider: "openai".to_owned(),
            ..Default::default()
        },
        position: 9,
        explicit_id: true,
        source: SOURCE_DASHBOARD.to_owned(),
        updated_at: Utc::now(),
    });
    let drift = compute_route_drift(&declared, &rows);
    assert_eq!(drift.missing_in_db, vec!["gpt".to_owned()]);
    assert_eq!(drift.only_in_db.len(), 1);
    assert!(drift.only_in_db[0].dashboard);
    assert_eq!(drift.changed.len(), 1);
    assert_eq!(
        drift.changed[0].fields,
        vec!["fallback_provider".to_owned()]
    );
    assert!(drift.changed[0].dashboard);
    assert!(drift.changed[0].in_db.contains("fallback cerebras"));
    assert!(drift.reordered.is_none(), "shared routes keep their order");
}

#[test]
fn drift_reports_order_once_not_per_route() {
    let declared = parse_declared_routes(DECLARED).expect("parses");
    let mut rows = rows_from(&declared, SOURCE_CODE);
    rows.swap(0, 2);
    let drift = compute_route_drift(&declared, &rows);
    assert!(drift.changed.is_empty());
    let (code, db) = drift.reordered.expect("order differs");
    assert_eq!(code[0], "claude-star");
    assert_eq!(db[0], "gpt");
}

#[test]
fn render_replaces_the_sequence_and_keeps_settings_and_header() {
    let declared = parse_declared_routes(DECLARED).expect("parses");
    let mut rows = rows_from(&declared, SOURCE_CODE);
    rows.truncate(1);
    let out = render_routes_export(DECLARED, &rows).expect("renders");
    assert!(out.starts_with("# Gateway routes — header comment.\n"));
    assert!(out.contains("quota_fault_mode: closed"));
    assert!(out.contains("default_provider: anthropic"));
    assert!(out.contains("fallback_provider: vertex"));
    assert!(!out.contains("gemini-*"));
    let back = parse_declared_routes(&out).expect("round-trips");
    assert_eq!(back.routes.len(), 1);
    assert_eq!(back.declared_hash(), {
        let mut one = declared;
        one.routes.truncate(1);
        one.declared_hash()
    });
}

#[test]
fn render_omits_an_id_the_loader_would_synthesise() {
    let declared = parse_declared_routes(DECLARED).expect("parses");
    let rows = rows_from(&declared, SOURCE_CODE);
    let out = render_routes_export(DECLARED, &rows).expect("renders");
    assert_eq!(
        out.matches("id:").count(),
        2,
        "only the two explicit ids: {out}"
    );
}
