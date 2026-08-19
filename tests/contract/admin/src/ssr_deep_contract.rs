//! The server-rendered pages, driven against a database that has rows in it.
//!
//! [`crate::handler_variants`] runs every page over a database seeded with
//! nothing but the two principals, which pins each template's *empty* branch.
//! That is half the contract. The other half — the branch that renders a row —
//! is where the joins, the formatters, and the per-row partials live, and a
//! page can render its empty state perfectly while the populated one has been
//! broken since a column was renamed.
//!
//! Two shapes of case:
//!
//! - **Detail pages**, which are a lookup and therefore have exactly two
//!   outcomes. A seeded id must render and carry that id in the body; an id
//!   that matches nothing must be a `404` naming what was not found, not a
//!   `500` and not a blank page that reads as "this record was deleted".
//! - **List pages**, driven again now that their lists are non-empty, so the
//!   row markup is asserted rather than the "nothing here" message.

use axum::http::StatusCode;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

// Everything one seeded activity trail is keyed on. The pages cross-link, so
// the same session must carry the requests, the decisions, the context, and
// the hook events for the joins to have anything to join.
struct Trail {
    user_id: UserId,
    session_id: String,
    context_id: String,
    trace_id: String,
    request_id: String,
}

async fn seed_trail(pool: &PgPool) -> Trail {
    let user_id_str = seed::unique("trail-user");
    let user_id =
        seed::insert_user(pool, &user_id_str, &format!("{user_id_str}@contract.test")).await;

    let session_id = seed::unique("trail-session");
    seed::insert_session(pool, &session_id, &user_id).await;

    // `ContextId` parses as a UUID and the detail page rejects anything else
    // before it reaches the database, so this id cannot use the `unique`
    // prefix form the others do.
    let context_id = uuid::Uuid::new_v4().to_string();
    seed::insert_context(
        pool,
        &context_id,
        &user_id,
        Some(&session_id),
        "Contract conversation",
    )
    .await;

    let trace_id = seed::unique("trail-trace");
    let request_id = seed::unique("trail-request");
    seed::insert_request(
        pool,
        &seed::RequestSpec {
            id: request_id.clone(),
            user_id: &user_id,
            session_id: Some(&session_id),
            trace_id: Some(&trace_id),
            context_id: Some(context_id.as_str()),
            status: "completed",
        },
    )
    .await;
    // A second request in the same trail, failed, so the error-count and
    // failed-status branches of every rollup have something to count.
    seed::insert_request(
        pool,
        &seed::RequestSpec {
            id: seed::unique("trail-request-failed"),
            user_id: &user_id,
            session_id: Some(&session_id),
            trace_id: Some(&trace_id),
            context_id: Some(context_id.as_str()),
            status: "error",
        },
    )
    .await;

    seed::insert_decision(
        pool,
        &seed::DecisionSpec {
            id: seed::unique("trail-allow"),
            user_id: &user_id,
            session_id: &session_id,
            context_id: &context_id,
            decision: "allow",
            policy: "scope_check",
            tool_name: "Read",
        },
    )
    .await;
    seed::insert_decision(
        pool,
        &seed::DecisionSpec {
            id: seed::unique("trail-deny"),
            user_id: &user_id,
            session_id: &session_id,
            context_id: &context_id,
            decision: "deny",
            policy: "blocklist",
            tool_name: "Bash",
        },
    )
    .await;

    seed::insert_summary(pool, &session_id, &user_id, "Contract session").await;
    seed::insert_event(pool, &user_id, &session_id, "Read").await;
    seed::insert_event(pool, &user_id, &session_id, "Bash").await;

    Trail {
        user_id,
        session_id,
        context_id,
        trace_id,
        request_id,
    }
}

