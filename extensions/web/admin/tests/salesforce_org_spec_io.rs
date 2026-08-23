//! `salesforce_org::spec` — reading and writing `org.yaml`.
//!
//! The tool writes this file as well as reading it: `export` renders a spec it
//! just read from an org. A round-trip that loses a field would hand the
//! operator a document that no longer describes their org, and the next apply
//! would deploy the loss.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    clippy::separated_literal_suffix,
    reason = "test code: panics are the assertion mechanism"
)]

use std::path::Path;

use systemprompt_web_admin::salesforce_org::spec::{OrgSpec, SpecError};

const MINIMAL: &str = r"external_client_app:
  developer_name: Systemprompt_SSO
  label: Systemprompt SSO
  contact_email: ed@systemprompt.io
  distribution_state: Local
  oauth:
    callback_url: https://example.test/callback
    scopes:
      - api
      - mcp
  policies:
    permitted_users: AdminApprovedPreAuthorized
    ip_relaxation: Enforce
    refresh_token_policy: SpecificLifetime
";

const FULL: &str = r"external_client_app:
  developer_name: Systemprompt_SSO
  label: Systemprompt SSO
  description: Systemprompt <=> Astound
  contact_email: ed@systemprompt.io
  distribution_state: Local
  oauth:
    callback_url: https://example.test/callback
    scopes:
      - basic
      - refresh_token
      - open_id
      - mcp
    first_party_app_enabled: true
    pkce_required: false
    consumer_secret_optional: true
    named_user_jwt: false
    single_logout_url: https://example.test/logout
  policies:
    permitted_users: AdminApprovedPreAuthorized
    ip_relaxation: Bypass_2factor
    refresh_token_policy: SpecificLifetime
    refresh_token_validity:
      period: 365
      unit: Days
    required_session_level: STANDARD
permission_sets:
  - name: Salesforce_MCP_Access
    label: Salesforce MCP Access
    description: Grants the app
    grants_app: Systemprompt_SSO
hosted_mcp_servers:
  - name: sobject-all
    developer_name: platform_sobject_all
    endpoint: https://api.salesforce.com/platform/mcp/v1/platform/sobject-all
";

fn parse(yaml: &str) -> OrgSpec {
    serde_yaml::from_str(yaml).unwrap_or_else(|e| panic!("spec should parse: {e}"))
}

fn write_spec(dir: &tempfile::TempDir, yaml: &str) -> std::path::PathBuf {
    let path = dir.path().join("org.yaml");
    std::fs::write(&path, yaml).expect("temp write");
    path
}

#[test]
fn a_full_spec_round_trips_through_yaml() {
    let spec = parse(FULL);
    let yaml = spec.to_yaml().expect("serialises");
    assert_eq!(parse(&yaml), spec);
}

#[test]
fn a_minimal_spec_round_trips_through_yaml() {
    let spec = parse(MINIMAL);
    let yaml = spec.to_yaml().expect("serialises");
    assert_eq!(parse(&yaml), spec);
}

// Absent optionals must stay absent. Emitting `description: null` would fail
// the next parse of a file this tool wrote itself.
#[test]
fn absent_optionals_are_not_emitted() {
    let yaml = parse(MINIMAL).to_yaml().expect("serialises");
    assert!(!yaml.contains("description"), "{yaml}");
    assert!(!yaml.contains("single_logout_url"), "{yaml}");
    assert!(!yaml.contains("refresh_token_validity"), "{yaml}");
    assert!(!yaml.contains("required_session_level"), "{yaml}");
}

#[test]
fn defaulted_booleans_are_emitted_explicitly() {
    let yaml = parse(MINIMAL).to_yaml().expect("serialises");
    assert!(yaml.contains("pkce_required: true"), "{yaml}");
    assert!(yaml.contains("named_user_jwt: true"), "{yaml}");
    assert!(yaml.contains("consumer_secret_optional: false"), "{yaml}");
}

#[test]
fn loading_reads_a_spec_from_disk() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = write_spec(&dir, FULL);
    assert_eq!(OrgSpec::load(&path).expect("loads"), parse(FULL));
}

#[test]
fn loading_a_missing_path_names_it() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("absent.yaml");
    match OrgSpec::load(&path) {
        Err(SpecError::NotFound(reported)) => assert_eq!(reported, path),
        other => panic!("expected NotFound, got {other:?}"),
    }
}

#[test]
fn loading_malformed_yaml_reports_a_parse_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = write_spec(&dir, "external_client_app: [not, a, map]\n");
    match OrgSpec::load(&path) {
        Err(SpecError::Parse { path: reported, .. }) => assert_eq!(reported, path),
        other => panic!("expected Parse, got {other:?}"),
    }
}

#[test]
fn loading_an_unreadable_directory_is_not_a_silent_default() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert!(OrgSpec::load(dir.path()).is_err());
}

#[test]
fn spec_errors_name_the_file_in_their_message() {
    let path = Path::new("/nowhere/org.yaml");
    let message = SpecError::NotFound(path.to_path_buf()).to_string();
    assert!(message.contains("/nowhere/org.yaml"), "{message}");
}
