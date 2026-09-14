//! The permission set an ADFS session is minted with.
//!
//! The JWT is checked on `permissions`, not on the roles stored beside them, so
//! a mapping that disagrees with the account's AD groups signs a token for a
//! user who is not the one who signed in.

use systemprompt::models::auth::Permission;
use systemprompt_web_admin::permissions_for_roles;

fn roles(names: &[&str]) -> Vec<String> {
    names.iter().map(|n| (*n).to_owned()).collect()
}

#[test]
fn an_admin_role_carries_the_admin_permission() {
    assert_eq!(
        permissions_for_roles(&roles(&["admin", "user"])),
        vec![Permission::Admin, Permission::User]
    );
}

#[test]
fn admin_does_not_depend_on_user_being_present_or_on_order() {
    assert_eq!(
        permissions_for_roles(&roles(&["admin"])),
        vec![Permission::Admin, Permission::User],
        "holding admin implies being a signed-in user"
    );
    assert_eq!(
        permissions_for_roles(&roles(&["user", "admin"])),
        permissions_for_roles(&roles(&["admin", "user"]))
    );
}

#[test]
fn every_other_role_set_is_only_a_user() {
    assert_eq!(
        permissions_for_roles(&roles(&["user"])),
        vec![Permission::User]
    );
    assert_eq!(permissions_for_roles(&roles(&[])), vec![Permission::User]);
    assert_eq!(
        permissions_for_roles(&roles(&["Admin", "administrator", "admins"])),
        vec![Permission::User],
        "the role name is matched exactly — a near-miss must not grant admin"
    );
}

#[test]
fn platform_admin_receives_admin_permissions_without_a_legacy_admin_role() {
    assert_eq!(
        permissions_for_roles(&roles(&["platform_admin"])),
        vec![Permission::Admin, Permission::User]
    );
    for role in ["developer", "project_manager", "knowledge_worker"] {
        assert_eq!(
            permissions_for_roles(&roles(&[role])),
            vec![Permission::User]
        );
    }
}
