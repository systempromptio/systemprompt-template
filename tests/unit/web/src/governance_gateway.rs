//! Gateway editor regression: reads and writes preserve operator-authored
//! configuration.

use systemprompt_web_admin::repositories::config::gateway::{
    create_route, get_gateway_config_from_file,
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
fn dashboard_route_reads_and_edits_preserve_operator_authored_ids()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("gateway.yaml");
    let original = format!("# Operator-owned gateway routes\n{TWO_ROUTE_PROFILE}");
    std::fs::write(&path, &original)?;
    let loaded = get_gateway_config_from_file(&path)?;
    assert_eq!(
        std::fs::read_to_string(&path)?,
        original,
        "reading must not rewrite YAML"
    );
    assert!(
        matches!(
            create_route(&path, &loaded.routes[0]),
            Err(MarketplaceError::BadRequest(_))
        ),
        "an implicit route id still owns its identity"
    );
    create_route(
        &path,
        &GatewayRouteView {
            id: "operator-chosen".into(),
            model_pattern: "new-model".into(),
            provider: "anthropic".into(),
            ..Default::default()
        },
    )?;
    let written = std::fs::read_to_string(&path)?;
    assert!(written.starts_with("# Operator-owned gateway routes\n"));
    let yaml: serde_yaml::Value = serde_yaml::from_str(&written)?;
    let routes = yaml["gateway"]["routes"]
        .as_sequence()
        .expect("routes array");
    assert!(routes[0].get("id").is_none());
    assert!(routes[1].get("id").is_none());
    assert_eq!(routes[2]["id"].as_str(), Some("operator-chosen"));
    Ok(())
}
