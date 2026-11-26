//! The `project_manager` semi-admin role: which role sets reach the admin
//! dashboard, how the AD group glob resolves to roles, and the derived
//! history view that widens for it.
//!
//! The route gates themselves are pinned by the contract suite; this covers
//! the pure rules underneath them.

use std::collections::BTreeMap;

use systemprompt::models::auth::UserType;
use systemprompt_web_admin::repositories::analytics::conversations::roles_grant_full_history;
use systemprompt_web_admin::types::{
    NotSignedIn, ROLE_ADMIN, ROLE_PROJECT_MANAGER, ROLE_USER, roles_grant_console,
    roles_grant_manage, roles_grant_platform,
};
use systemprompt_web_admin::{AdfsConfig, group_matches_pattern};

fn roles(list: &[&str]) -> Vec<String> {
    list.iter().map(|r| (*r).to_owned()).collect()
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
    let pm = roles(&[ROLE_PROJECT_MANAGER, ROLE_USER]);
    assert!(roles_grant_console(&pm));
    assert!(
        !roles_grant_manage(&pm),
        "is_admin still guards every privileged mutation and must mean the write tier alone"
    );
    assert!(!roles_grant_platform(&pm));
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
    assert!(roles_grant_full_history(&roles(&[
        ROLE_PROJECT_MANAGER,
        ROLE_USER
    ])));
    assert!(!roles_grant_full_history(&roles(&[ROLE_USER])));
}

#[test]
fn an_anonymous_session_token_is_never_a_console_principal() {
    assert_eq!(
        NotSignedIn::refuse_token(UserType::Anon),
        Err(NotSignedIn::AnonymousSession)
    );
    assert_eq!(NotSignedIn::refuse_token(UserType::User), Ok(()));
    assert_eq!(NotSignedIn::refuse_token(UserType::Admin), Ok(()));
}
