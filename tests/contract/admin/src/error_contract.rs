//! [`AdminError`]'s classification, and the two faces it renders through.
//!
//! Every handler in the plane propagates with a bare `?`, so the variant alone
//! decides both the status code and what the caller is told. That makes this
//! type the single place where an internal cause could leak into a response
//! body, and the single place where a mis-sorted variant turns a client's
//! mistake into a reported server fault.
//!
//! The cases below are exhaustive over the variants deliberately: the JSON and
//! HTML faces are separate `IntoResponse` implementations, and a variant added
//! to only one of them is exactly the drift this asserts against.

use axum::http::StatusCode;
use axum::response::IntoResponse;
use http_body_util::BodyExt as _;
use serde_json::Value;
use systemprompt_web_admin::error::{AdminError, AdminHtmlError};

// The status and rendered body of the JSON face.
async fn json_face(error: AdminError) -> (StatusCode, Value) {
    let response = error.into_response();
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("read the error body")
        .to_bytes();
    let value = serde_json::from_slice(&bytes).expect("the JSON face renders JSON");
    (status, value)
}

// The status and rendered body of the HTML face.
async fn html_face(error: AdminError) -> (StatusCode, String) {
    let response = AdminHtmlError(error).into_response();
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("read the error body")
        .to_bytes();
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

// One variant per class, spelled with a message distinctive enough that a
// leak into the response body is unmistakable.
fn client_errors() -> Vec<(AdminError, StatusCode, &'static str)> {
    vec![
        (
            AdminError::NotFound("no such widget".to_owned()),
            StatusCode::NOT_FOUND,
            "no such widget",
        ),
        (
            AdminError::BadRequest("id is not a uuid".to_owned()),
            StatusCode::BAD_REQUEST,
            "id is not a uuid",
        ),
        (
            AdminError::Unauthorized("token expired".to_owned()),
            StatusCode::UNAUTHORIZED,
            "token expired",
        ),
        (
            AdminError::Forbidden("admin only".to_owned()),
            StatusCode::FORBIDDEN,
            "admin only",
        ),
        (
            AdminError::Conflict("already exists".to_owned()),
            StatusCode::CONFLICT,
            "already exists",
        ),
        (
            AdminError::RateLimited("slow down".to_owned()),
            StatusCode::TOO_MANY_REQUESTS,
            "slow down",
        ),
        (
            AdminError::Unavailable("salesforce is not configured".to_owned()),
            StatusCode::SERVICE_UNAVAILABLE,
            "salesforce is not configured",
        ),
    ]
}

#[tokio::test]
async fn client_error_variants_carry_their_own_message() {
    for (error, expected_status, expected_message) in client_errors() {
        let (status, body) = json_face(error).await;
        assert_eq!(status, expected_status, "status for {expected_message}");
        assert_eq!(body["error"], expected_message);
    }
}

#[tokio::test]
async fn client_error_messages_survive_the_html_face_too() {
    // The two faces classify through the same code path, so a status that
    // disagreed between them would mean a browser and an API client were told
    // different things about the same failure.
    for (error, expected_status, expected_message) in client_errors() {
        let (status, body) = html_face(error).await;
        assert_eq!(status, expected_status, "html status for {expected_message}");
        assert!(
            body.contains(expected_message),
            "the page shows the public message: {body}"
        );
        assert!(
            body.contains(&expected_status.as_u16().to_string()),
            "the page shows the status code"
        );
    }
}

#[tokio::test]
async fn server_side_causes_never_reach_the_caller() {
    // Each of these wraps a cause that names something internal. The response
    // must be the generic text; the detail belongs in the log only.
    let cases: Vec<(AdminError, StatusCode, &str)> = vec![
        (
            AdminError::Database(sqlx::Error::RowNotFound),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error",
        ),
        (
            AdminError::internal("connection string was postgres://user:hunter2@db"),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal server error",
        ),
        (
            AdminError::unauthenticated("signature verification failed for kid abc123"),
            StatusCode::UNAUTHORIZED,
            "Unauthorized",
        ),
        (
            AdminError::Upstream("salesforce returned 503 for /services/oauth2/token".to_owned()),
            StatusCode::BAD_GATEWAY,
            "Upstream service error",
        ),
    ];

    for (error, expected_status, expected_message) in cases {
        let leak = error.to_string();
        let (status, body) = json_face(error).await;
        assert_eq!(status, expected_status, "status for {expected_message}");
        assert_eq!(body["error"], expected_message);
        assert!(
            !leak.is_empty(),
            "the internal rendering is still available for logging"
        );
    }
}

#[tokio::test]
async fn the_html_face_escapes_the_message_it_renders() {
    // The public message is interpolated into a page, and `BadRequest` is the
    // one class whose text can carry caller-supplied content.
    let (status, body) = html_face(AdminError::BadRequest(
        "<script>alert('xss')</script>".to_owned(),
    ))
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        !body.contains("<script>alert"),
        "the message is escaped, not injected: {body}"
    );
    assert!(
        body.contains("&lt;script&gt;"),
        "and it is escaped rather than dropped: {body}"
    );
}

#[tokio::test]
async fn html_internal_errors_render_a_page_rather_than_a_body_of_json() {
    let response = AdminHtmlError::internal("the write pool is exhausted").into_response();
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("read the error body")
        .to_bytes();
    let body = String::from_utf8_lossy(&bytes);

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(body.starts_with("<!DOCTYPE html>"), "it is a page: {body}");
    assert!(
        !body.contains("write pool is exhausted"),
        "the cause stays in the log: {body}"
    );
    assert!(
        body.contains("Internal server error"),
        "the page still says what class of failure this was"
    );
}

#[tokio::test]
async fn conversions_land_on_the_variant_that_matches_the_source() {
    // `?` in a handler relies on these, so a `From` that classified a
    // marketplace `NotFound` as a 500 would turn every missing row into a
    // reported outage.
    use systemprompt_web_shared::error::MarketplaceError;

    let (status, body) = json_face(MarketplaceError::NotFound("plugin".to_owned()).into()).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], "plugin");

    let (status, body) =
        json_face(MarketplaceError::BadRequest("bad filter".to_owned()).into()).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "bad filter");

    // Any other marketplace failure is ours, not the caller's.
    let (status, body) = json_face(MarketplaceError::Internal("boom".to_owned()).into()).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error"], "Internal server error");

    // The SSR alias converts through the same route, so an SSR handler cannot
    // disagree with an API handler about what a given failure means.
    let html: AdminHtmlError = MarketplaceError::NotFound("plugin".to_owned()).into();
    assert_eq!(html.0.status(), StatusCode::NOT_FOUND);
}
