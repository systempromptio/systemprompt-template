//! Boot-time governance check. `GovernanceConfig::load` is deliberately
//! infallible on the request path, so this is the only place a typo in the
//! policy chain can still be refused — an unparseable file must fail the job,
//! not degrade to the built-in defaults. The `active` count it reports is
//! what the "governance is not enforcing" warning keys on, so both the master
//! switch and the per-policy flags have to be honoured.

use std::path::Path;

use systemprompt_web_extension::jobs::internals::check_governance_config;
use tempfile::TempDir;

fn services_with(config: Option<&str>) -> TempDir {
    let dir = TempDir::new().unwrap();
    let governance = dir.path().join("governance");
    std::fs::create_dir_all(&governance).unwrap();
    if let Some(yaml) = config {
        std::fs::write(governance.join("config.yaml"), yaml).unwrap();
    }
    dir
}

fn active(dir: &Path) -> usize {
    check_governance_config(dir).unwrap().active
}

#[test]
fn an_absent_file_falls_back_to_the_four_built_in_policies() {
    let dir = services_with(None);
    assert_eq!(active(dir.path()), 4);
}

#[test]
fn every_enabled_policy_is_counted() {
    let dir = services_with(Some(
        "governance:\n  enabled: true\n  policies:\n    - id: secret_scan\n    - id: \
         rate_limit\n",
    ));
    assert_eq!(active(dir.path()), 2);
}

#[test]
fn a_disabled_policy_is_not_counted() {
    let dir = services_with(Some(
        "governance:\n  enabled: true\n  policies:\n    - id: secret_scan\n      enabled: \
         false\n    - id: rate_limit\n",
    ));
    assert_eq!(active(dir.path()), 1);
}

#[test]
fn the_master_switch_zeroes_the_count_whatever_the_policies_say() {
    let dir = services_with(Some(
        "governance:\n  enabled: false\n  policies:\n    - id: secret_scan\n    - id: \
         rate_limit\n",
    ));
    assert_eq!(active(dir.path()), 0);
}

#[test]
fn a_file_without_a_policy_list_fails_the_job() {
    let dir = services_with(Some("governance:\n  enabled: true\n"));
    let error = check_governance_config(dir.path()).unwrap_err().to_string();
    assert!(error.contains("governance/config.yaml"));
}

#[test]
fn unparseable_yaml_fails_the_job() {
    let dir = services_with(Some("governance:\n  policies: [\n"));
    assert!(check_governance_config(dir.path()).is_err());
}
