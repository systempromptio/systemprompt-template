//! Ingestion identity boundaries: a delivery cannot change owner or digest,
//! and a conflicting retry is rejected rather than re-applied.
use crate::fixtures::{insert_user, unique};
use crate::tempdb::TempDb;

#[tokio::test]
async fn ingestion_rejects_owner_changes_and_conflicting_retries() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let owner = unique("owner");
    let other = unique("other");
    let session = unique("session");
    insert_user(&db.pool, &owner).await;
    insert_user(&db.pool, &other).await;
    let sql = "INSERT INTO plugin_usage_events(id,user_id,session_id,plugin_id,event_type,dedup_key,metadata) VALUES($1,$2,$3,'test','SessionStart',$1,jsonb_build_object('_ingestion_digest',$4::text)) ON CONFLICT(dedup_key) WHERE dedup_key IS NOT NULL DO NOTHING";
    let event = unique("event");
    let write = |user: String, digest: String| {
        sqlx::query(sql)
            .bind(event.clone())
            .bind(user)
            .bind(session.clone())
            .bind(digest)
    };
    assert_eq!(
        write(owner.clone(), "one".into())
            .execute(&*db.pool)
            .await
            .expect("first delivery")
            .rows_affected(),
        1
    );
    assert_eq!(
        write(owner.clone(), "one".into())
            .execute(&*db.pool)
            .await
            .expect("same delivery")
            .rows_affected(),
        0
    );
    assert!(
        write(owner, "changed".into())
            .execute(&*db.pool)
            .await
            .is_err()
    );
    assert!(write(other, "one".into()).execute(&*db.pool).await.is_err());
    let events: i64 =
        sqlx::query_scalar("SELECT count(*) FROM plugin_usage_events WHERE session_id=$1")
            .bind(session)
            .fetch_one(&*db.pool)
            .await
            .expect("events");
    assert_eq!(events, 1, "only the first delivery is recorded");
}
