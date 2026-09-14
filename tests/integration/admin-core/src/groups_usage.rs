//! `repositories::people_usage` — the gateway rollups the group page draws.

use systemprompt_web_admin::repositories::groups::crud::insert_group;
use systemprompt_web_admin::repositories::groups::members::insert_group_member;
use systemprompt_web_admin::repositories::people_usage::breakdown::{
    list_linked_scopes, list_scope_top_models,
};
use systemprompt_web_admin::repositories::people_usage::{
    DEFAULT_WINDOW_DAYS, get_scope_usage, list_daily_requests,
};
use systemprompt_web_admin::repositories::scope::{Attribution, ScopeQuery, ScopeTarget};
use systemprompt_web_admin::types::groups::CreateGroupRequest;
use systemprompt_web_shared::GroupId;

use crate::fixtures::{
    RequestSpec, insert_request, insert_user, unclaimed_email, unique, unique_group,
};
use crate::tempdb::TempDb;

const WINDOW: i32 = 30;

async fn seed_group(pool: &sqlx::PgPool, id: &GroupId) {
    insert_group(
        pool,
        &CreateGroupRequest {
            id: id.clone(),
            name: id.as_str().to_owned(),
            description: None,
        },
        "dashboard",
    )
    .await
    .expect("insert group");
}

#[tokio::test]
async fn the_summary_counts_only_this_groups_members() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let group = unique_group("grp");
    seed_group(&db.pool, &group).await;
    let member = insert_user(&db.pool, &unique("user"), &unclaimed_email("member")).await;
    let outsider = insert_user(&db.pool, &unique("user"), &unclaimed_email("outsider")).await;
    let admin = insert_user(&db.pool, &unique("admin"), &unclaimed_email("admin")).await;
    insert_group_member(&db.pool, &group, &member, &admin)
        .await
        .expect("add member");

    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &member)).await;
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &member)).await;
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &outsider)).await;

    let summary = get_scope_usage(
        &db.pool,
        &ScopeQuery::new(ScopeTarget::Group(&group), Attribution::Member, WINDOW),
    )
    .await
    .expect("summary");

    assert_eq!(summary.requests, 2);
    assert_eq!(summary.active_members, 1);
    assert_eq!(
        summary.tokens, 240,
        "the provider's own token count is summed, not the components twice"
    );
    assert_eq!(summary.cost_microdollars, 10_000);
    db.cleanup().await;
}

// Why: `unassigned` has no membership rows at all, so a usage query written
// against `group_members` would report nothing for the group that most needs
// watching.
#[tokio::test]
async fn the_derived_unassigned_group_reports_usage() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let unplaced = insert_user(&db.pool, &unique("user"), &unclaimed_email("unplaced")).await;
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &unplaced)).await;

    let summary = get_scope_usage(
        &db.pool,
        &ScopeQuery::new(
            ScopeTarget::Group(&GroupId::new("unassigned")),
            Attribution::Member,
            WINDOW,
        ),
    )
    .await
    .expect("summary");

    assert!(summary.requests >= 1);
    db.cleanup().await;
}

#[tokio::test]
async fn the_model_leaderboard_and_daily_series_agree_with_the_summary() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let group = unique_group("grp");
    seed_group(&db.pool, &group).await;
    let member = insert_user(&db.pool, &unique("user"), &unclaimed_email("member")).await;
    let admin = insert_user(&db.pool, &unique("admin"), &unclaimed_email("admin")).await;
    insert_group_member(&db.pool, &group, &member, &admin)
        .await
        .expect("add member");
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &member)).await;
    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &member)).await;

    let models = list_scope_top_models(
        &db.pool,
        &ScopeQuery::new(ScopeTarget::Group(&group), Attribution::Member, WINDOW),
        10,
    )
    .await
    .expect("models");
    let daily = list_daily_requests(
        &db.pool,
        &ScopeQuery::new(ScopeTarget::Group(&group), Attribution::Member, WINDOW),
    )
    .await
    .expect("daily");

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].requests, 2);
    assert_eq!(
        daily.iter().map(|d| d.requests).sum::<i64>(),
        2,
        "the series covers the same rows the leaderboard does"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn a_groups_projects_are_the_projects_its_members_are_in() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let group = unique_group("grp");
    seed_group(&db.pool, &group).await;
    let member = insert_user(&db.pool, &unique("user"), &unclaimed_email("member")).await;
    let admin = insert_user(&db.pool, &unique("admin"), &unclaimed_email("admin")).await;
    insert_group_member(&db.pool, &group, &member, &admin)
        .await
        .expect("add member");
    sqlx::query(
        "INSERT INTO project_members (project_id, user_id, source) VALUES ('core', $1, 'manual')",
    )
    .bind(member.as_str())
    .execute(&*db.pool)
    .await
    .expect("attach to a project");

    let projects = list_linked_scopes(
        &db.pool,
        &ScopeQuery::new(
            ScopeTarget::Group(&group),
            Attribution::Member,
            DEFAULT_WINDOW_DAYS,
        ),
    )
    .await
    .expect("read");

    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].id, "core");
    assert_eq!(projects[0].member_count, 1);
    db.cleanup().await;
}
