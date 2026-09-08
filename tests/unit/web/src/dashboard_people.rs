//! Regressions for reusable account privileges and additive dashboard schemas.
use crate::support::repo_root;
use systemprompt_web_admin::types::role::{
    ROLES_CONSOLE, RoleChangeRefusal, authorize_role_change, has_any,
};

fn roles(values: &[&str]) -> Vec<String> {
    values.iter().map(|v| (*v).to_owned()).collect()
}

#[test]
fn custom_roles_can_be_granted_and_revoked_without_becoming_console_privileges() {
    let admin = roles(&["admin"]);
    let before = roles(&["user", "billing-reviewer"]);
    let after = roles(&["user", "warehouse:write"]);
    assert_eq!(
        authorize_role_change(&admin, &before, &after, &[], 1),
        Ok(())
    );
    assert!(!has_any(
        &roles(&["billing-reviewer", "warehouse:write"]),
        ROLES_CONSOLE
    ));
}

#[test]
fn custom_directory_grant_remains_protected() {
    let held = roles(&["directory:payroll"]);
    assert_eq!(
        authorize_role_change(&roles(&["admin"]), &held, &[], &held, 2),
        Err(RoleChangeRefusal::DirectoryRole("directory:payroll".into()))
    );
}

#[test]
fn group_upgrade_preserves_legacy_organization_data_and_free_text_roles() {
    let root = repo_root();
    let sql = std::fs::read_to_string(
        root.join("extensions/web/schema/migrations/052_dashboard_groups_projects.sql"),
    )
    .expect("group migration")
    .to_lowercase();
    for forbidden in [
        "drop table",
        "drop column",
        "delete from departments",
        "delete from organizations",
        "delete from access_control_rules",
        "role in (",
    ] {
        assert!(
            !sql.contains(forbidden),
            "additive group migration contains {forbidden}"
        );
    }
    assert!(sql.contains("create table if not exists groups"));
    assert!(sql.contains("create table if not exists projects"));
    assert!(!sql.contains("commerce"));
    assert!(!sql.contains("india-devs"));
}
