//! `POST /approvals/{call_id}/{approve|deny}` — deciding a held call, over the
//! real router.
//!
//! The baseline table already proves the two routes exist and refuse the wrong
//! principals. What it cannot prove is the property the table is there for: a
//! decision is taken once. Two approvers watching the same queue is the normal
//! case, and the second write has to lose rather than overwrite the first, so
//! the sequence below decides a row and then decides it again.
//!
//! The verb lives in the path rather than in a body on purpose, and that is
//! asserted here too: a request carrying no body at all must still be a valid
//! decision, because the JS that drives these buttons sends none.

use axum::http::StatusCode;
use chrono::{Duration, Utc};
use sqlx::PgPool;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

const CALL_ID: &str = "contract-approval-1";

async fn seed_pending(pool: &PgPool, call_id: &str) {
    sqlx::query(
        "INSERT INTO approval_requests
             (call_id, tool_name, server_name, arguments, args_digest, requested_by,
              rule, status, expires_at)
         VALUES ($1, 'write_file', 'systemprompt', '{\"path\":\"/etc/hosts\"}',
                 'contract-digest', 'contract-requester',
                 'require_approval:write_file', 'pending', $2)
         ON CONFLICT (call_id) DO NOTHING",
    )
    .bind(call_id)
    .bind(Utc::now() + Duration::hours(1))
    .execute(pool)
    .await
    .expect("seed approval request");
}

fn approve_path(call_id: &str) -> String {
    format!("/api/public/admin/approvals/{call_id}/approve")
}

fn post(path: &str, principal: Principal) -> Call<'_> {
    Call {
        method: "post",
        path,
        principal,
        content_type: None,
        body: None,
    }
}

#[tokio::test]
async fn a_held_call_is_decided_once_and_the_second_decision_conflicts() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    seed_pending(&db.pool, CALL_ID).await;

    let path = approve_path(CALL_ID);
    let (status, body) = app.call(post(&path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK, "first decision: {body}");
    assert!(body.contains("approved"), "verdict is echoed back: {body}");

    let (status, body) = app.call(post(&path, Principal::Admin)).await;
    assert_eq!(
        status,
        StatusCode::CONFLICT,
        "a decided row must not be re-decided: {body}"
    );

    let stored: (String, Option<String>) =
        sqlx::query_as("SELECT status, approver_id FROM approval_requests WHERE call_id = $1")
            .bind(CALL_ID)
            .fetch_one(&*db.pool)
            .await
            .expect("read the decided row");
    assert_eq!(stored.0, "approved");
    assert!(stored.1.is_some(), "the approver is recorded on the row");
}

#[tokio::test]
async fn a_project_manager_may_not_decide_a_held_call() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let call_id = "contract-approval-pm";
    seed_pending(&db.pool, call_id).await;

    let path = approve_path(call_id);
    let (status, body) = app.call(post(&path, Principal::ProjectManager)).await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "reading the queue is a console act; deciding one is not: {body}"
    );

    let stored: (String,) =
        sqlx::query_as("SELECT status FROM approval_requests WHERE call_id = $1")
            .bind(call_id)
            .fetch_one(&*db.pool)
            .await
            .expect("read the untouched row");
    assert_eq!(stored.0, "pending", "the refusal left the row alone");
}
