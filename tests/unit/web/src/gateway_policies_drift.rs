//! `policies.yaml` against `ai_gateway_policies`: the drift is blind to the
//! daily month-window rewrite and sees every console edit; the export is
//! the loader's inverse and always declares the sentinel.

use chrono::{NaiveDate, Utc};
use systemprompt::ai::{GatewayPolicyConfig, QuotaMode, SafetyMode};
use systemprompt_web_admin::repositories::gateway_policies::declared::parse_declared_policies;
use systemprompt_web_admin::repositories::gateway_policies::drift::compute_policy_drift;
use systemprompt_web_admin::repositories::gateway_policies::export::render_policies_export;
use systemprompt_web_admin::repositories::gateway_policies::month_window::{
    MONTH_WINDOW_SECONDS, month_window_seconds,
};
use systemprompt_web_admin::repositories::gateway_policies::rows::{PolicyRow, effective_spec};

const DECLARED: &str = r"
policies:
  - name: default-quotas
    enabled: true
    spec:
      quota_mode: warn
      quota_windows:
        - subject: user
          window_seconds: 3600
          max_requests: 600
        - subject: organization
          window_seconds: 2678400
          max_cost_microdollars: 200000000
      safety:
        mode: warn
        scanners: [heuristic, secrets]
        block_categories: [pii_ssn]
        block_response_categories: [secret, pii_ssn]
        heuristic:
          phrases: [ignore previous instructions]
";

fn row_from_declared(name: &str) -> PolicyRow {
    let declared = parse_declared_policies(DECLARED).expect("declaration parses");
    let entry = declared.find(name).expect("named policy");
    PolicyRow {
        name: entry.name.clone(),
        spec: entry.spec.clone(),
        enabled: entry.enabled,
        priority: entry.priority,
        updated_at: Utc::now(),
    }
}

#[test]
fn the_declared_hash_ignores_formatting_and_notices_a_ceiling() {
    let a = parse_declared_policies(DECLARED).expect("parses");
    let b = parse_declared_policies(&DECLARED.replace("max_requests: 600", "max_requests:   600"))
        .expect("parses");
    let c = parse_declared_policies(&DECLARED.replace("max_requests: 600", "max_requests: 601"))
        .expect("parses");
    assert_eq!(a.declared_hash(), b.declared_hash());
    assert_ne!(a.declared_hash(), c.declared_hash());
}

#[test]
fn a_row_the_daily_job_rewrote_is_not_drift() {
    let declared = parse_declared_policies(DECLARED).expect("parses");
    let mut row = row_from_declared("default-quotas");
    let today = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap_or_default();
    row.spec.quota_windows[1].window_seconds = month_window_seconds(today);
    let drift = compute_policy_drift(&declared, &[row]);
    assert!(drift.is_clean(), "{drift:?}");
}

#[test]
fn a_console_edit_names_the_field_that_moved() {
    let declared = parse_declared_policies(DECLARED).expect("parses");
    let mut row = row_from_declared("default-quotas");
    row.spec.quota_windows[0].max_requests = Some(1_000);
    row.spec.safety.mode = SafetyMode::Enforce;
    let drift = compute_policy_drift(&declared, &[row]);
    assert_eq!(drift.changed.len(), 1);
    assert_eq!(
        drift.changed[0].fields,
        vec!["quota_windows".to_owned(), "safety.mode".to_owned()]
    );
    assert!(drift.changed[0].in_db.contains("enforce"));
    assert!(drift.changed[0].in_code.contains("warn"));
}

