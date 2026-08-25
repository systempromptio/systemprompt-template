//! The pure halves of the FinOps delivery jobs: the digest composer, the
//! anomaly threshold, and the deprovisioning SOQL builder.
//!
//! The I/O halves (Slack posts, Salesforce calls, the queries) are covered by
//! the integration and e2e tiers; what belongs here is the arithmetic and the
//! escaping — the parts where a wrong answer is silent.

use chrono::{TimeZone, Utc};
use systemprompt::identifiers::UserId;
use systemprompt_web_admin::repositories::users::salesforce_identity::LinkedSalesforceIdentity;
use systemprompt_web_extension::jobs::internals::{
    Finding, OrgDigestRow, build_active_users_soql, compose_digest, compose_org_line, evaluate,
};

fn row(name: &str, cap: Option<i64>, mtd: i64) -> OrgDigestRow {
    OrgDigestRow {
        name: name.to_owned(),
        cap_microdollars: cap,
        mtd_microdollars: mtd,
        week_microdollars: 3_000_000,
        week_requests: 42,
    }
}

#[test]
fn digest_line_states_cap_utilization() {
    let now = Utc.with_ymd_and_hms(2026, 8, 16, 12, 0, 0).unwrap();
    let line = compose_org_line(&row("Acme", Some(100_000_000), 25_000_000), now);
    assert!(line.contains("*Acme*"), "line: {line}");
    assert!(line.contains("$25.00 MTD"), "line: {line}");
    assert!(line.contains("25% of $100.00 cap"), "line: {line}");
    assert!(line.contains("$3.00 over 42 requests"), "line: {line}");
    // Halfway through the month, spend doubles in the linear projection.
    assert!(line.contains("on pace for ~$50.00"), "line: {line}");
}

#[test]
fn digest_line_says_uncapped_rather_than_zero_percent() {
    let now = Utc.with_ymd_and_hms(2026, 8, 16, 12, 0, 0).unwrap();
    let line = compose_org_line(&row("Acme", None, 25_000_000), now);
    assert!(line.contains("(uncapped)"), "line: {line}");
    assert!(!line.contains('%'), "no percentage without a cap: {line}");
}

#[test]
fn digest_carries_one_line_per_organization_under_a_header() {
    let rows = vec![row("Acme", None, 1), row("Globex", None, 2)];
    let digest = compose_digest(&rows);
    let lines: Vec<&str> = digest.lines().collect();
    assert_eq!(lines.len(), 3, "header plus one line per org: {digest}");
    assert!(lines[0].contains("Weekly AI cost digest"));
}

#[test]
fn anomaly_needs_both_the_multiplier_and_the_floor() {
    // Past the multiplier but under the floor: a quiet instance stays quiet.
    assert!(evaluate("requests", 30, 5, 3, 50).is_none());
    // Past the floor but under the multiplier: ordinary growth is not a spike.
    assert!(evaluate("requests", 120, 100, 3, 50).is_none());
    // Past both: a finding, carrying what was observed against what was normal.
    let f: Finding = evaluate("requests", 300, 100, 3, 50).expect("a spike");
    assert_eq!((f.metric, f.observed, f.baseline), ("requests", 300, 100));
    // Zero baseline (a brand-new instance): the floor alone decides.
    assert!(evaluate("errors", 9, 0, 5, 10).is_none());
    assert!(evaluate("errors", 10, 0, 5, 10).is_some());
}

#[test]
fn deprovision_soql_escapes_quotes_and_backslashes() {
    let linked = vec![
        LinkedSalesforceIdentity {
            user_id: UserId::new("u1"),
            sf_username: "o'brien@example.com".to_owned(),
        },
        LinkedSalesforceIdentity {
            user_id: UserId::new("u2"),
            sf_username: "back\\slash@example.com".to_owned(),
        },
    ];
    let soql = build_active_users_soql(&linked);
    assert!(
        soql.contains(r"'o\'brien@example.com'"),
        "quote escaped: {soql}"
    );
    assert!(
        soql.contains(r"'back\\slash@example.com'"),
        "backslash escaped: {soql}"
    );
    assert!(soql.starts_with("SELECT Username FROM User WHERE IsActive = true"));
}
