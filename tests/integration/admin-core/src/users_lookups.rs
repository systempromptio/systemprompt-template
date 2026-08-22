//! `repositories::users` read models — the user index, its filter options, and
//! the per-user side tables the detail page renders.

use systemprompt_web_admin::repositories::users;
use systemprompt_web_admin::util::org_scope::OrgScope;

use crate::fixtures::{
    OrgSpec, insert_member, insert_org, insert_user, insert_user_full, set_department,
    unclaimed_email, unique,
};
use crate::tempdb::TempDb;

#[tokio::test]
async fn list_users_includes_a_freshly_created_user() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("listed")).await;

    let listed = users::queries::list_users(&db.pool, &OrgScope::AllOrganizations)
        .await
        .expect("listing succeeds");

    let row = listed
        .iter()
        .find(|u| u.user_id == user)
        .expect("the new user appears in the index");
    assert!(row.is_active);
    assert_eq!(row.total_events, 0, "a user with no traffic has no events");
    db.cleanup().await;
}

#[tokio::test]
async fn list_users_excludes_anonymous_identities() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let anon = insert_user_full(
        &db.pool,
        &unique("anon"),
        &unclaimed_email("anon"),
        None,
        &["anonymous".to_owned()],
        "active",
    )
    .await;

    let listed = users::queries::list_users(&db.pool, &OrgScope::AllOrganizations)
        .await
        .expect("listing succeeds");

    assert!(
        !listed.iter().any(|u| u.user_id == anon),
        "the anonymous role is filtered out of the user index"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn list_distinct_roles_surfaces_a_new_role_and_drops_machine_roles() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let role = unique("auditor");
    insert_user_full(
        &db.pool,
        &unique("user"),
        &unclaimed_email("roled"),
        None,
        &[role.clone()],
        "active",
    )
    .await;
    insert_user_full(
        &db.pool,
        &unique("svc"),
        &unclaimed_email("svc"),
        None,
        &["service".to_owned()],
        "active",
    )
    .await;

    let roles = users::queries::list_distinct_roles(&db.pool)
        .await
        .expect("listing succeeds");

    assert!(roles.contains(&role));
    assert!(
        !roles.contains(&"service".to_owned()),
        "machine roles are filtered out of the operator-facing list"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn find_share_token_version_returns_none_without_a_profile_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("share")).await;

    let version = users::find_share_token_version(&db.pool, &user)
        .await
        .expect("lookup succeeds");

    assert_eq!(
        version, None,
        "absence of a profile row is not an error for find_"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn find_share_token_version_starts_at_one() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("share2")).await;
    set_department(&db.pool, &user, "Default").await;

    let version = users::find_share_token_version(&db.pool, &user)
        .await
        .expect("lookup succeeds");

    assert_eq!(version, Some(1));
    db.cleanup().await;
}

#[tokio::test]
async fn find_user_settings_returns_none_when_the_user_has_none() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("settings")).await;

    let settings = users::user_settings::find_user_settings(&db.pool, &user)
        .await
        .expect("lookup succeeds");

    assert!(settings.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn list_device_app_links_is_empty_for_a_user_with_no_devices() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("devices")).await;

    let devices = users::devices::list_device_app_links(&db.pool, &user)
        .await
        .expect("listing succeeds");

    assert!(devices.is_empty());
    db.cleanup().await;
}

#[tokio::test]
async fn list_device_app_links_returns_only_this_users_devices() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let mine = insert_user(&db.pool, &unique("user"), &unclaimed_email("mine")).await;
    let theirs = insert_user(&db.pool, &unique("user"), &unclaimed_email("theirs")).await;
    for (device, owner) in [("dev-a", &mine), ("dev-b", &theirs)] {
        sqlx::query(
            "INSERT INTO device_app_links (device_id, user_id, app_platform, app_version)
             VALUES ($1, $2, 'linux', '1.2.3')",
        )
        .bind(unique(device))
        .bind(owner.as_str())
        .execute(&*db.pool)
        .await
        .expect("insert device link");
    }

    let devices = users::devices::list_device_app_links(&db.pool, &mine)
        .await
        .expect("listing succeeds");

    assert_eq!(devices.len(), 1);
    assert_eq!(devices[0].app_platform, "linux");
    assert_eq!(devices[0].app_version, "1.2.3");
    db.cleanup().await;
}

// Why: `admin` is the role a customer's own administrator holds on this pooled
// instance, so an unscoped listing hands them every other customer's roster.
// The scope is the whole control, which makes it worth a test of its own.
#[tokio::test]
async fn list_users_scoped_to_an_org_excludes_other_orgs_members() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let org_a = unique("org-a");
    let org_b = unique("org-b");
    let slug_a = unique("slug-a");
    let slug_b = unique("slug-b");
    insert_org(&db.pool, &OrgSpec::active(&org_a, &slug_a)).await;
    insert_org(&db.pool, &OrgSpec::active(&org_b, &slug_b)).await;

    let ours = insert_user(&db.pool, &unique("user"), &unclaimed_email("ours")).await;
    let theirs = insert_user(&db.pool, &unique("user"), &unclaimed_email("theirs")).await;
    insert_member(&db.pool, &ours, &org_a, "member").await;
    insert_member(&db.pool, &theirs, &org_b, "member").await;

    let scoped = users::queries::list_users(&db.pool, &OrgScope::Only(slug_a.clone()))
        .await
        .expect("listing succeeds");
    let ids: Vec<&str> = scoped.iter().map(|u| u.user_id.as_str()).collect();
    assert!(
        ids.contains(&ours.as_str()),
        "the caller's own member is listed"
    );
    assert!(
        !ids.contains(&theirs.as_str()),
        "another organization's member must not be listed"
    );

    // None is the platform admin's cross-customer view, and must still span both.
    let all = users::queries::list_users(&db.pool, &OrgScope::AllOrganizations)
        .await
        .expect("listing succeeds");
    let all_ids: Vec<&str> = all.iter().map(|u| u.user_id.as_str()).collect();
    assert!(all_ids.contains(&ours.as_str()));
    assert!(all_ids.contains(&theirs.as_str()));
    db.cleanup().await;
}

// Why: an empty slug is what an admin with no organization membership resolves
// to. It must match nothing rather than falling through to "every
// organization", which would make the widest scope the one reached by having
// the least attachment.
#[tokio::test]
async fn list_users_scoped_to_an_empty_slug_lists_nobody() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let org = unique("org");
    let slug = unique("slug");
    insert_org(&db.pool, &OrgSpec::active(&org, &slug)).await;
    let member = insert_user(&db.pool, &unique("user"), &unclaimed_email("member")).await;
    insert_member(&db.pool, &member, &org, "member").await;

    let listed = users::queries::list_users(&db.pool, &OrgScope::Only(String::new()))
        .await
        .expect("listing succeeds");
    assert!(
        listed.is_empty(),
        "an empty slug must match no organization"
    );
    db.cleanup().await;
}
