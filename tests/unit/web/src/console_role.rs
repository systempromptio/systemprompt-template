//! The `project_manager` semi-admin role: which role sets reach the admin
//! dashboard, how the AD group glob resolves to roles, and the derived
//! history view that widens for it.
//!
//! The route gates themselves are pinned by the contract suite; this covers
//! the pure rules underneath them.

use std::collections::BTreeMap;

use systemprompt::identifiers::{Email, UserId};
use systemprompt_web_admin::repositories::analytics::conversations::has_full_history_view;
use systemprompt_web_admin::types::{
    ROLE_ADMIN, ROLE_PROJECT_MANAGER, ROLE_USER, roles_grant_console, roles_grant_manage,
};
use systemprompt_web_admin::{AdfsConfig, UserContext, group_matches_pattern};

fn roles(list: &[&str]) -> Vec<String> {
    list.iter().map(|r| (*r).to_owned()).collect()
}

fn ctx(list: &[&str], groups: &[&str]) -> UserContext {
    let roles = roles(list);
    UserContext {
        user_id: UserId::new("u-1".to_owned()),
        username: "u-1".to_owned(),
        email: Email::try_new("u-1@example.test".to_owned()).expect("fixture email"),
        group_ids: groups.iter().map(|g| (*g).to_owned()).collect(),
        project_ids: Vec::new(),
        is_admin: roles_grant_manage(&roles),
        is_console: roles_grant_console(&roles),
        is_platform_admin: roles.iter().any(|r| r == "platform_admin"),
        is_developer: roles.iter().any(|r| r == "developer"),
        roles,
        email_verified: true,
        session_id: None,
    }
}

#[test]
fn only_the_admin_roles_and_project_manager_reach_the_console() {
    assert!(roles_grant_console(&roles(&[ROLE_ADMIN])));
    assert!(roles_grant_console(&roles(&["platform_admin"])));
    assert!(roles_grant_console(&roles(&[
        ROLE_PROJECT_MANAGER,
        ROLE_USER
    ])));
    assert!(!roles_grant_console(&roles(&[ROLE_USER])));
    assert!(!roles_grant_console(&roles(&["developer"])));
    assert!(!roles_grant_console(&roles(&["auditor"])));
    assert!(!roles_grant_console(&[]));
}

#[test]
fn a_project_manager_is_not_an_admin() {
    let pm = ctx(&[ROLE_PROJECT_MANAGER, ROLE_USER], &[]);
    assert!(pm.is_console);
    assert!(
        !pm.is_admin,
        "is_admin still guards every privileged mutation and must mean the write tier alone"
    );
    assert!(!pm.is_platform_admin);
}

#[test]
fn the_group_glob_matches_prefix_and_suffix() {
    assert!(group_matches_pattern(
        "Systemprompt-ProjectManagers-*",
        "Systemprompt-ProjectManagers-UK"
    ));
    assert!(group_matches_pattern(
        "Systemprompt-ProjectManagers-*",
        "Systemprompt-ProjectManagers-India"
    ));
    assert!(!group_matches_pattern(
        "Systemprompt-ProjectManagers-*",
        "Systemprompt-ProjectManagers"
    ));
    assert!(!group_matches_pattern(
        "Systemprompt-ProjectManagers-*",
        "Systemprompt-Admins"
    ));
    assert!(group_matches_pattern(
        "Systemprompt-Admins",
        "Systemprompt-Admins"
    ));
    assert!(!group_matches_pattern(
        "Systemprompt-Admins",
        "systemprompt-admins"
    ));
}

fn pattern_config() -> AdfsConfig {
    let mut cfg = AdfsConfig::disabled();
    cfg.group_roles = BTreeMap::from([(
        "Systemprompt-Admins".to_owned(),
        vec![ROLE_ADMIN.to_owned(), ROLE_USER.to_owned()],
    )]);
    cfg.group_role_patterns = BTreeMap::from([(
        "Systemprompt-ProjectManagers-*".to_owned(),
        vec![ROLE_PROJECT_MANAGER.to_owned(), ROLE_USER.to_owned()],
    )]);
    cfg
}

#[test]
fn a_regional_pm_group_maps_to_the_role() {
    let cfg = pattern_config();
    assert_eq!(
        cfg.roles_for_groups(&roles(&["Systemprompt-ProjectManagers-Europe"])),
        vec![ROLE_PROJECT_MANAGER.to_owned(), ROLE_USER.to_owned()]
    );
}

#[test]
fn patterns_are_applied_on_top_of_exact_groups() {
    let cfg = pattern_config();
    let granted = cfg.roles_for_groups(&roles(&[
        "Systemprompt-Admins",
        "Systemprompt-ProjectManagers-UK",
    ]));
    assert_eq!(
        granted,
        vec![
            ROLE_ADMIN.to_owned(),
            ROLE_PROJECT_MANAGER.to_owned(),
            ROLE_USER.to_owned()
        ]
    );
}

#[test]
fn an_unmapped_group_grants_no_role() {
    let cfg = pattern_config();
    assert!(
        cfg.roles_for_groups(&roles(&["Domain-Users"])).is_empty(),
        "the role map is silent about it; the login gate is what turns that into [user]"
    );
}

#[test]
fn a_project_manager_sees_every_conversation() {
    assert!(has_full_history_view(&ctx(
        &[ROLE_PROJECT_MANAGER, ROLE_USER],
        &[]
    )));
    assert!(!has_full_history_view(&ctx(&[ROLE_USER], &["europe-devs"])));
}
