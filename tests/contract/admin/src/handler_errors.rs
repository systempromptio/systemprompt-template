//! The failure half of the admin HTTP contract.
//!
//! [`crate::status_contract`] drives every route once, well-formed, and proves
//! it does not 5xx. That is the happy path even when it ends in a 404 — the
//! request was still shaped exactly as the router expects. This module drives
//! the requests that are wrong on purpose:
//!
//! - a detail page asked for an id that exists in no table, which owes the
//!   caller a 404 carrying the reason, not a 500 and not a blank page that
//!   reads as "this record was deleted";
//! - a payload that is not JSON, or is JSON of the wrong shape, or arrives with
//!   no content type, each of which the extractor must refuse before the
//!   handler ever runs;
//! - a write attempted with no credentials, or with credentials that are not an
//!   admin's;
//! - a query string carrying nonsense where a number or a date was expected,
//!   which must be a client error rather than a panic inside a parser.
//!
//! Where a handler's exact code is genuinely a judgement call the case asserts
//! the weaker property — a client error, or merely "not a server error" — and
//! says so at the case. An assertion that pins a code nobody chose deliberately
//! is a change detector, not a contract.

use axum::http::StatusCode;

use crate::app::{ADMIN_API_PREFIX, App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

// What a case demands of the response.
#[derive(Clone, Copy)]
enum Expect {
    // Exactly this code, and nothing else.
    Status(StatusCode),
    // Refused, by whichever mechanism the layer uses: 401, 403, or a redirect
    // to the sign-in page. The SSR and API planes differ here by design.
    Refused,
    // Some 4xx. Used where the boundary between 400 and 422 is the extractor's
    // choice rather than the handler's.
    ClientError,
    // Only that the server did not fault. Used where the handler's behaviour
    // is not determined by reading it.
    NotServerError,
}

impl Expect {
    fn accepts(self, status: StatusCode) -> bool {
        match self {
            Self::Status(want) => status == want,
            Self::Refused => {
                matches!(status, StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN)
                    || status.is_redirection()
            },
            Self::ClientError => status.is_client_error(),
            Self::NotServerError => !status.is_server_error(),
        }
    }

    fn describe(self) -> String {
        match self {
            Self::Status(want) => want.as_u16().to_string(),
            Self::Refused => "401, 403, or a redirect".to_owned(),
            Self::ClientError => "a 4xx".to_owned(),
            Self::NotServerError => "anything but a 5xx".to_owned(),
        }
    }
}

// One deliberately wrong request, and what the plane owes in return.
struct Case {
    method: &'static str,
    path: &'static str,
    principal: Principal,
    content_type: Option<&'static str>,
    body: Option<&'static str>,
    expect: Expect,
    // A substring the response must carry. The status says a request was
    // refused; this says it was refused for the stated reason, which is what
    // stops a validation 400 from passing as an unrelated 400.
    marker: Option<&'static str>,
}

const fn get(path: &'static str, expect: Expect, marker: Option<&'static str>) -> Case {
    Case {
        method: "get",
        path,
        principal: Principal::Admin,
        content_type: None,
        body: None,
        expect,
        marker,
    }
}

const fn json(
    method: &'static str,
    path: &'static str,
    body: &'static str,
    expect: Expect,
    marker: Option<&'static str>,
) -> Case {
    Case {
        method,
        path,
        principal: Principal::Admin,
        content_type: Some("application/json"),
        body: Some(body),
        expect,
        marker,
    }
}

const fn anon(method: &'static str, path: &'static str, body: &'static str) -> Case {
    Case {
        method,
        path,
        principal: Principal::Anonymous,
        content_type: Some("application/json"),
        body: Some(body),
        expect: Expect::Refused,
        marker: None,
    }
}

const fn non_admin(method: &'static str, path: &'static str, body: &'static str) -> Case {
    Case {
        method,
        path,
        principal: Principal::NonAdmin,
        content_type: Some("application/json"),
        body: Some(body),
        expect: Expect::Refused,
        marker: None,
    }
}

const OK: StatusCode = StatusCode::OK;
const BAD_REQUEST: StatusCode = StatusCode::BAD_REQUEST;
const NOT_FOUND: StatusCode = StatusCode::NOT_FOUND;
const UNAUTHORIZED: StatusCode = StatusCode::UNAUTHORIZED;
const UNSUPPORTED_MEDIA: StatusCode = StatusCode::UNSUPPORTED_MEDIA_TYPE;
const UNPROCESSABLE: StatusCode = StatusCode::UNPROCESSABLE_ENTITY;

// Detail pages and detail endpoints handed an id that matches nothing.
//
// Each owes a 404 whose body names what was not found. A 500 here is the
// classic "the query returned no rows and the handler unwrapped it"; a 200
// with a blank page is worse, because it asserts the record was deleted.
const UNKNOWN_ID: [Case; 8] = [
    get(
        "/admin/entities/contexts/no-such-context",
        Expect::Status(NOT_FOUND),
        Some("No context, AI request, or message rows match that context id."),
    ),
    get(
        "/admin/entities/requests/no-such-request",
        Expect::Status(NOT_FOUND),
        Some("No audit chain found for that id."),
    ),
    get(
        "/admin/entities/sessions/no-such-session",
        Expect::Status(NOT_FOUND),
        Some("No AI requests, contexts, or transcript rows match that session id."),
    ),
    get(
        "/admin/entities/traces/no-such-trace",
        Expect::Status(NOT_FOUND),
        Some("No spans found for that session or trace id."),
    ),
    get(
        "/admin/access/departments/no-such-department",
        Expect::Status(NOT_FOUND),
        Some("Department not found"),
    ),
    get(
        "/api/public/admin/users/no-such-user/detail",
        Expect::Status(NOT_FOUND),
        Some("User not found"),
    ),
    get(
        "/api/public/admin/agents/no-such-agent",
        Expect::Status(NOT_FOUND),
        Some("Agent not found"),
    ),
    // Usage is a list, and an unknown user's list is empty rather than absent.
    get(
        "/api/public/admin/users/no-such-user/usage",
        Expect::Status(OK),
        Some("\"events\""),
    ),
];

// Payloads the extractor or the handler must refuse.
//
// The three extractor rejections are asserted exactly because axum fixes them:
// unparseable JSON is a 400, well-formed JSON of the wrong shape is a 422, and
// a body with no `content-type` is a 415. Everything below them is the
// handler's own validation, whose message is the marker.
const MALFORMED: [Case; 17] = [
    json(
        "post",
        "/api/public/admin/users",
        "{ this is not json",
        Expect::Status(BAD_REQUEST),
        None,
    ),
    json(
        "post",
        "/api/public/admin/users",
        r#"{"name": 5}"#,
        Expect::Status(UNPROCESSABLE),
        None,
    ),
    json(
        "put",
        "/api/public/admin/users/no-such-user",
        "]]]",
        Expect::Status(BAD_REQUEST),
        None,
    ),
    json(
        "post",
        "/api/public/admin/management/departments",
        "{",
        Expect::Status(BAD_REQUEST),
        None,
    ),
    json(
        "post",
        "/api/public/admin/management/departments",
        r#"{"description": "no name"}"#,
        Expect::Status(UNPROCESSABLE),
        None,
    ),
    // Present but blank is the handler's business, not the extractor's.
    json(
        "post",
        "/api/public/admin/management/departments",
        r#"{"name": "   "}"#,
        Expect::Status(BAD_REQUEST),
        Some("name must not be empty"),
    ),
    json(
        "put",
        "/api/public/admin/management/departments/no-such-department",
        r#"{"name": "Renamed"}"#,
        Expect::Status(NOT_FOUND),
        Some("Department not found"),
    ),
    json(
        "post",
        "/api/public/admin/gateway/routes",
        "{{{",
        Expect::Status(BAD_REQUEST),
        None,
    ),
    // `entity_type` is a closed vocabulary mirroring the table's CHECK
    // constraint; an unknown one is rejected rather than stored.
    json(
        "post",
        "/api/public/admin/access-control/entity/not-an-entity-kind/x/rules",
        r#"{"rule_type": "user", "rule_value": "someone", "access": "allow"}"#,
        Expect::Status(BAD_REQUEST),
        Some("invalid entity_type"),
    ),
    json(
        "put",
        "/api/public/admin/access-control/entity/not-an-entity-kind/x",
        "{}",
        Expect::ClientError,
        None,
    ),
    json(
        "patch",
        "/api/public/admin/access-control/entity/not-an-entity-kind/x/default",
        r#"{"default_included": true}"#,
        Expect::Status(BAD_REQUEST),
        Some("invalid entity_type"),
    ),
    json(
        "post",
        "/api/public/admin/access-control/entity/skill/some-skill/rules",
        r#"{"rule_type": "department", "rule_value": "eng", "access": "allow"}"#,
        Expect::Status(BAD_REQUEST),
        Some("invalid rule_type"),
    ),
    json(
        "post",
        "/api/public/admin/access-control/entity/skill/some-skill/rules",
        r#"{"rule_type": "user", "rule_value": "6f1b0f7a-3c2e-4a51-9b8d-2f0c5a7e14d3", "access": "maybe"}"#,
        Expect::Status(BAD_REQUEST),
        Some("invalid access"),
    ),
    json(
        "post",
        "/api/public/admin/access-control/entity/skill/some-skill/rules",
        r#"{"rule_type": "user", "rule_value": "", "access": "allow"}"#,
        Expect::Status(BAD_REQUEST),
        Some("rule_value required"),
    ),
    json(
        "post",
        "/api/public/admin/access-control/bulk-template",
        r#"{"entity_type": "not-an-entity-kind", "subject_type": "user",
            "subject_value": "someone", "action": "allow"}"#,
        Expect::Status(BAD_REQUEST),
        Some("invalid entity_type"),
    ),
    json(
        "post",
        "/api/public/admin/access-control/bulk-template",
        r#"{"entity_type": "skill", "subject_type": "department",
            "subject_value": "eng", "action": "allow"}"#,
        Expect::Status(BAD_REQUEST),
        Some("invalid subject_type"),
    ),
    // An admin who is not the named user still cannot mint a share token for
    // an account that does not exist.
    json(
        "post",
        "/api/public/admin/users/no-such-user/share-token",
        "{}",
        Expect::Status(NOT_FOUND),
        Some("User not found"),
    ),
];

