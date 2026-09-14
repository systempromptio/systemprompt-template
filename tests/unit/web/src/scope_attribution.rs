//! `repositories::scope` — the container kind, the attribution mode, and what
//! a [`Scope`] resolves to without touching the database.

use systemprompt::identifiers::UserId;
use systemprompt_web_admin::repositories::scope::{
    Attribution, Scope, ScopeKind, ScopeQuery, ScopeTarget,
};
use systemprompt_web_shared::{GroupId, ProjectId};

#[test]
fn the_container_kind_is_bound_as_its_own_name() {
    assert_eq!(ScopeKind::Group.as_str(), "group");
    assert_eq!(ScopeKind::Project.as_str(), "project");
}

#[test]
fn the_linked_container_is_the_other_one() {
    assert_eq!(ScopeKind::Group.linked(), ScopeKind::Project);
    assert_eq!(ScopeKind::Project.linked(), ScopeKind::Group);
}

// Why: the exclusive flag is the second bound parameter of the shared
// membership CTE, so getting it backwards silently swaps every total.
#[test]
fn only_exclusive_attribution_reads_the_defaults_table() {
    assert!(Attribution::Exclusive.is_exclusive());
    assert!(!Attribution::Member.is_exclusive());
}

#[test]
fn a_scope_names_its_container_and_kind() {
    let group = Scope::Group(GroupId::new("europe-devs"));
    assert_eq!(group.kind(), Some(ScopeKind::Group));
    assert_eq!(group.container_id(), Some("europe-devs"));

    let project = Scope::Project(ProjectId::new("core"));
    assert_eq!(project.kind(), Some(ScopeKind::Project));
    assert_eq!(project.container_id(), Some("core"));
}

#[test]
fn everyone_and_one_person_name_no_container() {
    assert_eq!(Scope::All.kind(), None);
    assert_eq!(Scope::All.container_id(), None);

    let user = Scope::User(UserId::new("someone".to_owned()));
    assert_eq!(user.kind(), None);
    assert_eq!(user.container_id(), None);
}

#[test]
fn a_scope_query_carries_the_window_it_was_built_with() {
    let group = GroupId::new("europe-devs");
    let query = ScopeQuery::new(ScopeTarget::Group(&group), Attribution::Exclusive, 30);
    assert_eq!(query.id(), "europe-devs");
    assert_eq!(query.kind(), ScopeKind::Group);
    assert_eq!(query.window_days, 30);
    assert!(query.attribution.is_exclusive());
}
