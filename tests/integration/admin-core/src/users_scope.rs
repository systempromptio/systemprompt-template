//! `repositories::scope::membership` and `users::roles` — who a listing may
//! span, and how the two writers of `users.roles` are joined.

use systemprompt_web_admin::repositories::groups::crud::insert_group;
use systemprompt_web_admin::repositories::groups::members::insert_group_member;
use systemprompt_web_admin::repositories::scope::membership::get_subject_scope;
use systemprompt_web_admin::repositories::scope::{ScopeRequest, SubjectScope, Visibility};
use systemprompt_web_admin::repositories::users::roles::{
    count_platform_admins, list_directory_roles, list_manual_roles, recompute_roles,
    set_manual_roles,
};
use systemprompt_web_admin::types::groups::CreateGroupRequest;

use crate::fixtures::{insert_user, unclaimed_email, unique, unique_group};
use crate::tempdb::TempDb;

#[tokio::test]
async fn an_unfiltered_console_listing_spans_everyone() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let scope = get_subject_scope(
        &db.pool,
        &ScopeRequest {
            visibility: Visibility::All,
            group: None,
            project: None,
        },
    )
    .await
    .expect("resolve");

    assert_eq!(
        scope,
        SubjectScope::All,
        "no filter binds NULL rather than every id in the table"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn a_group_filter_narrows_to_that_groups_members() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let group = unique_group("grp");
    insert_group(
        &db.pool,
        &CreateGroupRequest {
            id: group.clone(),
            name: group.as_str().to_owned(),
            description: None,
        },
        "dashboard",
    )
    .await
    .expect("insert group");
    let member = insert_user(&db.pool, &unique("user"), &unclaimed_email("in")).await;
    let outsider = insert_user(&db.pool, &unique("user"), &unclaimed_email("out")).await;
    let admin = insert_user(&db.pool, &unique("admin"), &unclaimed_email("admin")).await;
    insert_group_member(&db.pool, &group, &member, &admin)
        .await
        .expect("add member");

    let scope = get_subject_scope(
        &db.pool,
        &ScopeRequest {
            visibility: Visibility::All,
            group: Some(group.as_str().to_owned()),
            project: None,
        },
    )
    .await
    .expect("resolve");

    let SubjectScope::Users(ids) = scope else {
        panic!("a filtered scope is an id list");
    };
    assert!(ids.contains(&member.as_str().to_owned()));
    assert!(!ids.contains(&outsider.as_str().to_owned()));
    db.cleanup().await;
}

// Why: an empty list is a legitimate answer. Widening it to everything would
// hand the least-attached caller the whole estate.
#[tokio::test]
async fn a_caller_in_no_group_sees_an_empty_list_not_everything() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let scope = get_subject_scope(
        &db.pool,
        &ScopeRequest {
            visibility: Visibility::Groups(Vec::new()),
            group: None,
            project: None,
        },
    )
    .await
    .expect("resolve");

    assert_eq!(scope, SubjectScope::Users(Vec::new()));
    db.cleanup().await;
}

#[tokio::test]
async fn the_effective_roles_are_the_union_of_both_writers() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("roles")).await;
    let admin = insert_user(&db.pool, &unique("admin"), &unclaimed_email("granter")).await;

    set_manual_roles(&db.pool, &user, &["project_manager".to_owned()], &admin)
        .await
        .expect("grant by hand");
    let roles = recompute_roles(&db.pool, &user, Some(&["user".to_owned()]))
        .await
        .expect("sign-in recompute");

    assert_eq!(
        roles,
        vec!["project_manager".to_owned(), "user".to_owned()],
        "the directory half and the manual half both survive"
    );
    assert_eq!(
        list_manual_roles(&db.pool, &user).await.expect("read"),
        vec!["project_manager".to_owned()]
    );
    assert_eq!(
        list_directory_roles(&db.pool, &user).await.expect("read"),
        vec!["user".to_owned()],
        "the directory half is whatever the effective set holds beyond the manual one"
    );
    db.cleanup().await;
}

// Why: revoking a manual grant must not take a role the directory also holds.
#[tokio::test]
async fn dropping_a_manual_role_leaves_the_directory_half_standing() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("demote")).await;
    let admin = insert_user(&db.pool, &unique("admin"), &unclaimed_email("granter")).await;
    set_manual_roles(&db.pool, &user, &["admin".to_owned()], &admin)
        .await
        .expect("grant");
    recompute_roles(&db.pool, &user, Some(&["user".to_owned()]))
        .await
        .expect("recompute");

    set_manual_roles(&db.pool, &user, &[], &admin)
        .await
        .expect("revoke");
    let roles = recompute_roles(&db.pool, &user, None)
        .await
        .expect("recompute");

    assert_eq!(roles, vec!["user".to_owned()]);
    db.cleanup().await;
}

#[tokio::test]
async fn platform_admins_are_counted_from_the_effective_role_set() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let before = count_platform_admins(&db.pool).await.expect("count");
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("platform")).await;
    let admin = insert_user(&db.pool, &unique("admin"), &unclaimed_email("granter")).await;
    set_manual_roles(&db.pool, &user, &["platform_admin".to_owned()], &admin)
        .await
        .expect("grant");
    recompute_roles(&db.pool, &user, None)
        .await
        .expect("recompute");

    assert_eq!(
        count_platform_admins(&db.pool).await.expect("count"),
        before + 1
    );
    db.cleanup().await;
}