// The hook plane, which is mounted at the root rather than under either admin
// prefix and answers on its own terms: a governance hook returns 200 with a
// decision, because an error status reads to the client as "hook unavailable"
// and lets the call through.
const HOOKS: [Case; 6] = [
    json(
        "post",
        "/hooks/track",
        "{ not json",
        Expect::Status(BAD_REQUEST),
        None,
    ),
    // A session token is not a hook token: `/hooks/track` wants `aud=hook`.
    json(
        "post",
        "/hooks/track",
        r#"{"hook_event_name": "Stop"}"#,
        Expect::Status(UNAUTHORIZED),
        None,
    ),
    json(
        "post",
        "/govern/authz",
        "{ not json",
        Expect::Status(BAD_REQUEST),
        None,
    ),
    json(
        "post",
        "/govern/authz",
        "{}",
        Expect::Status(UNPROCESSABLE),
        None,
    ),
    json(
        "post",
        "/hooks/govern",
        "{ not json",
        Expect::Status(BAD_REQUEST),
        None,
    ),
    // An envelope with nothing to govern is still a decision, not a failure.
    json("post", "/hooks/govern", "{}", Expect::Status(OK), None),
];

// Writes attempted without the standing to make them.
//
// The two planes refuse differently on purpose — the API answers 401/403 to a
// caller that expects JSON, the SSR pages redirect a browser to sign in — so
// the case demands refusal rather than a particular code.
const UNAUTHENTICATED: [Case; 12] = [
    anon("post", "/api/public/admin/users", "{}"),
    anon("put", "/api/public/admin/users/someone", "{}"),
    anon("delete", "/api/public/admin/users/someone", "{}"),
    anon("patch", "/api/public/admin/gateway", "{}"),
    anon("post", "/api/public/admin/gateway/routes", "{}"),
    anon("put", "/api/public/admin/access-control/bulk", "{}"),
    anon("post", "/api/public/admin/management/departments", "{}"),
    anon("post", "/api/public/admin/users/someone/pats", "{}"),
    anon("post", "/admin/tokens/pats", "{}"),
    non_admin("post", "/api/public/admin/users", "{}"),
    non_admin("post", "/api/public/admin/management/departments", "{}"),
    non_admin("post", "/admin/tokens/pats", "{}"),
];

