//! The new usage-metrics capture surfaces: observed commits and their dedup
//! index, the `admin_usage_daily_rollups` recompute (idempotent by design —
//! running it twice must not double anything).

use systemprompt::identifiers::SessionId;
use systemprompt_web_admin::repositories::dashboard::commits::{NewUserCommit, insert_user_commit};
use systemprompt_web_admin::repositories::dashboard::usage_rollups;

use crate::fixtures::{RequestSpec, insert_request, insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

#[tokio::test]
async fn insert_user_commit_dedupes_on_user_cwd_and_hash() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("commits")).await;
    let session = SessionId::new(unique("sess"));
    let commit = NewUserCommit {
        user_id: &user,
        session_id: &session,
        cwd: Some("/repo"),
        branch: Some("main"),
        commit_hash: "abc1234",
        message: "first",
        files_changed: Some(1),
        insertions: Some(2),
        deletions: Some(0),
    };

    let first = insert_user_commit(&db.pool, &commit).await.expect("insert");
    let second = insert_user_commit(&db.pool, &commit)
        .await
        .expect("dup insert");
    let other_repo = insert_user_commit(
        &db.pool,
        &NewUserCommit {
            cwd: Some("/other-repo"),
            ..commit
        },
    )
    .await
    .expect("other repo insert");

    assert!(first, "a fresh commit is written");
    assert!(!second, "the same commit in the same repo collapses");
    assert!(other_repo, "the same hash in another repo is a new row");
    db.cleanup().await;
}

#[tokio::test]
async fn daily_rollups_recompute_idempotently() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("rollup")).await;
    let session = SessionId::new(unique("sess"));

    insert_request(&db.pool, &RequestSpec::completed(&unique("req"), &user)).await;
    insert_user_commit(
        &db.pool,
        &NewUserCommit {
            user_id: &user,
            session_id: &session,
            cwd: Some("/repo"),
            branch: Some("main"),
            commit_hash: "def5678",
            message: "rollup test",
            files_changed: Some(2),
            insertions: Some(30),
            deletions: Some(5),
        },
    )
    .await
    .expect("insert commit");
    sqlx::query(
        "INSERT INTO plugin_usage_daily
            (id, date, user_id, event_type, tool_name, event_count, loc_added, loc_removed)
         VALUES ($1, (NOW() AT TIME ZONE 'UTC')::DATE, $2, 'PostToolUse', 'Edit', 4, 40, 7)",
    )
    .bind(unique("pud"))
    .bind(user.as_str())
    .execute(&*db.pool)
    .await
    .expect("seed plugin_usage_daily");

    usage_rollups::upsert_daily_rollups_for_window(&db.pool, 1)
        .await
        .expect("first rollup");
    usage_rollups::upsert_daily_rollups_for_window(&db.pool, 1)
        .await
        .expect("second rollup");

    let row: (i64, i64, i64, i64, i64) = sqlx::query_as(
        "SELECT tool_uses, loc_added_ai, commits_count::BIGINT, commit_insertions,
                ai_requests_count
         FROM admin_usage_daily_rollups WHERE user_id = $1",
    )
    .bind(user.as_str())
    .fetch_one(&*db.pool)
    .await
    .expect("read rollup row");

    assert_eq!(row.0, 4, "tool uses recomputed, not doubled");
    assert_eq!(row.1, 40, "AI lines recomputed, not doubled");
    assert_eq!(row.2, 1, "one commit");
    assert_eq!(row.3, 30, "commit insertions");
    assert_eq!(row.4, 1, "one gateway request");
    db.cleanup().await;
}
