//! `governance/config.yaml` against the staged chain: parsed by core's own
//! parser, hashed on meaning, drift on switch/mode/params/order, and an
//! export the parser accepts back.

use chrono::Utc;
use systemprompt_web_admin::repositories::governance_chain::declared::{
    DeclaredChain, parse_declared_chain,
};
use systemprompt_web_admin::repositories::governance_chain::export::{
    compute_chain_drift, render_chain_export,
};

const DECLARED: &str = r"
# chain header
governance:
  mode: warn
  policies:
  - id: scope_check
    enabled: true
    admin_only_prefixes: [mcp__systemprompt__]
  - id: tool_blocklist
    enabled: true
    mode: enforce
    patterns: [delete, drop]
  - id: rate_limit
    enabled: false
    requests_per_window: 300
    window_secs: 60
";

#[test]
fn parse_uses_core_semantics_for_inherited_mode_and_switch() {
    let chain = parse_declared_chain(DECLARED).expect("parses");
    assert!(chain.enabled);
    assert_eq!(chain.mode, "warn");
    assert_eq!(chain.policies.len(), 3);
    assert_eq!(chain.policies[0].mode, "warn", "inherits the chain mode");
    assert_eq!(chain.policies[1].mode, "enforce", "overrides it");
    assert!(!chain.policies[2].enabled);
}

#[test]
fn parse_refuses_what_core_refuses() {
    let bad_mode = DECLARED.replace("mode: warn", "mode: warnn");
    assert!(
        parse_declared_chain(&bad_mode)
            .expect_err("refused")
            .contains("warnn")
    );
    assert!(
        parse_declared_chain("governance: {}").is_err(),
        "no policies sequence"
    );
}

#[test]
fn hash_ignores_comments_and_key_order_but_sees_params_and_order() {
    let a = parse_declared_chain(DECLARED).expect("parses");
    let b = parse_declared_chain(&DECLARED.replace("# chain header\n", "")).expect("parses");
    assert_eq!(a.declared_hash(), b.declared_hash());
    let swapped_keys = DECLARED.replace(
        "    requests_per_window: 300\n    window_secs: 60\n",
        "    window_secs: 60\n    requests_per_window: 300\n",
    );
    assert_eq!(
        a.declared_hash(),
        parse_declared_chain(&swapped_keys)
            .expect("parses")
            .declared_hash()
    );
    let tuned = DECLARED.replace("window_secs: 60", "window_secs: 30");
    assert_ne!(
        a.declared_hash(),
        parse_declared_chain(&tuned)
            .expect("parses")
            .declared_hash()
    );
    let mut reordered = a.clone();
    reordered.policies.swap(0, 1);
    assert_ne!(a.declared_hash(), reordered.declared_hash());
}

#[test]
fn drift_is_clean_for_the_same_chain() {
    let a = parse_declared_chain(DECLARED).expect("parses");
    assert!(compute_chain_drift(&a, &a).is_clean());
}

#[test]
fn drift_names_settings_missing_orphan_changed_and_order() {
    let declared = parse_declared_chain(DECLARED).expect("parses");
    let staged = parse_declared_chain(
        r"
governance:
  enabled: false
  mode: warn
  policies:
  - id: tool_blocklist
    enabled: true
    mode: warn
    patterns: [delete, drop]
  - id: scope_check
    enabled: true
    admin_only_prefixes: [mcp__systemprompt__]
  - id: secret_scan
    enabled: true
    patterns: []
",
    )
    .expect("parses");
    let drift = compute_chain_drift(&declared, &staged);
    assert_eq!(drift.settings_changed, vec!["enabled".to_owned()]);
    assert_eq!(drift.missing_in_db, vec!["rate_limit".to_owned()]);
    assert_eq!(drift.only_in_db, vec!["secret_scan".to_owned()]);
    assert_eq!(drift.changed.len(), 1);
    assert_eq!(drift.changed[0].id, "tool_blocklist");
    assert_eq!(drift.changed[0].fields, vec!["mode".to_owned()]);
    let (code, db) = drift.reordered.clone().expect("order differs");
    assert_eq!(
        code,
        vec!["scope_check".to_owned(), "tool_blocklist".to_owned()]
    );
    assert_eq!(
        db,
        vec!["tool_blocklist".to_owned(), "scope_check".to_owned()]
    );
    assert_eq!(drift.total(), 5);
}

#[test]
fn export_round_trips_through_the_parser_with_the_same_hash() {
    let chain = parse_declared_chain(DECLARED).expect("parses");
    let out = render_chain_export(&chain, Utc::now());
    assert!(out.starts_with("# Governance policy chain."));
    let back = parse_declared_chain(&out).expect("core accepts the export");
    assert_eq!(back.declared_hash(), chain.declared_hash());
}

#[test]
fn export_writes_the_row_switch_and_mode_over_the_stored_entry() {
    let mut chain = parse_declared_chain(DECLARED).expect("parses");
    chain.policies[0].enabled = false;
    chain.policies[0].mode = "enforce".to_owned();
    let out = render_chain_export(&chain, Utc::now());
    let back = parse_declared_chain(&out).expect("parses");
    assert!(!back.policies[0].enabled);
    assert_eq!(back.policies[0].mode, "enforce");
    assert_ne!(
        back.declared_hash(),
        DeclaredChain::default().declared_hash()
    );
}
