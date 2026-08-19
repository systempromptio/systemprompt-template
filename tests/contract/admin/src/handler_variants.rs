//! Query-parameter coverage for the server-rendered admin pages.
//!
//! [`crate::status_contract`] drives every route once, with no query string,
//! and asserts the properties that hold for all of them. That leaves the
//! branching *inside* a page untested: a list page is a tab selector, a filter
//! set, a sort order, and a pager, and each of those is a separate query the
//! handler builds and a separate block the template renders. A page can answer
//! 200 on its default view for years while its `?tab=providers` arm has been
//! broken since a refactor.
//!
//! Each case therefore asserts two things: the status, and a marker string that
//! only the intended branch emits. The marker is what makes the case a test
//! rather than a smoke check — `200` proves the handler did not panic, the
//! marker proves it rendered the thing the query asked for.
//!
//! Markers are drawn from the templates in `storage/files/admin/templates/`
//! and their partials, so a template edit that drops a branch fails here
//! rather than shipping a page that silently renders nothing.
//!
//! The database is the throwaway one, seeded only with the two principals, so
//! every list is empty. That is deliberate: the empty state is a rendered
//! branch like any other, and asserting its message proves the query ran and
//! returned nothing rather than erroring into an `unwrap_or_default()`.

use axum::http::StatusCode;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

// A path (query string included) and the substring its response must contain.
struct Variant {
    path: &'static str,
    marker: &'static str,
}

const fn v(path: &'static str, marker: &'static str) -> Variant {
    Variant { path, marker }
}

// Markers used by more than one case, named so a template edit has one place
// to be reflected.
const REQUESTS_TITLE: &str = "Inference Requests";
const REQUESTS_OVERVIEW: &str = "Latency distribution";
const REQUESTS_EMPTY: &str = "No inference requests match.";
// The seeded fixture puts requests inside the default window, so an unfiltered
// log page renders rows. Match the row markup rather than a count, which moves
// whenever the seed does.
const REQUESTS_LOG_ROWS: &str = r#"class="audit-row""#;
const TRACES_TITLE: &str = "Trace Explorer";
const TRACES_EMPTY: &str = "No traces in the selected window.";
const CONTEXTS_TITLE: &str = "Stateful AI conversations";
const CONTEXTS_EMPTY: &str = "No conversation contexts match your filters.";
const CONTEXTS_USERS_EMPTY: &str = "No users with conversation contexts match your filters.";
const CUSTOMER_REPORT: &str = "— usage report";

// The requests page: five tabs, a time window, four filter chips, a sort, and
// a pager — the densest branching in the admin plane.
const REQUESTS: [Variant; 22] = [
    v("/admin/entities/requests", REQUESTS_OVERVIEW),
    v("/admin/entities/requests?tab=overview", REQUESTS_OVERVIEW),
    v("/admin/entities/requests?tab=overview", "Pre-flight denies"),
    v(
        "/admin/entities/requests?tab=models",
        "attributed to the model that produced them.",
    ),
    v(
        "/admin/entities/requests?tab=providers",
        "rolled up to the upstream provider.",
    ),
    v(
        "/admin/entities/requests?tab=status",
        "Outcome mix for the window.",
    ),
    // An unrecognised tab falls back to the overview rather than rendering a
    // page with every section switched off.
    v("/admin/entities/requests?tab=nonsense", REQUESTS_OVERVIEW),
    v("/admin/entities/requests?tab=log", REQUESTS_LOG_ROWS),
    v("/admin/entities/requests?tab=log&page=2", REQUESTS_EMPTY),
    v(
        "/admin/entities/requests?tab=log&page=100000",
        REQUESTS_EMPTY,
    ),
    // A negative page is clamped to the first, not turned into a negative
    // OFFSET the database would reject.
    v(
        "/admin/entities/requests?tab=log&page=-5",
        REQUESTS_LOG_ROWS,
    ),
    v(
        "/admin/entities/requests?tab=log&model=claude-opus-5",
        r#"filter-ribbon__chip-value">claude-opus-5</span>"#,
    ),
    v(
        "/admin/entities/requests?tab=log&provider=anthropic",
        r#"filter-ribbon__chip-value">anthropic</span>"#,
    ),
    v(
        "/admin/entities/requests?tab=log&status=error",
        r#"filter-ribbon__chip-value">error</span>"#,
    ),
    v(
        "/admin/entities/requests?tab=log&q=needle-xyz",
        r#"filter-ribbon__chip-value">needle-xyz</span>"#,
    ),
    v(
        "/admin/entities/requests?preset=15m",
        r#"data-preset="15m" aria-current="true""#,
    ),
    v(
        "/admin/entities/requests?preset=7d",
        r#"data-preset="7d" aria-current="true""#,
    ),
    v(
        "/admin/entities/requests?preset=30d",
        r#"data-preset="30d" aria-current="true""#,
    ),
    v("/admin/entities/requests?preset=nonsense", REQUESTS_TITLE),
    v(
        "/admin/entities/requests?from=2026-01-01T00:00:00Z&to=2026-02-01T00:00:00Z",
        REQUESTS_TITLE,
    ),
    v(
        "/admin/entities/requests?tab=log&sort=cost&dir=asc",
        REQUESTS_LOG_ROWS,
    ),
    v(
        "/admin/entities/requests?tab=log&sort=nonsense&dir=nonsense",
        REQUESTS_LOG_ROWS,
    ),
];

