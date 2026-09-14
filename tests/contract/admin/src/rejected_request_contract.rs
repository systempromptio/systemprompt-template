//! The pages that read `ai_requests`, driven over a gateway-rejected row.
//!
//! A request the gateway refuses before routing never acquires a provider or a
//! model, and the table's CHECK constraint permits exactly that shape for
//! `status = 'rejected'`. Every reader that forces those two columns non-null
//! decodes such a row into an error, so the page 500s for the whole session —
//! not just for the refused request, which is the part that makes it a
//! reporting failure rather than a cosmetic one: the operator investigating a
//! refusal is precisely who cannot load the page.
//!
//! [`crate::ssr_deep_contract`] seeds only routed requests, so it cannot see
//! this. These cases seed a session whose only request was refused.

use axum::http::StatusCode;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::app::{ADMIN_API_PREFIX, App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

struct RejectedTrail {
    user_id: UserId,
    session_id: String,
    context_id: String,
    trace_id: String,
    request_id: String,
}

async fn seed_rejected_trail(pool: &PgPool) -> RejectedTrail {
    let user_id_str = seed::unique("rejected-user");
    let user_id =
        seed::insert_user(pool, &user_id_str, &format!("{user_id_str}@contract.test")).await;

    let session_id = seed::unique("rejected-session");
    seed::insert_session(pool, &session_id, &user_id).await;

    let context_id = uuid::Uuid::new_v4().to_string();
    seed::insert_context(
        pool,
        &context_id,
        &user_id,
        Some(&session_id),
        "Refused conversation",
    )
    .await;

    let trace_id = seed::unique("rejected-trace");
    let request_id = seed::unique("rejected-request");
    seed::insert_rejected_request(
        pool,
        &seed::RequestSpec {
            id: request_id.clone(),
            user_id: &user_id,
            session_id: Some(&session_id),
            trace_id: Some(&trace_id),
            context_id: Some(&context_id),
            status: "rejected",
        },
    )
    .await;

    seed::insert_decision(
        pool,
        &seed::DecisionSpec {
            id: seed::unique("rejected-decision"),
            user_id: &user_id,
            session_id: &session_id,
            decision: "deny",
            policy: "blocklist",
            tool_name: "Bash",
        },
    )
    .await;

    RejectedTrail {
        user_id,
        session_id,
        context_id,
        trace_id,
        request_id,
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_rejected_request_renders_everywhere_it_is_read() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping rejected-request suite");
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let trail = seed_rejected_trail(&db.pool).await;

    let paths: [(&str, String); 8] = [
        (
            "the trace detail page, addressed by trace id",
            format!("/admin/traces/{}", trail.trace_id),
        ),
        (
            "the trace detail page, addressed by session id",
            format!("/admin/traces/{}", trail.session_id),
        ),
        ("the trace list", "/admin/traces".to_owned()),
        (
            "the session detail page",
            format!("/admin/sessions/{}", trail.session_id),
        ),
        (
            "the context detail page",
            format!("/admin/contexts/{}", trail.context_id),
        ),
        (
            "the governance audit chain",
            format!("/admin/requests/{}", trail.request_id),
        ),
        ("the request log", "/admin/requests".to_owned()),
        (
            "the per-user usage endpoint",
            format!("{ADMIN_API_PREFIX}/users/{}/usage", trail.user_id.as_str()),
        ),
    ];

    let mut failures = Vec::new();
    for (label, path) in paths {
        let (status, body) = app.call(Call::get(&path, Principal::Admin)).await;
        if status != StatusCode::OK {
            failures.push(format!(
                "  {label} -> {} (expected 200): {}",
                status.as_u16(),
                body.chars().take(240).collect::<String>()
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "a session whose only AI request was rejected breaks {} page(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_trace_waterfall_marks_the_refused_span() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping rejected-request suite");
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let trail = seed_rejected_trail(&db.pool).await;

    let (status, body) = app
        .call(Call::get(
            &format!("/admin/traces/{}", trail.trace_id),
            Principal::Admin,
        ))
        .await;

    assert_eq!(status, StatusCode::OK, "trace detail: {body}");
    assert!(
        body.contains("REJECTED"),
        "the refused request's span must be badged as rejected, not left to read as a model call"
    );
}
