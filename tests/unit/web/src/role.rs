//! The flat role set: the three router tiers, the parse rules, and the pure
//! authorisation for a role edit.
//!
//! Every one of these is decidable without a database, which is the point:
//! the role editor's whole rule set is a function, so the cases that must
//! never be reachable — demoting the last platform admin, an ordinary admin
//! minting a platform admin — are pinned here rather than in an integration
//! test that could stop covering them.

use systemprompt_web_admin::types::role::{
    ROLES_CONSOLE, ROLES_MANAGE, ROLES_PLATFORM, Role, RoleChangeRefusal, authorize_role_change,
    has_any, parse_roles,
};

fn roles(list: &[&str]) -> Vec<String> {
    list.iter().map(|r| (*r).to_owned()).collect()
}

#[test]
fn every_role_round_trips_through_its_wire_string() {
    for role in Role::ALL {
        assert_eq!(
            role.as_str().parse::<Role>().expect("role parses"),
            role,
            "{role} does not round trip"
        );
        assert!(!role.label().is_empty());
    }
    assert_eq!(Role::ALL.len(), 7);
}

#[test]
fn an_unknown_role_string_is_dropped_not_fatal() {
    let parsed = parse_roles(&roles(&["admin", "sorcerer", "user"]));
    assert_eq!(parsed, vec![Role::Admin, Role::User]);
    assert!("Admin".parse::<Role>().is_err(), "matching is exact");
}

#[test]
fn the_three_tiers_narrow_strictly() {
    assert!(has_any(&roles(&["project_manager"]), ROLES_CONSOLE));
    assert!(!has_any(&roles(&["project_manager"]), ROLES_MANAGE));
    assert!(has_any(&roles(&["admin"]), ROLES_MANAGE));
    assert!(!has_any(&roles(&["admin"]), ROLES_PLATFORM));
    assert!(has_any(&roles(&["platform_admin"]), ROLES_PLATFORM));
    assert!(has_any(&roles(&["platform_admin"]), ROLES_MANAGE));
    assert!(has_any(&roles(&["platform_admin"]), ROLES_CONSOLE));
    assert!(!has_any(&roles(&["user", "developer"]), ROLES_CONSOLE));
    assert!(!has_any(&[], ROLES_CONSOLE));
}

#[test]
fn a_developer_is_not_a_console_role() {
    assert!(
        !has_any(&roles(&["developer"]), ROLES_CONSOLE),
        "developer names what someone builds, never what they may administer"
    );
}

#[test]
fn an_ordinary_admin_may_not_mint_a_platform_admin() {
    let refusal = authorize_role_change(
        &roles(&["admin"]),
        &roles(&["user"]),
        &roles(&["user", "platform_admin"]),
        &[],
        2,
    );
    assert_eq!(refusal, Err(RoleChangeRefusal::PlatformAdminRequired));
}

#[test]
fn an_ordinary_admin_may_not_revoke_a_platform_admin_either() {
    let refusal = authorize_role_change(
        &roles(&["admin"]),
        &roles(&["user", "platform_admin"]),
        &roles(&["user"]),
        &[],
        3,
    );
    assert_eq!(refusal, Err(RoleChangeRefusal::PlatformAdminRequired));
}

#[test]
fn a_platform_admin_may_grant_the_role() {
    assert_eq!(
        authorize_role_change(
            &roles(&["platform_admin"]),
            &roles(&["user"]),
            &roles(&["user", "platform_admin"]),
            &[],
            1,
        ),
        Ok(())
    );
}

#[test]
fn the_last_platform_admin_cannot_be_demoted() {
    assert_eq!(
        authorize_role_change(
            &roles(&["platform_admin"]),
            &roles(&["platform_admin"]),
            &roles(&["user"]),
            &[],
            1,
        ),
        Err(RoleChangeRefusal::LastPlatformAdmin)
    );
    assert_eq!(
        authorize_role_change(
            &roles(&["platform_admin"]),
            &roles(&["platform_admin"]),
            &roles(&["user"]),
            &[],
            2,
        ),
        Ok(()),
        "one other platform admin is enough"
    );
}

#[test]
fn a_role_the_directory_holds_cannot_be_revoked_by_hand() {
    let refusal = authorize_role_change(
        &roles(&["admin"]),
        &roles(&["user", "admin"]),
        &roles(&["user"]),
        &roles(&["admin"]),
        2,
    );
    assert_eq!(
        refusal,
        Err(RoleChangeRefusal::DirectoryRole("admin".to_owned())),
        "the next sign-in would restore it, so refusing is honest"
    );
}

#[test]
fn a_directory_role_kept_in_place_is_fine() {
    assert_eq!(
        authorize_role_change(
            &roles(&["admin"]),
            &roles(&["user", "admin"]),
            &roles(&["user", "admin", "developer"]),
            &roles(&["admin"]),
            2,
        ),
        Ok(())
    );
}

#[test]
fn an_unknown_role_is_refused_by_name() {
    assert_eq!(
        authorize_role_change(
            &roles(&["platform_admin"]),
            &roles(&["user"]),
            &roles(&["user", "sorcerer"]),
            &[],
            2,
        ),
        Err(RoleChangeRefusal::UnknownRole("sorcerer".to_owned()))
    );
}

#[test]
fn every_refusal_renders_a_reason_a_person_can_act_on() {
    for refusal in [
        RoleChangeRefusal::PlatformAdminRequired,
        RoleChangeRefusal::LastPlatformAdmin,
        RoleChangeRefusal::UnknownRole("x".to_owned()),
        RoleChangeRefusal::DirectoryRole("admin".to_owned()),
    ] {
        assert!(
            refusal.to_string().len() > 15,
            "{refusal:?} says too little"
        );
    }
}

#[test]
fn executive_entitlement_does_not_grant_console_or_platform_powers() {
    let executive = roles(&["user", "super_admin"]);
    assert!(!has_any(&executive, ROLES_CONSOLE));
    assert!(!has_any(&executive, ROLES_MANAGE));
    assert!(!has_any(&executive, ROLES_PLATFORM));
}

#[test]
fn only_platform_admin_can_grant_or_revoke_super_admin() {
    for (before, after) in [
        (roles(&["user"]), roles(&["user", "super_admin"])),
        (roles(&["user", "super_admin"]), roles(&["user"])),
    ] {
        for caller in ["admin", "super_admin", "project_manager", "user"] {
            assert_eq!(
                authorize_role_change(&roles(&[caller]), &before, &after, &[], 1),
                Err(RoleChangeRefusal::PlatformAdminRequired),
                "{caller} must not change executive entitlement"
            );
        }
        assert_eq!(
            authorize_role_change(&roles(&["platform_admin"]), &before, &after, &[], 1),
            Ok(())
        );
    }
}

#[test]
fn ordinary_admin_can_preserve_an_existing_executive_entitlement() {
    assert_eq!(
        authorize_role_change(
            &roles(&["admin"]),
            &roles(&["user", "super_admin"]),
            &roles(&["user", "super_admin", "knowledge_worker"]),
            &[],
            1,
        ),
        Ok(())
    );
}
