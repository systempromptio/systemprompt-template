//! Export → import is a no-op: a plane's database rendered as its file,
//! parsed back as an uploaded declaration, drifts by nothing against the
//! rows it was rendered from.

use chrono::Utc;
use systemprompt_web_admin::repositories::gateway_policies::declared::parse_declared_policies;
use systemprompt_web_admin::repositories::gateway_policies::drift::compute_policy_drift;
use systemprompt_web_admin::repositories::gateway_policies::export::render_policies_export;
use systemprompt_web_admin::repositories::gateway_policies::rows::PolicyRow;
use systemprompt_web_admin::repositories::sync::groups_db::{
    parse_groups_doc, render_groups_export,
};
use systemprompt_web_admin::repositories::sync::groups_drift::{
    MappingRow, MemberSetRow, compute_groups_drift,
};
use systemprompt_web_admin::repositories::sync::plane::DeclarationSource;

const POLICIES: &str = r"
policies:
  - name: team-quotas
    enabled: true
    priority: 10
    spec:
      quota_mode: enforce
      quota_windows:
        - subject: user
          window_seconds: 86400
          max_requests: 200
      safety:
        mode: warn
        scanners: [secrets]
";

#[test]
fn gateway_policies_export_then_import_is_clean() {
    let declared = parse_declared_policies(POLICIES).expect("parses");
    let rows: Vec<PolicyRow> = declared
        .entries
        .iter()
        .map(|e| PolicyRow {
            name: e.name.clone(),
            spec: e.spec.clone(),
            enabled: e.enabled,
            priority: e.priority,
            updated_at: Utc::now(),
        })
        .collect();
    let exported = render_policies_export(&rows, Utc::now());
    let reimported = parse_declared_policies(&exported).expect("export parses as a declaration");
    let drift = compute_policy_drift(&reimported, &rows);
    assert!(drift.is_clean(), "{drift:?}");
    assert_eq!(reimported.declared_hash(), declared.declared_hash());
}

#[test]
fn groups_export_then_import_is_clean() {
    let sets = vec![
        MemberSetRow {
            kind: "group".to_owned(),
            id: "europe-devs".to_owned(),
            name: "Europe developers".to_owned(),
            description: Some("Commerce Cloud developers.".to_owned()),
            source: "yaml".to_owned(),
            is_system: false,
        },
        MemberSetRow {
            kind: "project".to_owned(),
            id: "commerce".to_owned(),
            name: "Commerce".to_owned(),
            description: None,
            source: "dashboard".to_owned(),
            is_system: false,
        },
    ];
    let mappings = vec![MappingRow {
        kind: "group".to_owned(),
        ad_group: "Systemprompt-Commerce".to_owned(),
        set_id: "europe-devs".to_owned(),
        source: "yaml".to_owned(),
    }];
    let exported = render_groups_export(&sets, &mappings);
    let doc = parse_groups_doc(&exported).expect("export parses as a declaration");
    let drift = compute_groups_drift(&doc, &sets, &mappings);
    assert!(drift.is_clean(), "{:?}", drift.rows);
}

#[test]
fn the_default_declaration_source_is_disk() {
    assert!(DeclarationSource::default().is_disk());
    assert!(!DeclarationSource::Text("x").is_disk());
}
