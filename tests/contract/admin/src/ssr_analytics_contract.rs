//! `/admin/users/{user_id}` and the report exports.
//!
//! Both are addressed by something [`crate::status_contract`] cannot supply:
//! the per-user page takes a path parameter, and the CSV routes answer with a
//! file rather than a page. So neither's populated branch is driven anywhere
//! else, and the whole person-shaped view — the trend, the model mix, the
//! daily records, the session history — renders only when a real user id
//! resolves to a real person with real traffic.
//!
//! The time range is the other axis: every panel is built from a window, and
//! the page accepts both a named preset and an explicit `from`/`to` pair. A
//! range that parses to nothing is the branch that silently renders an empty
//! page instead of the range that was asked for.

use axum::http::StatusCode;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

// A person with enough traffic that every panel on the page has something to
// render: two requests so the trend has a shape, one of them failed so the
// error counts are non-zero, and a session to hang the cost history on.
async fn seed_person(pool: &PgPool) -> UserId {
    let id = seed::unique("analytics-user");
    let user_id = seed::insert_user(pool, &id, &format!("{id}@contract.test")).await;

    let session_id = seed::unique("analytics-session");
    seed::insert_session(pool, &session_id, &user_id).await;

    seed::insert_request(
        pool,
        &seed::RequestSpec {
            id: seed::unique("analytics-request"),
            user_id: &user_id,
            session_id: Some(&session_id),
            trace_id: None,
            context_id: None,
            status: "completed",
        },
    )
    .await;
    seed::insert_request(
        pool,
        &seed::RequestSpec {
            id: seed::unique("analytics-request-failed"),
            user_id: &user_id,
            session_id: Some(&session_id),
            trace_id: None,
            context_id: None,
            status: "failed",
        },
    )
    .await;
    seed::insert_summary(pool, &session_id, &user_id, "Analytics session").await;
    seed::insert_event(pool, &user_id, &session_id, "Edit").await;

    user_id
}

