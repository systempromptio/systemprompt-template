//! Database → `rules.yaml`: the export is the loader's inverse.

use systemprompt_security::authz::Access;
use systemprompt_web_admin::repositories::access_control::drift::{BandRuleRow, EntityDefaultRow};
use systemprompt_web_admin::repositories::access_control::export::{Owners, render_export};
use systemprompt_web_admin::repositories::config::rules_yaml_loader::parse_rules_doc;
use systemprompt_web_admin::repositories::config::rules_yaml_types::{BandSpec, EntityDefault};

#[expect(
    clippy::too_many_arguments,
    reason = "a test row is spelled out in full"
)]
fn row(
    entity_type: &str,
    entity_id: &str,
    rule_type: &str,
    value: &str,
    access: Access,
    why: Option<&str>,
) -> BandRuleRow {
    BandRuleRow {
        id: format!("{entity_type}-{entity_id}-{rule_type}-{value}"),
        entity_type: entity_type.to_owned(),
        entity_id: entity_id.to_owned(),
        rule_type: rule_type.to_owned(),
        rule_value: value.to_owned(),
        access,
        justification: why.map(str::to_owned),
        source: "yaml".to_owned(),
        valid_until: None,
    }
}

fn entity(entity_type: &str, entity_id: &str, open: bool) -> EntityDefaultRow {
    EntityDefaultRow {
        entity_type: entity_type.to_owned(),
        entity_id: entity_id.to_owned(),
        default_included: open,
        source: "yaml".to_owned(),
    }
}

#[test]
fn rows_group_back_into_one_entity_block_that_the_loader_accepts() {
    let yaml = render_export(
        &[
            row(
                "mcp_server",
                "atlassian",
                "role",
                "admin",
                Access::Allow,
                Some("pilot"),
            ),
            row(
                "mcp_server",
                "atlassian",
                "group",
                "uk",
                Access::Allow,
                Some("pilot"),
            ),
            row(
                "mcp_server",
                "atlassian",
                "group",
                "india-devs",
                Access::Allow,
                Some("pilot"),
            ),
            row(
                "mcp_server",
                "atlassian",
                "project",
                "storefront",
                Access::Deny,
                Some("pilot"),
            ),
            row(
                "mcp_server",
                "atlassian",
                "user",
                "someone",
                Access::Deny,
                None,
            ),
        ],
        &[entity("mcp_server", "atlassian", false)],
        &Owners::default(),
    );
    let doc = parse_rules_doc(&yaml).expect("export is valid rules.yaml");
    assert_eq!(doc.entities.len(), 1);
    let decl = &doc.entities[0];
    assert_eq!(decl.entity, "mcp_server/atlassian");
    assert_eq!(decl.default, EntityDefault::Closed);
    assert_eq!(decl.why, "pilot");
    assert_eq!(
        decl.allow.group.as_ref().map(BandSpec::values),
        Some(&["india-devs".to_owned(), "uk".to_owned()][..])
    );
    assert_eq!(
        decl.deny.project.as_ref().map(BandSpec::values),
        Some(&["storefront".to_owned()][..])
    );
    assert!(!yaml.contains("someone"), "the user band is never exported");
}

#[test]
fn a_band_with_its_own_uniform_reason_keeps_it() {
    let yaml = render_export(
        &[
            row(
                "marketplace",
                "commons",
                "role",
                "user",
                Access::Allow,
                Some("everyone"),
            ),
            row(
                "marketplace",
                "commons",
                "group",
                "uk",
                Access::Allow,
                Some("everyone"),
            ),
            row(
                "marketplace",
                "commons",
                "group",
                "india-devs",
                Access::Allow,
                Some("everyone"),
            ),
            row(
                "marketplace",
                "commons",
                "project",
                "storefront",
                Access::Allow,
                Some("the storefront build"),
            ),
        ],
        &[entity("marketplace", "commons", true)],
        &Owners::default(),
    );
    let doc = parse_rules_doc(&yaml).expect("valid");
    let decl = &doc.entities[0];
    assert_eq!(decl.default, EntityDefault::Open);
    assert_eq!(decl.why, "everyone");
    assert_eq!(
        decl.allow.project.as_ref().and_then(BandSpec::why),
        Some("the storefront build")
    );
    assert_eq!(decl.allow.group.as_ref().and_then(BandSpec::why), None);
}

#[test]
fn identical_gateway_routes_collapse_to_the_glob_and_exceptions_are_noted() {
    let yaml = render_export(
        &[
            row(
                "gateway_route",
                "a",
                "role",
                "user",
                Access::Allow,
                Some("routes"),
            ),
            row(
                "gateway_route",
                "b",
                "role",
                "user",
                Access::Allow,
                Some("routes"),
            ),
            row(
                "gateway_route",
                "c",
                "role",
                "admin",
                Access::Allow,
                Some("routes"),
            ),
        ],
        &[
            entity("gateway_route", "a", true),
            entity("gateway_route", "b", true),
            entity("gateway_route", "c", true),
        ],
        &Owners::default(),
    );
    let doc = parse_rules_doc(&yaml).expect("valid");
    assert_eq!(doc.entities.len(), 1);
    assert_eq!(doc.entities[0].entity, "gateway_route/*");
    assert_eq!(
        doc.entities[0].allow.role.as_ref().map(BandSpec::values),
        Some(&["user".to_owned()][..])
    );
    assert!(yaml.contains("# NOTE: gateway_route/*: c differ"), "{yaml}");
}

#[test]
fn a_row_without_any_reason_is_flagged_for_the_operator() {
    let yaml = render_export(
        &[row("skill", "x", "role", "admin", Access::Allow, None)],
        &[entity("skill", "x", false)],
        &Owners::default(),
    );
    assert!(
        yaml.contains("# NOTE: skill/x: no rule carried a justification"),
        "{yaml}"
    );
    assert!(
        parse_rules_doc(&yaml).is_ok(),
        "still a loadable file, with a TODO why"
    );
}
