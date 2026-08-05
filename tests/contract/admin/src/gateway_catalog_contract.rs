//! The per-user gateway catalog and the after-the-fact ACL detector.
//!
//! Both surfaces are redundant by design — real enforcement happens in core's
//! `AuthzDecisionHook` — and redundancy is exactly what makes them easy to
//! leave broken: nothing downstream fails when the catalog quietly returns
//! everything, or when the detector quietly emits nothing. The exhaustive
//! table drives each route once, with a user id that exists nowhere, which
//! pins the `404` and leaves every loop body unentered.
//!
//! So the cases below put rules in the database and requests in
//! `ai_requests`, and assert on what is *kept* and what is *written*.
//!
//! Two facts shape every assertion here. The resolver's default is **deny**:
//! a route with no `access_control_entities` row and no rules is invisible,
//! so the interesting transition is empty-to-populated rather than the other
//! way round. And the detector sweeps *every* recent request in the database,
//! including the ~1080 that migration `025_demo_organizations` seeds, so
//! nothing asserts on a global count — the evidence is the row written
//! against the user the case created.

use axum::http::StatusCode;
use serde_json::Value;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

// The two routes `fixtures/profile.yaml` declares. `contract-claude` matches
// the `claude-contract-model` that `seed::insert_request` writes.
const CLAUDE_ROUTE: &str = "contract-claude";
const GPT_ROUTE: &str = "contract-gpt";

const DETECT: &str = "/api/public/admin/gateway/acl/detect";

fn parse(body: &str) -> Value {
    serde_json::from_str(body).unwrap_or_else(|e| panic!("response is JSON: {e}\n{body}"))
}

fn catalog_path(user_id: &str) -> String {
    format!("/api/public/admin/gateway/catalog/for-user/{user_id}")
}

// The route ids the catalog returned, sorted.
async fn catalog_for(app: &App, user_id: &str) -> Vec<String> {
    let (status, body) = app
        .call(Call::get(&catalog_path(user_id), Principal::Admin))
        .await;
    assert_eq!(status, StatusCode::OK, "catalog: {body}");
    let parsed = parse(&body);
    assert_eq!(parsed["user_id"], user_id);
    let mut ids: Vec<String> = parsed["routes"]
        .as_array()
        .expect("routes is an array")
        .iter()
        .map(|r| r["id"].as_str().unwrap_or_default().to_owned())
        .collect();
    ids.sort();
    ids
}

async fn rule_for_user(pool: &PgPool, route_id: &str, user_id: &UserId, access: &str) {
    seed::insert_acl_rule(
        pool,
        &seed::AclRule {
            entity_type: "gateway_route",
            entity_id: route_id,
            rule_type: "user",
            rule_value: user_id.as_str(),
            access,
        },
    )
    .await;
}

// Run a sweep and return the decisions recorded against one user.
//
// A 5xx here is the detector failing to write the row it just decided on, not
// a moved goalpost: `governance_decisions.decision` is constrained to
// allow/deny, so a sweep that finds anything to flag fails its own insert.
async fn sweep_and_count(app: &App, pool: &PgPool, user_id: &UserId) -> i64 {
    let (status, body) = app.call(Call::get(DETECT, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK, "sweep: {body}");
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM governance_decisions
         WHERE user_id = $1 AND policy = 'gateway_acl'",
    )
    .bind(user_id.as_str())
    .fetch_one(pool)
    .await
    .expect("count detector decisions")
}

