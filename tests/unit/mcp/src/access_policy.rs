//! The shared MCP role predicate is an authorization boundary: only the two
//! configured administrative role strings may enable management operations.
//! Keeping this exact avoids granting access because a role merely resembles
//! an administrator role or appears alongside unrelated roles.

use systemprompt_mcp_shared::access_policy::roles_grant_manage;

fn roles(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn either_configured_administrative_role_grants_management() {
    assert!(roles_grant_manage(&roles(&["admin"])));
    assert!(roles_grant_manage(&roles(&["platform_admin"])));
    assert!(roles_grant_manage(&roles(&[
        "viewer",
        "platform_admin",
        "author"
    ])));
}

#[test]
fn empty_and_non_administrative_role_sets_cannot_manage() {
    assert!(!roles_grant_manage(&roles(&[])));
    assert!(!roles_grant_manage(&roles(&["viewer", "editor", "owner"])));
}

#[test]
fn role_matching_is_exact_and_case_sensitive() {
    // Roles are free-text configuration, so substring or case-insensitive
    // matching here would turn a harmless label into an authorization grant.
    assert!(!roles_grant_manage(&roles(&[
        "Admin",
        "platform-admin",
        "admin_assistant",
        "superadmin",
        "platform_admin_readonly",
    ])));
}
