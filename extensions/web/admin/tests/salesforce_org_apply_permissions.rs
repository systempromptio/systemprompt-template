//! `salesforce_org::apply::lookup` — the joins that decide what apply creates.
//!
//! Apply is additive: it creates what these lookups report as missing. A false
//! negative therefore creates a duplicate grant or a second assignment, and a
//! false positive silently skips access a user was meant to get. The SOQL
//! escaping matters for the same reason — a value that closes the literal early
//! turns a lookup into a different query.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    clippy::separated_literal_suffix,
    reason = "test code: panics are the assertion mechanism"
)]

use systemprompt_web_admin::salesforce_org::apply::lookup::{
    find_app_id, find_permission_set_id, find_user_id, grant_exists, holds_permission_set,
    soql_escape, soql_list, str_field,
};

fn apps() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({ "Id": "0Ci0000000001", "DeveloperName": "Systemprompt_SSO" }),
        serde_json::json!({ "Id": "0Ci0000000002", "DeveloperName": "Other_App" }),
    ]
}

fn permsets() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({ "Id": "0PS0000000001", "Name": "Salesforce_MCP_Access" }),
        serde_json::json!({ "Id": "0PS0000000002", "Name": "Other_Set" }),
    ]
}

fn users() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({ "Id": "005000000000001", "Username": "ed@agentforce.com" }),
        serde_json::json!({ "Id": "005000000000002", "Username": "sam@agentforce.com" }),
    ]
}

#[test]
fn a_string_field_is_read_as_written() {
    let record = serde_json::json!({ "Name": "Salesforce_MCP_Access" });
    assert_eq!(
        str_field(&record, "Name").as_deref(),
        Some("Salesforce_MCP_Access")
    );
}

#[test]
fn a_missing_or_non_string_field_reads_as_absent() {
    let record = serde_json::json!({ "Active": true, "Count": 3, "Name": null });
    assert!(str_field(&record, "Absent").is_none());
    assert!(str_field(&record, "Active").is_none());
    assert!(str_field(&record, "Count").is_none());
    assert!(str_field(&record, "Name").is_none());
}

#[test]
fn an_ordinary_value_needs_no_escaping() {
    assert_eq!(soql_escape("ed@agentforce.com"), "ed@agentforce.com");
    assert_eq!(soql_escape(""), "");
    assert_eq!(
        soql_escape("Salesforce_MCP_Access"),
        "Salesforce_MCP_Access"
    );
}

// A single quote would otherwise close the literal and let the rest of the
// value be read as SOQL.
#[test]
fn a_quote_is_escaped() {
    assert_eq!(soql_escape("o'brien@example.com"), "o\\'brien@example.com");
    assert_eq!(soql_escape("' OR Id != null--"), "\\' OR Id != null--");
}

// The backslash goes first, so an escape the value already contains is not
// mistaken for one this function added.
#[test]
fn a_backslash_is_escaped_before_the_quote() {
    assert_eq!(soql_escape("a\\b"), "a\\\\b");
    assert_eq!(soql_escape("a\\'b"), "a\\\\\\'b");
}

#[test]
fn a_list_quotes_every_value() {
    assert_eq!(soql_list(&["a"]), "'a'");
    assert_eq!(soql_list(&["a", "b", "c"]), "'a','b','c'");
}

#[test]
fn an_empty_list_renders_empty() {
    assert_eq!(soql_list(&[]), "");
}

#[test]
fn a_list_escapes_each_value() {
    assert_eq!(soql_list(&["o'brien", "plain"]), "'o\\'brien','plain'");
}

#[test]
fn an_app_is_found_by_developer_name() {
    assert_eq!(
        find_app_id(&apps(), "Systemprompt_SSO").as_deref(),
        Some("0Ci0000000001")
    );
    assert_eq!(
        find_app_id(&apps(), "Other_App").as_deref(),
        Some("0Ci0000000002")
    );
}