#[tokio::test]
async fn the_catalog_returns_only_the_routes_a_user_is_granted() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let id = seed::unique("catalog-user");
    let user = seed::insert_user(&db.pool, &id, &format!("{id}@contract.test")).await;

    // Nothing granted, nothing listed. A catalog that defaulted to open would
    // advertise every model to every employee.
    assert!(
        catalog_for(&app, &id).await.is_empty(),
        "an unconfigured user sees no routes"
    );

    // Grant one. The other must stay hidden — a rule leaking from one entity
    // onto its siblings is the failure mode worth pinning.
    rule_for_user(&db.pool, CLAUDE_ROUTE, &user, "allow").await;
    assert_eq!(
        catalog_for(&app, &id).await,
        vec![CLAUDE_ROUTE.to_owned()],
        "only the granted route appears"
    );

    rule_for_user(&db.pool, GPT_ROUTE, &user, "allow").await;
    assert_eq!(
        catalog_for(&app, &id).await,
        vec![CLAUDE_ROUTE.to_owned(), GPT_ROUTE.to_owned()],
        "both grants are honoured"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn a_grant_to_one_user_does_not_reach_another() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let granted_id = seed::unique("catalog-granted");
    let granted = seed::insert_user(
        &db.pool,
        &granted_id,
        &format!("{granted_id}@contract.test"),
    )
    .await;
    let other_id = seed::unique("catalog-other");
    seed::insert_user(&db.pool, &other_id, &format!("{other_id}@contract.test")).await;

    rule_for_user(&db.pool, CLAUDE_ROUTE, &granted, "allow").await;

    assert_eq!(catalog_for(&app, &granted_id).await, vec![CLAUDE_ROUTE]);
    assert!(
        catalog_for(&app, &other_id).await.is_empty(),
        "the rule is scoped to the user it names"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn the_catalog_and_the_detector_are_both_behind_the_admin_gate() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    // Both routes are mounted behind the admin middleware, so a non-admin is
    // stopped before either handler runs. Note this means the catalog
    // handler's own "or the subject themselves" carve-out is unreachable over
    // HTTP: a user cannot read their own catalog through this endpoint.
    for path in [catalog_path(&seed::unique("someone-else")), DETECT.to_owned()] {
        let (status, body) = app.call(Call::get(&path, Principal::NonAdmin)).await;
        assert_eq!(status, StatusCode::FORBIDDEN, "non-admin {path}: {body}");
        assert_eq!(parse(&body)["error"], "Admin access required");
    }

    // An admin asking about a user who does not exist gets the honest answer.
    let (status, body) = app
        .call(Call::get(
            &catalog_path(&seed::unique("ghost")),
            Principal::Admin,
        ))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "unknown user: {body}");
    assert_eq!(parse(&body)["error"], "User not found");

    db.cleanup().await;
}

#[tokio::test]
async fn the_detector_echoes_the_window_it_swept() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    // No window given: the handler's own default stands in, and it is echoed
    // so the caller knows what was actually swept rather than guessing.
    let (status, body) = app.call(Call::get(DETECT, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK, "default window: {body}");
    assert_eq!(parse(&body)["since_minutes"], 60);

    let (status, body) = app
        .call(Call::get(&format!("{DETECT}?since_minutes=5"), Principal::Admin))
        .await;
    assert_eq!(status, StatusCode::OK, "explicit window: {body}");
    assert_eq!(parse(&body)["since_minutes"], 5);

    db.cleanup().await;
}

#[tokio::test]
async fn the_detector_records_a_request_that_should_have_been_denied() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let id = seed::unique("detect-user");
    let user = seed::insert_user(&db.pool, &id, &format!("{id}@contract.test")).await;
    let session = seed::unique("detect-session");
    seed::insert_session(&db.pool, &session, &user).await;
    let context = uuid::Uuid::new_v4().to_string();
    seed::insert_context(
        &db.pool,
        &context,
        &user,
        Some(&session),
        "Contract conversation",
    )
    .await;

    // Grant the route first, so the recorded request was legitimate. Without
    // this half the case cannot tell "the detector works" from "the detector
    // flags everything it sees".
    rule_for_user(&db.pool, CLAUDE_ROUTE, &user, "allow").await;

    let request_id = seed::unique("detect-request");
    seed::insert_request(
        &db.pool,
        &seed::RequestSpec {
            id: request_id.clone(),
            user_id: &user,
            session_id: Some(&session),
            trace_id: None,
            context_id: &context,
            status: "completed",
        },
    )
    .await;

    assert_eq!(
        sweep_and_count(&app, &db.pool, &user).await,
        0,
        "a request the ACL permits is not flagged"
    );

    // Revoke. The request is now, retroactively, one that should not have been
    // allowed — which is precisely the drift this detector exists to surface.
    sqlx::query("DELETE FROM access_control_rules WHERE rule_value = $1")
        .bind(user.as_str())
        .execute(db.pool.as_ref())
        .await
        .expect("revoke the grant");

    assert_eq!(
        sweep_and_count(&app, &db.pool, &user).await,
        1,
        "the now-denied request is flagged"
    );

    // The count is a summary; the durable output is the audit row, and that is
    // what an operator actually goes looking at.
    let (decision, policy, actor_id, reason, evaluated) =
        sqlx::query_as::<_, (String, String, String, String, Option<Value>)>(
            "SELECT decision, policy, actor_id, reason, evaluated_rules
             FROM governance_decisions WHERE user_id = $1 AND policy = 'gateway_acl'",
        )
        .bind(user.as_str())
        .fetch_one(db.pool.as_ref())
        .await
        .expect("a decision row was written");

    // `decision` is constrained to allow/deny, so what marks this row as a
    // redundancy check rather than live enforcement is the policy and actor.
    assert_eq!(decision, "deny");
    assert_eq!(policy, "gateway_acl");
    assert_eq!(actor_id, "gateway_acl_detector");
    assert!(!reason.is_empty(), "the row says why it was denied");

    let evaluated = evaluated.expect("evaluated_rules is populated");
    assert_eq!(
        evaluated["ai_request_id"], request_id,
        "the audit points back at the request it judged"
    );
    assert_eq!(evaluated["matched_route_id"], CLAUDE_ROUTE);
    assert_eq!(evaluated["model"], "claude-contract-model");

    db.cleanup().await;
}

