//! Exclusive attribution against the database: a person in two groups counts
//! once, the group totals partition the instance, and manual choices survive
//! recomputation.

use systemprompt_web_admin::repositories::groups::members::replace_directory_group_memberships;
use systemprompt_web_admin::repositories::people_usage::get_scope_usage;
use systemprompt_web_admin::repositories::people_usage::totals::list_scope_totals;
use systemprompt_web_admin::repositories::scope::defaults::{
    find_scope_defaults, recompute_scope_defaults, set_scope_defaults,
};
use systemprompt_web_admin::repositories::scope::{
    Attribution, ScopeKind, ScopeQuery, ScopeTarget,
};

use crate::fixtures::{
    RequestSpec, insert_group, insert_group_member, insert_request, insert_user, unclaimed_email,
    unique, unique_group,
};
use crate::tempdb::TempDb;

const WINDOW: i32 = 30;

#[tokio::test]
async fn a_member_of_two_groups_counts_once_exclusively_and_twice_as_a_member() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let first = unique_group("grp");
    let second = unique_group("grp");
    insert_group(&db.pool, &first, first.as_str()).await;
    insert_group(&db.pool, &second, second.as_str()).await;
    let person = insert_user(&db.pool, &unique("user"), &unclaimed_email("both")).await;
    insert_group_member(&db.pool, &first, &person, "adfs").await;
    insert_group_member(&db.pool, &second, &person, "adfs").await;
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &person)).await;
    recompute_scope_defaults(&db.pool)
        .await
        .expect("recompute defaults");

    let mut member_total = 0i64;
    for id in [&first, &second] {
        member_total += get_scope_usage(
            &db.pool,
            &ScopeQuery::new(ScopeTarget::Group(id), Attribution::Member, WINDOW),
        )
        .await
        .expect("member usage")
        .requests;
    }
    assert_eq!(
        member_total, 2,
        "membership attribution counts the same request under both groups"
    );

    let mut exclusive_total = 0i64;
    for id in [&first, &second] {
        exclusive_total += get_scope_usage(
            &db.pool,
            &ScopeQuery::new(ScopeTarget::Group(id), Attribution::Exclusive, WINDOW),
        )
        .await
        .expect("exclusive usage")
        .requests;
    }
    assert_eq!(
        exclusive_total, 1,
        "exclusive attribution files the request under the primary group only"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn exclusive_group_totals_sum_to_the_instance_total() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let group = unique_group("grp");
    insert_group(&db.pool, &group, group.as_str()).await;
    let member = insert_user(&db.pool, &unique("user"), &unclaimed_email("member")).await;
    let unplaced = insert_user(&db.pool, &unique("user"), &unclaimed_email("unplaced")).await;
    insert_group_member(&db.pool, &group, &member, "adfs").await;
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &member)).await;
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &unplaced)).await;
    recompute_scope_defaults(&db.pool)
        .await
        .expect("recompute defaults");

    let rows = list_scope_totals(&db.pool, ScopeKind::Group, Attribution::Exclusive, WINDOW)
        .await
        .expect("totals");
    let instance: i64 = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)::BIGINT FROM ai_requests
         WHERE created_at >= NOW() - INTERVAL '30 days'",
    )
    .fetch_one(&*db.pool)
    .await
    .expect("instance total");

    assert_eq!(
        rows.iter().map(|row| row.requests).sum::<i64>(),
        instance,
        "every request lands in exactly one bucket, unattributed included"
    );
    assert!(
        rows.iter().any(|row| row.scope_id == group.as_str()),
        "the seeded group is one of the buckets"
    );

    db.cleanup().await;
}

// Why: the recomputation job runs hourly, so a manual choice that it could
// overwrite would last less than an hour and mean nothing.
#[tokio::test]
async fn recomputation_leaves_a_manual_choice_alone() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let auto = unique_group("grp");
    let chosen = unique_group("grp");
    insert_group(&db.pool, &auto, auto.as_str()).await;
    insert_group(&db.pool, &chosen, chosen.as_str()).await;
    let person = insert_user(&db.pool, &unique("user"), &unclaimed_email("manual")).await;
    insert_group_member(&db.pool, &auto, &person, "adfs").await;
    recompute_scope_defaults(&db.pool)
        .await
        .expect("recompute defaults");

    set_scope_defaults(&db.pool, &person, Some(&chosen), None)
        .await
        .expect("set defaults");
    recompute_scope_defaults(&db.pool)
        .await
        .expect("recompute again");

    let stored = find_scope_defaults(&db.pool, &person)
        .await
        .expect("find defaults")
        .expect("a row");
    assert_eq!(stored.primary_group_id.as_ref(), Some(&chosen));
    assert_eq!(stored.source, "manual");

    db.cleanup().await;
}

#[tokio::test]
async fn a_person_in_one_group_gets_it_as_their_primary() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let group = unique_group("grp");
    insert_group(&db.pool, &group, group.as_str()).await;
    let person = insert_user(&db.pool, &unique("user"), &unclaimed_email("single")).await;
    insert_group_member(&db.pool, &group, &person, "adfs").await;
    recompute_scope_defaults(&db.pool)
        .await
        .expect("recompute defaults");

    let stored = find_scope_defaults(&db.pool, &person)
        .await
        .expect("find defaults")
        .expect("a row");
    assert_eq!(stored.primary_group_id.as_ref(), Some(&group));
    assert_eq!(stored.source, "auto");

    db.cleanup().await;
}

// Why: sign-in is where a production estate gets its memberships, so a sync
// that leaves the attribution key behind reads as unattributed until the
// hourly job runs — on a fresh install, an estate of zeros for an hour.
#[tokio::test]
async fn a_directory_sync_leaves_the_new_user_attributed() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let group = unique_group("grp");
    let ad_group = unique("AD-Group");
    insert_group(&db.pool, &group, group.as_str()).await;
    sqlx::query(
        "INSERT INTO group_ad_mappings (ad_group, group_id, source) VALUES ($1, $2, 'dashboard')",
    )
    .bind(&ad_group)
    .bind(&group)
    .execute(&*db.pool)
    .await
    .expect("insert ad mapping");
    let person = insert_user(&db.pool, &unique("user"), &unclaimed_email("adfs")).await;

    replace_directory_group_memberships(&db.pool, &person, &[ad_group])
        .await
        .expect("directory sync");

    let stored = find_scope_defaults(&db.pool, &person)
        .await
        .expect("find defaults")
        .expect("the sync wrote a scope-defaults row");
    assert_eq!(stored.primary_group_id.as_ref(), Some(&group));
    assert_eq!(stored.source, "auto");

    db.cleanup().await;
}