#[test]
fn compute_policy_drift_with_every_scalar_edit_returns_fields_in_declaration_order() {
    let declared = parse_declared_policies(DECLARED).expect("parses");
    let mut row = row_from_declared("default-quotas");
    row.enabled = !row.enabled;
    row.priority += 1;
    row.spec.quota_mode = QuotaMode::Enforce;
    row.spec.safety.scanners.pop();
    row.spec.safety.block_categories.clear();
    row.spec.safety.block_response_categories.clear();
    let drift = compute_policy_drift(&declared, &[row]);
    assert_eq!(
        drift.changed[0].fields,
        vec![
            "enabled".to_owned(),
            "priority".to_owned(),
            "quota_mode".to_owned(),
            "safety.scanners".to_owned(),
            "safety.block_categories".to_owned(),
            "safety.block_response_categories".to_owned(),
        ]
    );
    assert!(drift.missing_in_db.is_empty());
    assert!(drift.only_in_db.is_empty());
}

#[test]
fn missing_and_orphan_policies_are_reported_by_name() {
    let declared = parse_declared_policies(DECLARED).expect("parses");
    let mut orphan = row_from_declared("default-quotas");
    orphan.name = "team-quotas".to_owned();
    let drift = compute_policy_drift(&declared, &[orphan]);
    assert_eq!(drift.missing_in_db, vec!["default-quotas".to_owned()]);
    assert_eq!(drift.only_in_db, vec!["team-quotas".to_owned()]);
    assert!(!drift.is_clean());
}

#[test]
fn the_export_declares_the_sentinel_and_the_loader_accepts_it() {
    let mut row = row_from_declared("default-quotas");
    let today = NaiveDate::from_ymd_opt(2026, 9, 17).unwrap_or_default();
    row.spec.quota_windows[1].window_seconds = month_window_seconds(today);
    let yaml = render_policies_export(&[row], Utc::now());
    assert!(yaml.starts_with("# Gateway policy baseline."));
    assert!(
        yaml.contains("governance_bootstrap"),
        "the header names the real loader"
    );
    let cfg: GatewayPolicyConfig =
        serde_yaml::from_str(&yaml).expect("the export parses as the file");
    cfg.validate().expect("the export validates as the file");
    let month = &cfg.policies[0].spec.quota_windows[1];
    assert_eq!(month.window_seconds, MONTH_WINDOW_SECONDS);
    assert_eq!(month.subject, "organization");
    assert_eq!(cfg.policies[0].spec.quota_mode, QuotaMode::Warn);
}

// Why: core's merge is a section-wise override in `(priority, name)` order;
// the quota page has to show the same windows the gateway enforces, so the
// reproduction is pinned against a two-row table.
#[test]
fn the_effective_spec_lets_the_highest_priority_row_win_each_section() {
    let base = row_from_declared("default-quotas");
    let mut team = row_from_declared("default-quotas");
    team.name = "team-quotas".to_owned();
    team.priority = 10;
    team.spec.quota_windows.truncate(1);
    team.spec.quota_windows[0].max_requests = Some(50);
    team.spec.safety.scanners.clear();
    team.spec.safety.block_categories.clear();
    team.spec.safety.block_response_categories.clear();
    team.spec.safety.mode = SafetyMode::Enforce;

    let merged = effective_spec(&[base.clone(), team.clone()]);
    assert_eq!(
        merged.quota_windows.len(),
        1,
        "windows replace, never union"
    );
    assert_eq!(merged.quota_windows[0].max_requests, Some(50));
    assert_eq!(
        merged.safety.scanners,
        vec!["heuristic".to_owned(), "secrets".to_owned()],
        "an empty safety section does not override the one below it"
    );

    let mut disabled = team;
    disabled.enabled = false;
    let merged = effective_spec(&[base, disabled]);
    assert_eq!(
        merged.quota_windows.len(),
        2,
        "a disabled row contributes nothing"
    );
}

#[test]
fn the_committed_file_declares_through_the_same_parser() {
    let path = crate::support::repo_root().join("services/gateway/policies.yaml");
    let yaml = std::fs::read_to_string(&path).expect("policies.yaml is readable");
    let declared = parse_declared_policies(&yaml).expect("the committed file parses");
    assert!(declared.find("default-quotas").is_some());
}
