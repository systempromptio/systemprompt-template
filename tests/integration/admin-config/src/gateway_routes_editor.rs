//! Dashboard route edits are durable only when the database and boot YAML
//! agree; partial form saves must retain policy blocks that the form omits.

use std::collections::BTreeMap;

use systemprompt_web_admin::repositories::gateway_routes::editor::{
    create_route_entry, delete_route_at, reorder_route_positions, update_route_at,
};
use systemprompt_web_admin::repositories::gateway_routes::rows::list_gateway_routes;
use systemprompt_web_admin::types::GatewayRouteView;

use crate::tempdb::TempDb;

fn route(id: &str, pattern: &str) -> GatewayRouteView {
    GatewayRouteView {
        id: id.into(),
        name: Some(format!("Route {id}")),
        description: Some("initial description".into()),
        model_pattern: pattern.into(),
        provider: "anthropic".into(),
        upstream_model: Some("claude-fixture".into()),
        extra_headers: BTreeMap::from([("x-tenant".into(), "acme".into())]),
        pricing: Some(serde_yaml::from_str("input: 3").expect("pricing yaml")),
        when: Some(serde_yaml::from_str("region: eu").expect("when yaml")),
        requires: Some(serde_yaml::from_str("tier: enterprise").expect("requires yaml")),
        fallback_provider: Some("openai".into()),
        fallback_upstream_model: Some("gpt-fixture".into()),
    }
}

#[tokio::test]
async fn route_editor_preserves_hidden_policy_blocks_and_regenerates_dispatch_order() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    // The extension seed is not part of this editor scenario; a clean route
    // list makes the positional API assertions describe the rows we own.
    sqlx::query("DELETE FROM gateway_routes")
        .execute(&*db.pool)
        .await
        .expect("clear seeded routes");
    let dir = tempfile::tempdir().expect("gateway dir");
    let gateway = dir.path().join("gateway.yaml");
    std::fs::write(&gateway, "gateway:\n  enabled: true\n  routes: []\n").expect("gateway fixture");

    assert_eq!(
        create_route_entry(&db.pool, &gateway, &route("primary", "claude-*"))
            .await
            .expect("create primary"),
        0
    );
    let mut partial = route("", "claude-updated-*");
    partial.description = Some("edited from dashboard".into());
    partial.pricing = None;
    partial.when = None;
    partial.requires = None;
    partial.fallback_provider = None;
    partial.fallback_upstream_model = None;
    partial.extra_headers.clear();
    update_route_at(&db.pool, &gateway, 0, &partial)
        .await
        .expect("save partial form route");

    create_route_entry(&db.pool, &gateway, &route("secondary", "gpt-*"))
        .await
        .expect("create secondary");
    reorder_route_positions(&db.pool, &gateway, &[1, 0])
        .await
        .expect("reorder routes");
    let rows = list_gateway_routes(&db.pool).await.expect("read routes");
    assert_eq!(
        rows.iter().map(|r| r.route.id.as_str()).collect::<Vec<_>>(),
        ["secondary", "primary"]
    );
    let primary = rows
        .iter()
        .find(|r| r.route.id == "primary")
        .expect("primary row");
    assert_eq!(
        primary.route.description.as_deref(),
        Some("edited from dashboard")
    );
    assert_eq!(primary.route.pricing, route("x", "x").pricing);
    assert_eq!(primary.route.when, route("x", "x").when);
    assert_eq!(primary.route.requires, route("x", "x").requires);
    assert_eq!(primary.route.fallback_provider.as_deref(), Some("openai"));
    assert_eq!(primary.route.extra_headers["x-tenant"], "acme");
    let rendered = std::fs::read_to_string(&gateway).expect("regenerated gateway");
    assert!(
        rendered.find("secondary").expect("secondary yaml")
            < rendered.find("primary").expect("primary yaml")
    );

    delete_route_at(&db.pool, &gateway, 0)
        .await
        .expect("delete first route");
    let remaining = list_gateway_routes(&db.pool)
        .await
        .expect("read remaining route");
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].position, 0);
    assert_eq!(remaining[0].route.id, "primary");
    db.cleanup().await;
}
