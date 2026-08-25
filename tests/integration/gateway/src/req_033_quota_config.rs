//! REQ-033 Per-Consumer LLM Rate Limiting — "Rate limits can be applied per
//! agreed consumer boundary so one user/team/agent/application cannot exhaust
//! shared provider capacity."
//!
//! Honest config-level evidence: the checked-in gateway policy baseline
//! (`services/gateway/policies.yaml`, ingested into `ai_gateway_policies` at
//! boot) declares an enabled policy whose `quota_windows` bound both a
//! per-user hourly window and a per-organization daily cost window with
//! positive limits. Enforcement under load is the register's separate load
//! test; this pins that the deployed configuration actually carries the
//! boundaries.

use std::path::{Path, PathBuf};

fn policies_yaml() -> serde_yaml::Value {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("crate sits three levels below the repository root")
        .join("services/gateway/policies.yaml");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_yaml::from_str(&raw).expect("policies.yaml is valid YAML")
}

fn quota_windows(doc: &serde_yaml::Value) -> &serde_yaml::Value {
    let policy = doc
        .get("policies")
        .and_then(|p| p.get(0usize))
        .expect("policies.yaml declares at least one policy");
    assert_eq!(
        policy.get("enabled").and_then(serde_yaml::Value::as_bool),
        Some(true),
        "the baseline policy is enabled"
    );
    policy
        .get("spec")
        .and_then(|s| s.get("quota_windows"))
        .expect("the policy spec carries quota_windows")
}

fn window_for<'a>(windows: &'a serde_yaml::Value, subject: &str) -> &'a serde_yaml::Value {
    windows
        .as_sequence()
        .expect("quota_windows is a list")
        .iter()
        .find(|w| w.get("subject").and_then(serde_yaml::Value::as_str) == Some(subject))
        .unwrap_or_else(|| panic!("a quota window for subject '{subject}' exists"))
}

#[test]
fn a_per_user_window_bounds_requests_and_cost() {
    let doc = policies_yaml();
    let windows = quota_windows(&doc);
    let user = window_for(windows, "user");

    let seconds = user
        .get("window_seconds")
        .and_then(serde_yaml::Value::as_i64)
        .expect("the user window declares window_seconds");
    assert!(seconds > 0, "the user window is a real time window");
    let max_requests = user
        .get("max_requests")
        .and_then(serde_yaml::Value::as_i64)
        .expect("the user window declares max_requests");
    assert!(max_requests > 0, "the request ceiling is positive");
    let max_cost = user
        .get("max_cost_microdollars")
        .and_then(serde_yaml::Value::as_i64)
        .expect("the user window declares max_cost_microdollars");
    assert!(max_cost > 0, "the cost ceiling is positive");
}

#[test]
fn a_per_organization_window_caps_daily_spend() {
    let doc = policies_yaml();
    let windows = quota_windows(&doc);
    let org = window_for(windows, "organization");

    let seconds = org
        .get("window_seconds")
        .and_then(serde_yaml::Value::as_i64)
        .expect("the organization window declares window_seconds");
    assert!(
        seconds >= 86_400,
        "the organization boundary is at least a daily window, got {seconds}s"
    );
    let max_cost = org
        .get("max_cost_microdollars")
        .and_then(serde_yaml::Value::as_i64)
        .expect("the organization window declares max_cost_microdollars");
    assert!(max_cost > 0, "the organization cost ceiling is positive");
}

#[test]
fn the_two_consumer_boundaries_are_distinct_subjects() {
    let doc = policies_yaml();
    let windows = quota_windows(&doc);
    let subjects: Vec<&str> = windows
        .as_sequence()
        .expect("quota_windows is a list")
        .iter()
        .filter_map(|w| w.get("subject").and_then(serde_yaml::Value::as_str))
        .collect();

    assert!(subjects.contains(&"user"), "a per-user boundary exists");
    assert!(
        subjects.contains(&"organization"),
        "a per-organization boundary exists"
    );
}