// A body with no `content-type` at all. `Json` refuses it rather than
// guessing, which is a 415 — a distinct outcome from "the JSON was bad".
const NO_CONTENT_TYPE: [Case; 3] = [
    Case {
        method: "post",
        path: "/api/public/admin/users",
        principal: Principal::Admin,
        content_type: None,
        body: Some("{}"),
        expect: Expect::Status(UNSUPPORTED_MEDIA),
        marker: None,
    },
    Case {
        method: "post",
        path: "/api/public/admin/management/departments",
        principal: Principal::Admin,
        content_type: None,
        body: Some(r#"{"name": "Ops"}"#),
        expect: Expect::Status(UNSUPPORTED_MEDIA),
        marker: None,
    },
    Case {
        method: "post",
        path: "/hooks/govern",
        principal: Principal::Admin,
        content_type: None,
        body: Some("{}"),
        expect: Expect::Status(UNSUPPORTED_MEDIA),
        marker: None,
    },
];

// Long and unparseable query strings on the list pages.
//
// A page whose `?page=` is a word, or whose `?from=` is not a date, must
// answer a client error or render its default — never fault. The overlong
// search terms are here because a search box is the easiest place to reach a
// query builder with something it did not expect.
const BAD_QUERY: [Case; 12] = [
    get(
        "/admin/entities/requests?page=not-a-number",
        Expect::Status(BAD_REQUEST),
        None,
    ),
    get(
        "/admin/entities/requests?page=99999999999999999999",
        Expect::Status(BAD_REQUEST),
        None,
    ),
    get(
        "/admin/entities/traces?page=not-a-number",
        Expect::Status(BAD_REQUEST),
        None,
    ),
    get(
        "/admin/entities/contexts?limit=not-a-number",
        Expect::Status(BAD_REQUEST),
        None,
    ),
    get(
        "/api/public/admin/events?limit=not-a-number",
        Expect::Status(BAD_REQUEST),
        None,
    ),
    get(
        "/api/public/admin/users/search?limit=not-a-number",
        Expect::Status(BAD_REQUEST),
        None,
    ),
    // `limit` and `offset` are plain `i64` query parameters bound straight
    // into `LIMIT $3 OFFSET $4`, and Postgres rejects a negative LIMIT — so
    // `list_events` clamps to `1..=500` and `offset` to non-negative before
    // binding. The echoed values are the marker: asserting only "not a 5xx"
    // would pass just as well against an endpoint that had quietly stopped
    // clamping and started returning every row instead.
    get(
        "/api/public/admin/events?limit=-1",
        Expect::Status(OK),
        Some(r#""limit":1,"offset":0"#),
    ),
    get(
        "/api/public/admin/events?limit=-1&offset=-1",
        Expect::Status(OK),
        Some(r#""limit":1,"offset":0"#),
    ),
    get(
        "/api/public/admin/events?limit=999999999",
        Expect::Status(OK),
        Some(r#""limit":500,"offset":0"#),
    ),
    get(
        "/admin/entities/requests?from=not-a-date&to=also-not-a-date",
        Expect::NotServerError,
        None,
    ),
    get(LONG_SEARCH_REQUESTS, Expect::NotServerError, None),
    get(LONG_SEARCH_CONTEXTS, Expect::NotServerError, None),
];

// A search term far longer than any box would submit, spelled out so the
// cases above stay readable.
const LONG_SEARCH_REQUESTS: &str = concat!(
    "/admin/entities/requests?tab=log&q=",
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
);

const LONG_SEARCH_CONTEXTS: &str = concat!(
    "/admin/entities/contexts?q=",
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
);

#[tokio::test(flavor = "multi_thread")]
async fn admin_routes_refuse_malformed_requests_without_faulting() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping admin error-path suite");
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    // Every API case is written against the prefix the router mounts, so a
    // change to it fails loudly here rather than turning every case into a
    // vacuous 404.
    assert_eq!(
        ADMIN_API_PREFIX, "/api/public/admin",
        "the API cases below spell the mount prefix out; update them together"
    );

    let mut failures = Vec::new();
    for case in UNKNOWN_ID
        .iter()
        .chain(MALFORMED.iter())
        .chain(HOOKS.iter())
        .chain(UNAUTHENTICATED.iter())
        .chain(NO_CONTENT_TYPE.iter())
        .chain(BAD_QUERY.iter())
    {
        let (status, body) = app
            .call(Call {
                method: case.method,
                path: case.path,
                principal: case.principal,
                content_type: case.content_type,
                body: case.body,
            })
            .await;

        if !case.expect.accepts(status) {
            failures.push(report(case, status, &body, &case.expect.describe()));
            continue;
        }
        if let Some(marker) = case.marker
            && !body.contains(marker)
        {
            failures.push(report(
                case,
                status,
                &body,
                "a body naming the reason it was refused",
            ));
        }
    }

    db.cleanup().await;

    assert!(
        failures.is_empty(),
        "{} error-path case(s) answered wrongly:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

fn report(case: &Case, status: StatusCode, body: &str, wanted: &str) -> String {
    let head: String = body.chars().take(300).collect();
    format!(
        "  {} {} [{}] -> {} : expected {wanted}\n      body: {head}",
        case.method.to_uppercase(),
        case.path,
        case.principal.label(),
        status.as_u16()
    )
}