// Names are matched exactly. Salesforce developer names are case sensitive,
// and a loose match would grant the wrong app.
#[test]
fn an_app_lookup_does_not_match_loosely() {
    assert!(find_app_id(&apps(), "systemprompt_sso").is_none());
    assert!(find_app_id(&apps(), "Systemprompt_SS").is_none());
    assert!(find_app_id(&apps(), "").is_none());
    assert!(find_app_id(&[], "Systemprompt_SSO").is_none());
}

#[test]
fn an_app_row_without_an_id_yields_nothing() {
    let rows = vec![serde_json::json!({ "DeveloperName": "Systemprompt_SSO" })];
    assert!(find_app_id(&rows, "Systemprompt_SSO").is_none());
}

#[test]
fn a_permission_set_is_found_by_api_name() {
    assert_eq!(
        find_permission_set_id(&permsets(), "Salesforce_MCP_Access").as_deref(),
        Some("0PS0000000001")
    );
    assert!(find_permission_set_id(&permsets(), "Absent_Set").is_none());
    assert!(find_permission_set_id(&[], "Salesforce_MCP_Access").is_none());
}

#[test]
fn a_user_is_found_by_username() {
    assert_eq!(
        find_user_id(&users(), "sam@agentforce.com").as_deref(),
        Some("005000000000002")
    );
}

// The username is not the email, and a miss must stay a miss: apply records a
// follow-up rather than assigning a permission set to the wrong person.
#[test]
fn an_unknown_username_is_not_resolved() {
    assert!(find_user_id(&users(), "ed@systemprompt.io").is_none());
    assert!(find_user_id(&users(), "ED@agentforce.com").is_none());
    assert!(find_user_id(&[], "ed@agentforce.com").is_none());
}

fn grants() -> Vec<serde_json::Value> {
    vec![serde_json::json!({
        "ParentId": "0PS0000000001",
        "SetupEntityId": "0Ci0000000001",
    })]
}

#[test]
fn an_existing_grant_is_recognised() {
    assert!(grant_exists(&grants(), "0PS0000000001", "0Ci0000000001"));
}

// Both halves must match. Treating either alone as a hit would skip a grant
// the app needs to be pre-authorized.
#[test]
fn a_partial_grant_match_is_not_a_grant() {
    assert!(!grant_exists(&grants(), "0PS0000000002", "0Ci0000000001"));
    assert!(!grant_exists(&grants(), "0PS0000000001", "0Ci0000000002"));
    assert!(!grant_exists(&grants(), "", ""));
    assert!(!grant_exists(&[], "0PS0000000001", "0Ci0000000001"));
}

#[test]
fn a_grant_row_missing_a_field_is_not_a_match() {
    let rows = vec![
        serde_json::json!({ "ParentId": "0PS0000000001" }),
        serde_json::json!({ "SetupEntityId": "0Ci0000000001" }),
    ];
    assert!(!grant_exists(&rows, "0PS0000000001", "0Ci0000000001"));
}

fn held() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({ "PermissionSet": { "Name": "Salesforce_MCP_Access" } }),
        serde_json::json!({ "PermissionSet": { "Name": "Standard_User" } }),
    ]
}

#[test]
fn an_existing_assignment_is_recognised_through_the_nested_record() {
    assert!(holds_permission_set(&held(), "Salesforce_MCP_Access"));
    assert!(holds_permission_set(&held(), "Standard_User"));
}

#[test]
fn an_absent_assignment_reads_as_missing() {
    assert!(!holds_permission_set(&held(), "Other_Set"));
    assert!(!holds_permission_set(&held(), ""));
    assert!(!holds_permission_set(&[], "Salesforce_MCP_Access"));
}

// A row whose relationship did not come back must not read as held —
// that would skip an assignment the user never received.
#[test]
fn a_row_without_the_nested_record_is_not_held() {
    let rows = vec![
        serde_json::json!({ "PermissionSetId": "0PS0000000001" }),
        serde_json::json!({ "PermissionSet": null }),
        serde_json::json!({ "PermissionSet": { "Id": "0PS0000000001" } }),
    ];
    assert!(!holds_permission_set(&rows, "Salesforce_MCP_Access"));
}
