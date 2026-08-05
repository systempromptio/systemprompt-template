//! `repositories::config::acl_detect` — the after-the-fact gateway-ACL scan
//! and the decision rows it writes.

use systemprompt_web_admin::repositories::config::acl_detect::{
    GatewayAclDecision, insert_gateway_acl_decision, list_recent_unrejected_requests,
};

use crate::fixtures::{RequestSeed, at, insert_request, insert_user, unique};
use crate::tempdb::TempDb;

fn decision_json() -> serde_json::Value {
    serde_json::json!([{ "rule": "gateway_route", "access": "deny" }])
}

#[tokio::test]
async fn list_recent_unrejected_requests_includes_a_fresh_completed_request() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    let id = unique("req");
    insert_request(&db.pool, &RequestSeed::new(&id, &user, chrono::Utc::now())).await;

    let rows = list_recent_unrejected_requests(&db.pool, 60)
        .await
        .expect("list recent requests");

    let found = rows.iter().find(|r| r.id == id).expect("seeded request");
    assert_eq!(found.user_id.as_str(), user);
    assert_eq!(found.model, "claude-sonnet-4-5-20250929");
    assert!(
        found.session_id.is_none(),
        "a request with no session must not synthesize one"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn list_recent_unrejected_requests_omits_rejected_and_denied() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;

    let rejected = unique("req-rej");
    let mut seed = RequestSeed::new(&rejected, &user, chrono::Utc::now());
    seed.status = "rejected";
    seed.provider = None;
    seed.model = None;
    insert_request(&db.pool, &seed).await;

    let denied = unique("req-den");
    let mut seed = RequestSeed::new(&denied, &user, chrono::Utc::now());
    seed.status = "denied";
    insert_request(&db.pool, &seed).await;

    let rows = list_recent_unrejected_requests(&db.pool, 60)
        .await
        .expect("list recent requests");

    assert!(!rows.iter().any(|r| r.id == rejected));
    assert!(!rows.iter().any(|r| r.id == denied));

    db.cleanup().await;
}

#[tokio::test]
async fn list_recent_unrejected_requests_omits_rows_outside_the_window() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    let id = unique("req-old");
    insert_request(&db.pool, &RequestSeed::new(&id, &user, at(2001, 3, 2, 9))).await;

    let rows = list_recent_unrejected_requests(&db.pool, 5)
        .await
        .expect("list recent requests");

    assert!(
        !rows.iter().any(|r| r.id == id),
        "a request older than the window must not be re-evaluated"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn insert_gateway_acl_decision_writes_one_auditable_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    let decision_id = unique("dec");
    let rules = decision_json();

    insert_gateway_acl_decision(
        &db.pool,
        GatewayAclDecision {
            decision_id: &decision_id,
            user_id: &user,
            session_id: "sess-1",
            model: "claude-opus-4-5-20251101",
            agent_scope: "gateway",
            decision: "deny",
            reason: "route not granted",
            evaluated_rules: &rules,
        },
    )
    .await
    .expect("insert gateway acl decision");

    let row = sqlx::query_as::<_, (String, String, String)>(
        "SELECT policy, decision, tool_name FROM governance_decisions WHERE id = $1",
    )
    .bind(&decision_id)
    .fetch_one(&*db.pool)
    .await
    .expect("read back decision");

    assert_eq!(
        row.0, "gateway_acl",
        "the detector's rows must be attributable to the redundancy check"
    );
    assert_eq!(row.1, "deny");
    assert_eq!(
        row.2, "claude-opus-4-5-20251101",
        "the model stands in for the tool name on the gateway path"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn insert_gateway_acl_decision_rejects_a_duplicate_id() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    let decision_id = unique("dec");
    let rules = decision_json();
    let decision = || GatewayAclDecision {
        decision_id: &decision_id,
        user_id: &user,
        session_id: "sess-1",
        model: "m",
        agent_scope: "gateway",
        decision: "deny",
        reason: "r",
        evaluated_rules: &rules,
    };

    insert_gateway_acl_decision(&db.pool, decision())
        .await
        .expect("first insert");
    let second = insert_gateway_acl_decision(&db.pool, decision()).await;

    assert!(
        second.is_err(),
        "the decision id is the primary key, so a replay must not silently pass"
    );

    db.cleanup().await;
}