// A detail page for a record that exists renders it; one for a record that
// does not is a 404 that says so.
#[tokio::test(flavor = "multi_thread")]
async fn seeded_detail_pages_render_the_record_and_miss_cleanly() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping seeded SSR suite");
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let trail = seed_trail(&db.pool).await;

    // (label, path, a substring the rendered page must carry)
    let found: [(&str, String, String); 6] = [
        (
            "the session detail page",
            format!("/admin/entities/sessions/{}", trail.session_id),
            trail.session_id.clone(),
        ),
        (
            "the context detail page",
            format!("/admin/entities/contexts/{}", trail.context_id),
            trail.context_id.clone(),
        ),
        (
            "the trace detail page, addressed by trace id",
            format!("/admin/entities/traces/{}", trail.trace_id),
            "Waterfall".to_owned(),
        ),
        // The same page resolves a session id too — a caller holding either
        // half of the pair must land somewhere useful.
        (
            "the trace detail page, addressed by session id",
            format!("/admin/entities/traces/{}", trail.session_id),
            "Waterfall".to_owned(),
        ),
        (
            "the governance audit chain for a request",
            format!("/admin/entities/requests/{}", trail.request_id),
            "Policy chain".to_owned(),
        ),
        (
            "the per-user page",
            format!("/admin/access/user?id={}", trail.user_id.as_str()),
            trail.user_id.as_str().to_owned(),
        ),
    ];

    let mut failures = Vec::new();
    for (label, path, marker) in found {
        let (status, body) = app.call(Call::get(&path, Principal::Admin)).await;
        if status != StatusCode::OK {
            failures.push(format!(
                "  {label} -> {} (expected 200): {}",
                status.as_u16(),
                body.chars().take(240).collect::<String>()
            ));
            continue;
        }
        if !body.contains(&marker) {
            failures.push(format!(
                "  {label} -> 200 but the body never carried {marker:?} — the page rendered \
                 without the record it was asked for"
            ));
        }
    }

    // The miss half. A detail page for an id in no table owes a 404: a 200 with
    // an empty shell is indistinguishable from a record that was deleted.
    let missing: [(&str, String); 5] = [
        (
            "a session id in no table",
            "/admin/entities/sessions/no-such-session".to_owned(),
        ),
        (
            "a context id that is a well-formed UUID but matches nothing",
            format!("/admin/entities/contexts/{}", uuid::Uuid::new_v4()),
        ),
        // The context id segment is parsed as a UUID before any query runs, so
        // a non-UUID is a miss rather than a parser panic.
        (
            "a context id that is not a UUID at all",
            "/admin/entities/contexts/not-a-uuid".to_owned(),
        ),
        (
            "a trace id in no table",
            "/admin/entities/traces/no-such-trace".to_owned(),
        ),
        (
            "a request id in no table",
            "/admin/entities/requests/no-such-request".to_owned(),
        ),
    ];
    for (label, path) in missing {
        let (status, body) = app.call(Call::get(&path, Principal::Admin)).await;
        if status != StatusCode::NOT_FOUND {
            failures.push(format!(
                "  {label} -> {} (expected 404): {}",
                status.as_u16(),
                body.chars().take(200).collect::<String>()
            ));
        }
    }

    // The same pages under a non-admin principal are refused rather than
    // rendered — these carry another customer's conversation content.
    for path in [
        format!("/admin/entities/sessions/{}", trail.session_id),
        format!("/admin/entities/contexts/{}", trail.context_id),
        format!("/admin/entities/traces/{}", trail.trace_id),
        format!("/admin/entities/requests/{}", trail.request_id),
    ] {
        let (status, _) = app.call(Call::get(&path, Principal::NonAdmin)).await;
        if !(status == StatusCode::FORBIDDEN || status.is_redirection()) {
            failures.push(format!(
                "  {path} as a non-admin -> {} (expected a refusal)",
                status.as_u16()
            ));
        }
    }

    db.cleanup().await;
    assert!(
        failures.is_empty(),
        "{} seeded detail-page case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// The list pages, re-driven now that their lists have rows.
#[tokio::test(flavor = "multi_thread")]
async fn seeded_list_pages_render_rows_rather_than_the_empty_state() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let trail = seed_trail(&db.pool).await;

    // (label, path, marker the populated branch emits, marker it must NOT emit)
    let cases: [(&str, String, String, Option<&str>); 10] = [
        (
            "the trace explorer",
            "/admin/entities/traces".to_owned(),
            trail.session_id.clone(),
            Some("No traces in the selected window."),
        ),
        (
            "the trace explorer filtered to denials",
            "/admin/entities/traces?deny_only=true".to_owned(),
            trail.session_id.clone(),
            None,
        ),
        (
            "the trace explorer filtered to errors",
            "/admin/entities/traces?error_only=true".to_owned(),
            trail.session_id.clone(),
            None,
        ),
        (
            "the trace explorer filtered by policy and decision",
            "/admin/entities/traces?policy=blocklist&decision=deny".to_owned(),
            trail.session_id.clone(),
            None,
        ),
        (
            "the trace explorer sorted by cost",
            "/admin/entities/traces?sort=cost&dir=asc".to_owned(),
            trail.session_id.clone(),
            None,
        ),
        (
            "the contexts list",
            "/admin/entities/contexts".to_owned(),
            "Contract conversation".to_owned(),
            Some("No conversation contexts match your filters."),
        ),
        (
            "the contexts list grouped by user",
            "/admin/entities/contexts?view=users".to_owned(),
            trail.user_id.as_str().to_owned(),
            Some("No users with conversation contexts match your filters."),
        ),
        (
            "the contexts list searched for the seeded name",
            "/admin/entities/contexts?q=Contract".to_owned(),
            "Contract conversation".to_owned(),
            None,
        ),
        // `/entities/sessions` is the signed-in principal's own session page,
        // not a roster, so it is asserted on the viewer rather than the trail.
        (
            "the current-session page",
            "/admin/entities/sessions".to_owned(),
            "contract-admin@contract.test".to_owned(),
            None,
        ),
        (
            "the roster",
            "/admin/access/users".to_owned(),
            trail.user_id.as_str().to_owned(),
            None,
        ),
    ];

    let mut failures = Vec::new();
    for (label, path, marker, forbidden) in cases {
        let (status, body) = app.call(Call::get(&path, Principal::Admin)).await;
        if status != StatusCode::OK {
            failures.push(format!(
                "  {label} -> {} (expected 200): {}",
                status.as_u16(),
                body.chars().take(240).collect::<String>()
            ));
            continue;
        }
        if !body.contains(&marker) {
            failures.push(format!(
                "  {label} -> 200 but never rendered {marker:?}, so the seeded row did not \
                 reach the template"
            ));
        }
        if let Some(empty_message) = forbidden
            && body.contains(empty_message)
        {
            failures.push(format!(
                "  {label} -> rendered the empty state {empty_message:?} despite having rows"
            ));
        }
    }

    db.cleanup().await;
    assert!(
        failures.is_empty(),
        "{} seeded list-page case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// The department pages, whose content comes from the org tables rather than
// the activity spine.
#[tokio::test(flavor = "multi_thread")]
async fn department_pages_render_the_seeded_department() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let mut failures = Vec::new();

    // Migration `009` seeds the `Default` department, so the management pages
    // have a row without the fixture writing one.
    let dept: Option<String> = sqlx::query_scalar("SELECT id FROM departments LIMIT 1")
        .fetch_optional(&*db.pool)
        .await
        .expect("read a seeded department");

    let (status, body) = app
        .call(Call::get("/admin/access/departments", Principal::Admin))
        .await;
    if status != StatusCode::OK {
        failures.push(format!("  the departments page -> {}", status.as_u16()));
    } else if !body.contains("Departments") {
        failures.push("  the departments page rendered without its heading".to_owned());
    }

    if let Some(id) = dept {
        let path = format!("/admin/access/departments/{id}");
        let (status, body) = app.call(Call::get(&path, Principal::Admin)).await;
        if status != StatusCode::OK {
            failures.push(format!(
                "  {path} -> {} (expected 200): {}",
                status.as_u16(),
                body.chars().take(200).collect::<String>()
            ));
        } else if !body.contains("Members") {
            failures.push(format!("  {path} rendered without the members panel"));
        }
    }

    // A department id in no table is a miss, not a rendered shell.
    let (status, _) = app
        .call(Call::get(
            "/admin/access/departments/no-such-department",
            Principal::Admin,
        ))
        .await;
    if status != StatusCode::NOT_FOUND {
        failures.push(format!(
            "  an unknown department id -> {} (expected 404)",
            status.as_u16()
        ));
    }

    db.cleanup().await;
    assert!(
        failures.is_empty(),
        "{} department page case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// The analytics pages, driven over a window that contains the seeded trail.
#[tokio::test(flavor = "multi_thread")]
async fn analytics_pages_aggregate_the_seeded_trail() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let trail = seed_trail(&db.pool).await;

    let mut failures = Vec::new();
    let paths: [(&str, String); 8] = [
        (
            "the requests log filtered to the seeded model",
            "/admin/entities/requests?tab=log&model=claude-contract-model".to_owned(),
        ),
        (
            "the requests log filtered to failures",
            "/admin/entities/requests?tab=log&status=error".to_owned(),
        ),
        (
            "the requests log searched for the seeded session",
            format!("/admin/entities/requests?tab=log&q={}", trail.session_id),
        ),
        (
            "the model breakdown",
            "/admin/entities/requests?tab=models".to_owned(),
        ),
        (
            "the provider breakdown",
            "/admin/entities/requests?tab=providers".to_owned(),
        ),
        (
            "the outcome mix",
            "/admin/entities/requests?tab=status".to_owned(),
        ),
        (
            "the governance decisions log",
            "/admin/governance/decisions".to_owned(),
        ),
        (
            "the governance hook activity page",
            "/admin/governance/hooks".to_owned(),
        ),
    ];
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

    // The one assertion that proves an aggregate ran rather than merely
    // rendering: the seeded model must appear in the model breakdown.
    let (_, body) = app
        .call(Call::get(
            "/admin/entities/requests?tab=models",
            Principal::Admin,
        ))
        .await;
    if !body.contains("claude-contract-model") {
        failures.push(
            "  the model breakdown never named the seeded model — the rollup query \
             returned nothing"
                .to_owned(),
        );
    }

    db.cleanup().await;
    assert!(
        failures.is_empty(),
        "{} analytics page case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
