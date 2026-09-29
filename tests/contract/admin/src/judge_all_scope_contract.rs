//! Bulk judging is a write surface: selected contexts are re-read under the
//! requested scope and an arbitrary return target must never become an open
//! redirect.

use axum::http::StatusCode;
use chrono::{Duration, Utc};

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

fn origin() -> String {
    let profile = systemprompt::config::ProfileBootstrap::get().expect("fixture profile");
    url::Url::parse(&profile.server.api_external_url)
        .expect("fixture origin")
        .origin()
        .ascii_serialization()
}

async fn insert_fact(pool: &sqlx::PgPool, context: &str, user: &systemprompt::identifiers::UserId) {
    let now = Utc::now();
    sqlx::query("INSERT INTO conversation_facts (context_id, user_id, client_kind, client_attestation, wire_protocol, turn_count, first_at, last_at, duration_seconds) VALUES ($1, $2, 'contract', 'verified', 'contract', 1, $3, $3, 0)")
        .bind(context).bind(user.as_str()).bind(now - Duration::minutes(1)).execute(pool).await.expect("insert conversation fact");
    sqlx::query("INSERT INTO conversation_analyses (context_id, user_id, status, trigger, title) VALUES ($1, $2, 'classified', 'automatic', 'previous verdict')")
        .bind(context).bind(user.as_str()).execute(pool).await.expect("insert existing verdict");
}

#[tokio::test(flavor = "multi_thread")]
async fn bulk_judge_requeues_only_selected_contexts_in_the_requested_scope() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let inside_user = credentials.non_admin_user_id.clone();
    let admin_user_id = credentials.admin_user_id.clone();
    let inside = uuid::Uuid::new_v4().to_string();
    let unselected_inside = uuid::Uuid::new_v4().to_string();
    let outside = uuid::Uuid::new_v4().to_string();
    insert_fact(&db.pool, &inside, &inside_user).await;
    insert_fact(&db.pool, &unselected_inside, &inside_user).await;
    sqlx::query(
        "UPDATE conversation_facts SET group_id = 'contract-group' WHERE context_id = ANY($1)",
    )
    .bind(vec![inside.clone(), unselected_inside.clone()])
    .execute(&*db.pool)
    .await
    .expect("attribute selected facts to the requested group");
    let outside_user = seed::insert_user(
        &db.pool,
        &seed::unique("judge-outside"),
        "judge-outside@contract.test",
    )
    .await;
    insert_fact(&db.pool, &outside, &outside_user).await;
    let app = App::new(&db.pool, credentials);
    let form_body = format!(
        "ids={inside}%2C{outside}&query=group%3Dcontract-group&back=https%3A%2F%2Fevil.invalid"
    );
    let request = Call {
        method: "post",
        path: "/admin/analysis/conversations/judge",
        principal: Principal::Admin,
        content_type: Some("application/x-www-form-urlencoded"),
        body: Some(&form_body),
    };
    let request_origin = origin();
    let (status, headers) = app
        .response_headers_with(request, &[("origin", &request_origin)])
        .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert_eq!(
        headers.location.as_deref(),
        Some("/admin/analysis/conversations"),
        "untrusted back URL is ignored"
    );
    let rows: Vec<(String, String, String, Option<String>)> = sqlx::query_as("SELECT context_id, status, trigger, requested_by FROM conversation_analyses WHERE context_id = ANY($1) ORDER BY context_id")
        .bind(vec![inside.clone(), unselected_inside.clone(), outside.clone()]).fetch_all(&*db.pool).await.expect("read judge rows");
    assert_eq!(rows.len(), 3);
    let inside_row = rows.iter().find(|row| row.0 == inside).expect("scoped row");
    assert_eq!(inside_row.1, "pending");
    assert_eq!(inside_row.2, "manual");
    assert_eq!(inside_row.3.as_deref(), Some(admin_user_id.as_str()));
    let outside_row = rows
        .iter()
        .find(|row| row.0 == outside)
        .expect("outside row");
    assert_eq!(
        outside_row.1, "classified",
        "out-of-scope selection is not requeued"
    );
    assert_eq!(outside_row.2, "automatic");
    let unselected_row = rows
        .iter()
        .find(|row| row.0 == unselected_inside)
        .expect("unselected scoped row");
    assert_eq!(
        unselected_row.1, "classified",
        "a selected-only request does not requeue nearby scoped rows"
    );
    assert_eq!(unselected_row.2, "automatic");
    db.cleanup().await;
}