// The trace explorer: the same window and pager, plus the two stat tiles that
// double as filters.
const TRACES: [Variant; 11] = [
    v("/admin/entities/traces", TRACES_EMPTY),
    v(
        "/admin/entities/traces?preset=7d",
        r#"data-preset="7d" aria-current="true""#,
    ),
    v("/admin/entities/traces?page=3", TRACES_EMPTY),
    v("/admin/entities/traces?page=99999", TRACES_EMPTY),
    v("/admin/entities/traces?page=-2", TRACES_EMPTY),
    // `deny_only` / `error_only` mark their tile as the applied filter; the
    // tile is the only thing on the page that says so.
    v("/admin/entities/traces?deny_only=true", "is-active"),
    v("/admin/entities/traces?error_only=true", "is-active"),
    v("/admin/entities/traces?sort=cost&dir=asc", TRACES_EMPTY),
    v(
        "/admin/entities/traces?policy=blocklist&decision=deny",
        TRACES_EMPTY,
    ),
    v(
        "/admin/entities/traces?agent_scope=global&agent_id=no-such-agent",
        TRACES_EMPTY,
    ),
    // An unparseable custom window falls back to the default one and still
    // renders the page rather than refusing it.
    v(
        "/admin/entities/traces?from=not-a-date&to=not-a-date",
        TRACES_TITLE,
    ),
];

// The contexts page: a two-way view switch, a search box, a since-window pill
// group, and a row limit.
const CONTEXTS: [Variant; 9] = [
    v("/admin/entities/contexts", CONTEXTS_EMPTY),
    v("/admin/entities/contexts?view=contexts", CONTEXTS_EMPTY),
    v("/admin/entities/contexts?view=users", CONTEXTS_USERS_EMPTY),
    v(
        "/admin/entities/contexts?q=needle-xyz",
        r#"value="needle-xyz""#,
    ),
    v("/admin/entities/contexts?since=7d", r#"value="7d" checked"#),
    v(
        "/admin/entities/contexts?since=30d",
        r#"value="30d" checked"#,
    ),
    v("/admin/entities/contexts?since=nonsense", CONTEXTS_TITLE),
    v("/admin/entities/contexts?limit=1", CONTEXTS_EMPTY),
    v("/admin/entities/contexts?limit=100000", CONTEXTS_EMPTY),
];

// The roster, the per-user page, the customer report, and the catalog.
//
// The roster is the one list with rows: the suite seeds two principals, so it
// renders the table rather than the empty state, and the seeded address is the
// marker that proves a row reached the template.
const PAGES: [Variant; 12] = [
    v("/admin/access/users", "contract-admin@contract.test"),
    v(
        "/admin/access/users?unknown_param=1&page=99",
        "contract-admin@contract.test",
    ),
    // No `id` renders the same blank page as an id that matches nothing.
    v("/admin/access/user", "User not found."),
    v("/admin/access/user?id=no-such-user", "User not found."),
    v("/admin/user?id=no-such-user", "User not found."),
    v("/admin/reports/customer", CUSTOMER_REPORT),
    v("/admin/reports/customer?month=2023-03", "March 2023"),
    v("/admin/reports/customer?month=2024-07", "July 2024"),
    // An unparseable month falls back to the last complete one.
    v("/admin/reports/customer?month=nonsense", CUSTOMER_REPORT),
    v(
        "/admin/catalog/plugins",
        "Plugins are the installable units in the catalog",
    ),
    v(
        "/admin/catalog/skills",
        "Skills are reusable instruction sets",
    ),
    v("/admin/catalog/mcp", "MCP servers expose tools to agents"),
];

#[tokio::test(flavor = "multi_thread")]
async fn admin_pages_render_the_branch_their_query_selects() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping admin handler-variant suite");
        return;
    };

    let credentials = principal::provision(&db.pool).await;

    // The requests-log rows the `tab=log` variants assert on. One row keeps
    // page 1 populated and page 2 empty, which is exactly what the variant
    // table expects.
    let request_user = seed::insert_user(
        &db.pool,
        &seed::unique("variant-user"),
        "variant-user@contract.test",
    )
    .await;
    seed::insert_request(
        &db.pool,
        &seed::RequestSpec {
            id: seed::unique("variant-request"),
            user_id: &request_user,
            session_id: None,
            trace_id: None,
            context_id: None,
            status: "completed",
        },
    )
    .await;

    let app = App::new(&db.pool, credentials);

    let mut failures = Vec::new();
    for variant in REQUESTS
        .iter()
        .chain(TRACES.iter())
        .chain(CONTEXTS.iter())
        .chain(PAGES.iter())
    {
        let (status, body) = app.call(Call::get(variant.path, Principal::Admin)).await;

        if status != StatusCode::OK {
            failures.push(format!(
                "  {} -> {} (expected 200){}",
                variant.path,
                status.as_u16(),
                snippet(&body)
            ));
            continue;
        }
        if !body.contains(variant.marker) {
            failures.push(format!(
                "  {} -> 200 but the body never contained {:?} — the query selected a \
                 different branch than the one it names{}",
                variant.path,
                variant.marker,
                snippet(&body)
            ));
        }
    }

    db.cleanup().await;

    assert!(
        failures.is_empty(),
        "{} page variant(s) rendered the wrong thing:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// Enough of the body to tell which page came back, without pasting a whole
// rendered document into the failure.
fn snippet(body: &str) -> String {
    let head: String = body.chars().take(300).collect();
    format!("\n      body: {head}")
}
