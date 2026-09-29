//! Approval queue lifecycle: expiry is a read-time state and a decision is a
//! compare-and-set, so a console never authorises an already lapsed call or
//! overwrites another approver.

use chrono::{Duration, Utc};
use systemprompt::identifiers::{CallId, UserId};
use systemprompt_web_admin::repositories::governance::approvals::{
    ApprovalVerdict, find_approval, get_approval_stats, list_approvals_paged,
    update_approval_decision,
};

use crate::fixtures::unique;
use crate::tempdb::TempDb;

async fn seed(
    db: &TempDb,
    status: &str,
    expires_at: chrono::DateTime<Utc>,
    created_at: chrono::DateTime<Utc>,
) -> CallId {
    let call = CallId::new(unique("approval"));
    sqlx::query(
        "INSERT INTO approval_requests
             (call_id, tool_name, server_name, arguments, args_digest, requested_by,
              rule, status, expires_at, created_at)
         VALUES ($1, 'write_file', 'test-server', '{\"path\":\"/tmp/output\"}',
                 'digest', 'requester', 'require_approval:write_file', $2, $3, $4)",
    )
    .bind(call.as_str())
    .bind(status)
    .bind(expires_at)
    .bind(created_at)
    .execute(&*db.pool)
    .await
    .expect("seed approval");
    call
}

#[tokio::test]
async fn approval_filters_and_kpis_treat_lapsed_pending_rows_as_expired() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let now = Utc::now();
    let pending = seed(
        &db,
        "pending",
        now + Duration::hours(1),
        now - Duration::minutes(12),
    )
    .await;
    let lapsed = seed(
        &db,
        "pending",
        now - Duration::seconds(1),
        now - Duration::hours(2),
    )
    .await;
    let explicit_expired = seed(
        &db,
        "expired",
        now - Duration::hours(1),
        now - Duration::hours(3),
    )
    .await;
    let approved = seed(
        &db,
        "approved",
        now + Duration::hours(1),
        now - Duration::minutes(2),
    )
    .await;
    let denied = seed(
        &db,
        "denied",
        now + Duration::hours(1),
        now - Duration::minutes(1),
    )
    .await;

    let (pending_rows, pending_total) = list_approvals_paged(&db.pool, Some("pending"), 20, 0)
        .await
        .expect("pending queue");
    assert_eq!(pending_total, 1);
    assert_eq!(pending_rows.len(), 1);
    assert_eq!(pending_rows[0].call_id, pending);
    assert!(pending_rows[0].is_actionable());

    let (expired_rows, expired_total) = list_approvals_paged(&db.pool, Some("expired"), 20, 0)
        .await
        .expect("expired queue");
    assert_eq!(expired_total, 2);
    let expired_ids: Vec<_> = expired_rows.iter().map(|row| row.call_id.clone()).collect();
    assert!(expired_ids.contains(&lapsed));
    assert!(expired_ids.contains(&explicit_expired));
    assert!(expired_rows.iter().all(|row| !row.is_actionable()));

    let (approved_rows, approved_total) = list_approvals_paged(&db.pool, Some("approved"), 20, 0)
        .await
        .expect("approved queue");
    assert_eq!(approved_total, 1);
    assert_eq!(approved_rows[0].call_id, approved);
    let (denied_rows, denied_total) = list_approvals_paged(&db.pool, Some("denied"), 20, 0)
        .await
        .expect("denied queue");
    assert_eq!(denied_total, 1);
    assert_eq!(denied_rows[0].call_id, denied);

    let stats = get_approval_stats(&db.pool).await.expect("approval stats");
    assert_eq!(stats.pending, 1);
    assert_eq!(stats.approved, 1);
    assert_eq!(stats.denied, 1);
    assert_eq!(stats.expired, 2);
    assert!(
        stats.oldest_pending_minutes >= 10,
        "the live wait age is retained"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn first_approval_decision_wins_and_retains_its_audit_fields() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let call = seed(&db, "pending", Utc::now() + Duration::hours(1), Utc::now()).await;
    let first = UserId::new(unique("approver"));
    let second = UserId::new(unique("approver"));

    assert_eq!(
        update_approval_decision(
            &db.pool,
            &call,
            ApprovalVerdict {
                status: "approved",
                approver: &first,
                approver_username: "first approver",
                note: Some("safe after review"),
            },
        )
        .await
        .expect("first decision"),
        1
    );
    assert_eq!(
        update_approval_decision(
            &db.pool,
            &call,
            ApprovalVerdict {
                status: "denied",
                approver: &second,
                approver_username: "second approver",
                note: Some("attempted overwrite"),
            },
        )
        .await
        .expect("second decision query"),
        0,
        "the later decision loses its compare-and-set race"
    );
    let row = find_approval(&db.pool, &call)
        .await
        .expect("read decision")
        .expect("approval row");
    assert_eq!(row.status, "approved");
    assert_eq!(row.approver_id.as_deref(), Some(first.as_str()));
    assert_eq!(row.approver_username.as_deref(), Some("first approver"));
    assert_eq!(row.decision_note.as_deref(), Some("safe after review"));
    assert!(row.decided_at.is_some());
    db.cleanup().await;
}