#[tokio::test]
async fn the_detector_skips_requests_outside_the_window_and_off_the_catalog() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let id = seed::unique("detect-window");
    let user = seed::insert_user(&db.pool, &id, &format!("{id}@contract.test")).await;
    let session = seed::unique("detect-window-session");
    seed::insert_session(&db.pool, &session, &user).await;
    let context = uuid::Uuid::new_v4().to_string();
    seed::insert_context(
        &db.pool,
        &context,
        &user,
        Some(&session),
        "Contract conversation",
    )
    .await;
    seed::insert_request(
        &db.pool,
        &seed::RequestSpec {
            id: seed::unique("old-request"),
            user_id: &user,
            session_id: Some(&session),
            trace_id: None,
            context_id: &context,
            status: "completed",
        },
    )
    .await;

    // Age it out. A sweep that ignored `since_minutes` would re-flag every
    // historical request on every run, and the audit table would grow without
    // bound from a button an operator pressed twice.
    sqlx::query("UPDATE ai_requests SET created_at = NOW() - INTERVAL '3 hours' WHERE user_id = $1")
        .bind(user.as_str())
        .execute(db.pool.as_ref())
        .await
        .expect("age the request");

    let (status, body) = app
        .call(Call::get(
            &format!("{DETECT}?since_minutes=30"),
            Principal::Admin,
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "windowed sweep: {body}");
    assert_eq!(
        detector_rows(&db.pool, &user).await,
        0,
        "a request older than the window is not swept"
    );

    // Bring it back into the window but point it at a model no route matches.
    // An unrouted model has no ACL to violate, so it is skipped rather than
    // defaulting to denied.
    sqlx::query(
        "UPDATE ai_requests SET created_at = NOW(), model = 'llama-3-70b' WHERE user_id = $1",
    )
    .bind(user.as_str())
    .execute(db.pool.as_ref())
    .await
    .expect("retarget at an unrouted model");

    assert_eq!(
        sweep_and_count(&app, &db.pool, &user).await,
        0,
        "no route matched the model, so there was nothing to judge"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn already_rejected_requests_are_not_swept_again() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let id = seed::unique("detect-rejected");
    let user = seed::insert_user(&db.pool, &id, &format!("{id}@contract.test")).await;
    let session = seed::unique("detect-rejected-session");
    seed::insert_session(&db.pool, &session, &user).await;
    let context = uuid::Uuid::new_v4().to_string();
    seed::insert_context(
        &db.pool,
        &context,
        &user,
        Some(&session),
        "Contract conversation",
    )
    .await;

    // A request live enforcement already refused. Re-flagging it would double
    // count the same incident: the point of the detector is to catch what
    // enforcement *missed*.
    seed::insert_request(
        &db.pool,
        &seed::RequestSpec {
            id: seed::unique("rejected-request"),
            user_id: &user,
            session_id: Some(&session),
            trace_id: None,
            context_id: &context,
            status: "rejected",
        },
    )
    .await;

    assert_eq!(
        sweep_and_count(&app, &db.pool, &user).await,
        0,
        "a request enforcement already rejected is skipped"
    );

    db.cleanup().await;
}

async fn detector_rows(pool: &PgPool, user_id: &UserId) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM governance_decisions
         WHERE user_id = $1 AND policy = 'gateway_acl'",
    )
    .bind(user_id.as_str())
    .fetch_one(pool)
    .await
    .expect("count detector decisions")
}

