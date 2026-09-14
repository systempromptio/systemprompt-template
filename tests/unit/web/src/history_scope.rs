//! Conversation-history scope resolution: self is always visible, the full
//! view (admin / auditor) bypasses the allowlist entirely, and a plain user's
//! allowlist is exactly themselves.

use systemprompt::identifiers::{Email, UserId};
use systemprompt_web_admin::UserContext;
use systemprompt_web_admin::repositories::analytics::conversations::{
    HistoryScope, history_scope_for, resolve_history_scope,
};

fn uid(s: &str) -> UserId {
    UserId::new(s.to_owned())
}

fn ctx(user: &str, roles: &[&str]) -> UserContext {
    UserContext {
        user_id: uid(user),
        username: user.to_owned(),
        email: Email::try_new(format!("{user}@example.test")).expect("fixture email"),
        roles: roles.iter().map(|r| (*r).to_owned()).collect(),
        group_ids: Vec::new(),
        project_ids: vec!["core".to_owned()],
        is_admin: roles.contains(&"admin"),
        is_console: roles.contains(&"admin") || roles.contains(&"project_manager"),
        is_platform_admin: roles.contains(&"platform_admin"),
        is_developer: roles.contains(&"developer"),
        email_verified: true,
        session_id: None,
    }
}

#[test]
fn full_view_is_unrestricted() {
    let scope = resolve_history_scope(&uid("viewer"), true, vec![uid("someone")]);
    assert_eq!(scope, HistoryScope::All);
    assert!(scope.may_view(&uid("anyone-at-all")));
    assert_eq!(scope.user_ids(), None);
}

#[test]
fn plain_user_sees_only_self() {
    let scope = resolve_history_scope(&uid("viewer"), false, Vec::new());
    assert!(scope.may_view(&uid("viewer")));
    assert!(!scope.may_view(&uid("other")));
    assert_eq!(scope.user_ids(), Some(vec!["viewer".to_owned()]));
}

#[test]
fn viewer_in_membership_is_not_duplicated() {
    let scope = resolve_history_scope(&uid("viewer"), false, vec![uid("viewer"), uid("a")]);
    let HistoryScope::Users(ids) = scope else {
        panic!("expected an allowlist scope");
    };
    assert_eq!(ids.iter().filter(|id| id.as_str() == "viewer").count(), 1);
}

#[test]
fn an_admin_or_auditor_gets_the_full_view() {
    assert_eq!(
        history_scope_for(&ctx("root", &["admin", "user"])),
        HistoryScope::All
    );
    assert_eq!(
        history_scope_for(&ctx("eyes", &["auditor"])),
        HistoryScope::All
    );
}

#[test]
fn a_non_admin_scope_is_exactly_themselves() {
    let scope = history_scope_for(&ctx("viewer", &["user"]));
    assert_eq!(scope, HistoryScope::Users(vec![uid("viewer")]));
    assert!(!scope.may_view(&uid("colleague-in-the-same-project")));
}
