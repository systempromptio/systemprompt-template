//! Conversation-history scope resolution: self is always visible, the full
//! view (admin / auditor) bypasses the allowlist entirely, and a plain user's
//! allowlist is exactly themselves.

use systemprompt::identifiers::UserId;
use systemprompt_web_admin::repositories::analytics::conversations::{
    HistoryScope, resolve_history_scope, roles_grant_full_history,
};

fn uid(s: &str) -> UserId {
    UserId::new(s.to_owned())
}

fn org_scope(user: &str, roles: &[&str]) -> HistoryScope {
    let roles: Vec<String> = roles.iter().map(|r| (*r).to_owned()).collect();
    resolve_history_scope(&uid(user), roles_grant_full_history(&roles), Vec::new())
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
    assert_eq!(org_scope("root", &["admin", "user"]), HistoryScope::All);
    assert_eq!(org_scope("eyes", &["auditor"]), HistoryScope::All);
}

#[test]
fn a_non_admin_scope_is_exactly_themselves() {
    let scope = org_scope("viewer", &["user"]);
    assert_eq!(scope, HistoryScope::Users(vec![uid("viewer")]));
    assert!(!scope.may_view(&uid("colleague-in-the-same-project")));
}
