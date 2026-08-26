//! Conversation-history scope resolution: self is always visible, an org
//! owner/admin's allowlist covers their members plus themselves exactly once,
//! and the full view bypasses the allowlist entirely.

use systemprompt::identifiers::UserId;
use systemprompt_web_admin::repositories::analytics::conversations::{
    HistoryScope, resolve_history_scope,
};

fn uid(s: &str) -> UserId {
    UserId::new(s.to_owned())
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
fn org_manager_sees_members_and_self() {
    let scope = resolve_history_scope(&uid("viewer"), false, vec![uid("a"), uid("b")]);
    assert!(scope.may_view(&uid("viewer")));
    assert!(scope.may_view(&uid("a")));
    assert!(scope.may_view(&uid("b")));
    assert!(!scope.may_view(&uid("outsider")));
}

#[test]
fn viewer_in_membership_is_not_duplicated() {
    let scope = resolve_history_scope(&uid("viewer"), false, vec![uid("viewer"), uid("a")]);
    let HistoryScope::Users(ids) = scope else {
        panic!("expected an allowlist scope");
    };
    assert_eq!(ids.iter().filter(|id| id.as_str() == "viewer").count(), 1);
}