#[tokio::test(flavor = "multi_thread")]
async fn the_per_user_analytics_page_renders_that_person_over_several_windows() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping the analytics SSR suite");
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let user_id = seed_person(&db.pool).await;

    // The window is what every panel is built from, so each form of asking for
    // one is a separate path through the handler.
    let windows = [
        ("the default window", String::new()),
        ("a named preset", "?preset=7d".to_owned()),
        ("a longer preset", "?preset=30d".to_owned()),
        (
            "an explicit from/to pair",
            "?preset=custom&from=2026-08-01&to=2026-09-01".to_owned(),
        ),
        // A range that cannot be parsed must fall back to a window rather than
        // rendering a page with no data in it and no explanation.
        ("an unparseable range", "?from=not-a-date".to_owned()),
    ];

    let mut failures = Vec::new();
    for (label, query) in windows {
        let path = format!("/admin/users/{}{query}", user_id.as_str());
        let (status, body) = app.call(Call::get(&path, Principal::Admin)).await;

        if status != StatusCode::OK {
            failures.push(format!(
                "  {label} -> {} (expected 200): {}",
                status.as_u16(),
                body.chars().take(240).collect::<String>()
            ));
            continue;
        }
        if !body.contains(user_id.as_str()) {
            failures.push(format!(
                "  {label} rendered a page that does not name the person it is about"
            ));
        }
    }

    assert!(failures.is_empty(), "\n{}", failures.join("\n"));

    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_user_id_that_matches_nobody_is_a_404_rather_than_a_blank_page() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (status, body) = app
        .call(Call::get(
            "/admin/users/nobody-by-that-name",
            Principal::Admin,
        ))
        .await;

    // Why: an empty analytics page for a user who does not exist reads as
    // "this person did nothing", which is a different and much worse claim
    // than "there is no such person".
    assert_eq!(status, StatusCode::NOT_FOUND, "body: {body}");
    assert!(
        body.contains("User not found") || body.contains("404"),
        "the page says what was not found: {body}"
    );

    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn one_persons_analytics_are_not_readable_by_another_user() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let user_id = seed_person(&db.pool).await;
    let path = format!("/admin/users/{}", user_id.as_str());

    let (non_admin, _) = app.call(Call::get(&path, Principal::NonAdmin)).await;
    let (anonymous, _) = app.call(Call::get(&path, Principal::Anonymous)).await;

    // Why: the admin-only layer turns a non-admin away before the handler
    // runs, so the refusal is a redirect rather than the handler's own 403 —
    // what matters is that neither caller receives the page.
    assert_eq!(
        non_admin,
        StatusCode::SEE_OTHER,
        "a signed-in non-admin is sent away from someone else's analytics"
    );
    assert_eq!(
        anonymous,
        StatusCode::TEMPORARY_REDIRECT,
        "an anonymous caller is sent to sign in"
    );

    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_report_exports_answer_with_a_csv_body() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    seed_person(&db.pool).await;

    let mut failures = Vec::new();
    for path in [
        "/admin/reports/customer.csv",
        "/admin/reports/customer.csv?preset=30d",
        "/admin/reports/internal.csv",
        "/admin/analytics/cost.csv?audience=internal&preset=30d",
        "/admin/analytics/cost.csv?audience=customer&axis=project&preset=30d",
    ] {
        let (status, body) = app.call(Call::get(path, Principal::Admin)).await;
        if status != StatusCode::OK {
            failures.push(format!("  {path} -> {}", status.as_u16()));
            continue;
        }
        // A CSV is a header row and then rows: the first line has to carry
        // separators, and the body must not be the HTML error page.
        let first_line = body.lines().next().unwrap_or_default();
        if !first_line.contains(',') {
            failures.push(format!("  {path} has no header row: {first_line}"));
        }
        if body.starts_with("<!DOCTYPE html>") {
            failures.push(format!("  {path} answered with a page, not a file"));
        }
    }

    assert!(failures.is_empty(), "\n{}", failures.join("\n"));

    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn every_dashboard_tab_renders_over_seeded_traffic() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let user_id = seed_person(&db.pool).await;

    // Each tab is a different set of queries and a different half of the
    // template; the filters below are the axes the page re-queries on.
    let cases = [
        ("overview", "?tab=overview".to_owned()),
        ("models", "?tab=models".to_owned()),
        ("skills", "?tab=skills".to_owned()),
        ("tools", "?tab=tools".to_owned()),
        ("sessions", "?tab=sessions".to_owned()),
        ("cost", "?tab=cost".to_owned()),
        ("an unknown tab falls back", "?tab=nonsense".to_owned()),
        ("hourly buckets", "?tab=overview&bucket=hour".to_owned()),
        ("a latency objective", "?tab=overview&slo_ms=250".to_owned()),
        (
            "a sorted, paged leaderboard",
            "?tab=overview&sort=cost&page=2".to_owned(),
        ),
        (
            "a project filter",
            "?tab=overview&project=commerce".to_owned(),
        ),
        (
            "member attribution, which overlaps on purpose",
            "?tab=overview&attr=member".to_owned(),
        ),
        (
            "the customer half of the cost tab",
            "?tab=cost&audience=customer&axis=project".to_owned(),
        ),
        (
            "one person's slice of the site view",
            format!("?tab=overview&user_id={}", user_id.as_str()),
        ),
    ];

    let mut failures = Vec::new();
    for (label, query) in cases {
        let path = format!("/admin/analytics{query}");
        let (status, body) = app.call(Call::get(&path, Principal::Admin)).await;
        if status != StatusCode::OK {
            failures.push(format!(
                "  {label} -> {} : {}",
                status.as_u16(),
                body.chars().take(240).collect::<String>()
            ));
        } else if !body.starts_with("<!DOCTYPE html>") {
            failures.push(format!("  {label} did not render a page"));
        }
    }

    assert!(failures.is_empty(), "\n{}", failures.join("\n"));

    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn the_history_page_and_its_search_endpoint_run_over_a_seeded_session() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let user_id = seed_person(&db.pool).await;

    let cases = [
        ("the unfiltered page", "/admin/history".to_owned()),
        ("a full-text query", "/admin/history?q=Analytics".to_owned()),
        (
            "a query that matches nothing",
            "/admin/history?q=zzzzznomatch".to_owned(),
        ),
        ("a page past the end", "/admin/history?page=99".to_owned()),
        (
            "the search endpoint the page calls",
            "/admin/api/history/search?q=Analytics".to_owned(),
        ),
        ("the org-wide listing", "/admin/conversations".to_owned()),
        (
            "one person's conversations",
            format!("/admin/conversations?user_id={}", user_id.as_str()),
        ),
        (
            "the org-wide listing with side calls",
            "/admin/conversations?side=1".to_owned(),
        ),
    ];

    let mut failures = Vec::new();
    for (label, path) in cases {
        let (status, body) = app.call(Call::get(&path, Principal::Admin)).await;
        if status != StatusCode::OK {
            failures.push(format!(
                "  {label} -> {} : {}",
                status.as_u16(),
                body.chars().take(240).collect::<String>()
            ));
        }
    }

    // Why: "My conversations" means the viewer's own, admin or not. Asking it
    // for somebody else is out of scope rather than merely empty — the
    // org-wide listing above is where an admin reads another account.
    let (status, _) = app
        .call(Call::get(
            &format!("/admin/history?user_id={}", user_id.as_str()),
            Principal::Admin,
        ))
        .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "/admin/history must not widen to another user for an admin"
    );

    assert!(failures.is_empty(), "\n{}", failures.join("\n"));

    db.cleanup().await;
}
