//! The public content API driven as HTTP.
//!
//! `api::router` and `api::redirect_router` are what the extension mounts, so
//! the routes are exercised through them rather than by calling the handler
//! functions: the path shape, the query and body extractors, and the status
//! code each error maps to are all part of what a client sees, and none of
//! them are visible from a direct call.
//!
//! The three content handlers and the two session-cookie handlers are the
//! exception. They are not mounted on either router — the extension serves
//! content pages through SSR — so they are called directly, which is the only
//! entry point they have.

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::extract::{Json as JsonExtract, Path as PathExtract, State};
use axum::http::{HeaderMap, Request, StatusCode, header};
use http_body_util::BodyExt as _;
use serde_json::{Value, json};
use systemprompt::identifiers::{CampaignId, LinkClickId, LinkId, SessionId, SourceId};
use systemprompt_web_content::api::{self, BlogState};
use systemprompt_web_content::repository::{
    ContentRepository, LinkAnalyticsRepository, LinkRepository,
};
use systemprompt_web_shared::config::{BlogConfigRaw, BlogConfigValidated};
use systemprompt_web_shared::models::{CampaignLink, RecordClickParams};
use tower::ServiceExt as _;

use crate::fixtures::{content_params, link_params};
use crate::tempdb::TempDb;

const BASE_URL: &str = "https://astound.test";

fn blog_config() -> Arc<BlogConfigValidated> {
    Arc::new(
        BlogConfigValidated::validate(
            BlogConfigRaw {
                base_url: BASE_URL.to_owned(),
                ..BlogConfigRaw::default()
            },
            std::path::Path::new("."),
        )
        .expect("a config with no content sources validates"),
    )
}

fn app(db: &TempDb) -> Router {
    api::router(Arc::clone(&db.pool), Some(blog_config()))
}

// The same router with no blog config: `generate_link_handler` then has no
// base URL to build a short URL against and falls back to a literal.
fn app_without_config(db: &TempDb) -> Router {
    api::router(Arc::clone(&db.pool), None)
}

fn state(db: &TempDb) -> BlogState {
    BlogState {
        pool: Arc::clone(&db.pool),
        config: Some(blog_config()),
    }
}

async fn get(router: Router, uri: &str) -> (StatusCode, Value) {
    let response = router
        .oneshot(
            Request::builder()
                .uri(uri)
                .body(Body::empty())
                .expect("build the request"),
        )
        .await
        .expect("the router is infallible");
    let status = response.status();
    (status, body_json(response.into_body()).await)
}

async fn post(router: Router, uri: &str, body: &Value) -> (StatusCode, Value) {
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .expect("build the request"),
        )
        .await
        .expect("the router is infallible");
    let status = response.status();
    (status, body_json(response.into_body()).await)
}

async fn body_json(body: Body) -> Value {
    let bytes = body
        .collect()
        .await
        .expect("read the response body")
        .to_bytes();
    serde_json::from_slice(&bytes).unwrap_or(Value::Null)
}

async fn seed_link(db: &TempDb, short_code: &str) -> CampaignLink {
    LinkRepository::new(Arc::clone(&db.pool))
        .create_link(&link_params(short_code))
        .await
        .expect("seed a campaign link")
}

async fn record_click(db: &TempDb, link_id: &LinkId) {
    let params = RecordClickParams::new(
        LinkClickId::generate(),
        link_id.clone(),
        SessionId::new(format!("session-{}", uuid::Uuid::new_v4())),
        chrono::Utc::now(),
    );
    LinkAnalyticsRepository::new(Arc::clone(&db.pool))
        .record_click(&params)
        .await
        .expect("record a click");
}

async fn redirect(db: &TempDb, short_code: &str) -> axum::response::Response {
    api::redirect_router(Arc::clone(&db.pool))
        .oneshot(
            Request::builder()
                .uri(format!("/{short_code}"))
                .body(Body::empty())
                .expect("build the request"),
        )
        .await
        .expect("the router is infallible")
}

#[tokio::test]
async fn generate_link_returns_a_short_url_built_from_the_configured_base_url() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let (status, body) = post(
        app(&db),
        "/links/generate",
        &json!({ "target_url": "https://astound.test/guides/one", "campaign_name": "launch" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    let short_code = body["short_code"].as_str().expect("a short code is issued");
    assert_eq!(
        body["short_url"],
        json!(format!("{BASE_URL}/r/{short_code}"))
    );
    assert_eq!(body["target_url"], json!("https://astound.test/guides/one"));

    db.cleanup().await;
}

#[tokio::test]
async fn generate_link_falls_back_to_a_literal_base_url_when_no_config_is_mounted() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let (status, body) = post(
        app_without_config(&db),
        "/links/generate",
        &json!({ "target_url": "https://astound.test/guides/two" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        body["short_url"]
            .as_str()
            .expect("a short url")
            .starts_with("https://example.com/r/")
    );

    db.cleanup().await;
}

#[tokio::test]
async fn generate_link_rejects_a_target_that_is_not_a_url() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let (status, body) = post(
        app(&db),
        "/links/generate",
        &json!({ "target_url": "not a url at all" }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("Invalid URL"));

    db.cleanup().await;
}

#[tokio::test]
async fn generate_link_rejects_a_scheme_other_than_http_or_https() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let (status, body) = post(
        app(&db),
        "/links/generate",
        &json!({ "target_url": "javascript:alert(1)" }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], json!("Only http and https URLs are allowed"));

    db.cleanup().await;
}

#[tokio::test]
async fn list_links_filters_by_campaign() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let campaign = CampaignId::new("spring".to_owned());
    LinkRepository::new(Arc::clone(&db.pool))
        .create_link(&link_params("camp01").with_campaign_id(Some(campaign.clone())))
        .await
        .expect("seed a campaign link");
    seed_link(&db, "other1").await;

    let (status, body) = get(app(&db), "/links?campaign_id=spring").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], json!(1));
    assert_eq!(body["links"][0]["short_code"], json!("camp01"));

    db.cleanup().await;
}

#[tokio::test]
async fn list_links_filters_by_source_content() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let content = ContentRepository::new(Arc::clone(&db.pool))
        .create(&content_params("linked", &SourceId::new("blog".to_owned())))
        .await
        .expect("seed the article the link hangs off");
    LinkRepository::new(Arc::clone(&db.pool))
        .create_link(&link_params("cont01").with_source_content_id(Some(content.id.clone())))
        .await
        .expect("seed a content link");

    let (status, body) = get(
        app(&db),
        &format!("/links?content_id={}", content.id.as_str()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], json!(1));
    assert_eq!(body["links"][0]["short_code"], json!("cont01"));

    db.cleanup().await;
}

#[tokio::test]
async fn list_links_with_no_filter_returns_an_empty_page_rather_than_every_link() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed_link(&db, "unlist").await;

    let (status, body) = get(app(&db), "/links").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], json!(0));
    assert_eq!(body["links"], json!([]));

    db.cleanup().await;
}

#[tokio::test]
async fn recording_a_click_persists_the_attributes_the_caller_supplied() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let link = seed_link(&db, "click1").await;

    let response = app(&db)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/links/click")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(
                    json!({
                        "link_id": link.id.as_str(),
                        "session_id": "session-explicit",
                        "referrer_page": "/guides/one",
                        "referrer_url": "https://astound.test/guides/one",
                        "user_agent": "integration-test",
                        "device_type": "desktop",
                    })
                    .to_string(),
                ))
                .expect("build the request"),
        )
        .await
        .expect("the router is infallible");

    assert_eq!(response.status(), StatusCode::CREATED);

    let (session, device): (String, Option<String>) =
        sqlx::query_as("SELECT session_id, device_type FROM link_clicks WHERE link_id = $1")
            .bind(link.id.as_str())
            .fetch_one(&*db.pool)
            .await
            .expect("read the recorded click back");
    assert_eq!(session, "session-explicit");
    assert_eq!(device.as_deref(), Some("desktop"));

    db.cleanup().await;
}

#[tokio::test]
async fn recording_a_click_generates_a_session_id_when_the_caller_omits_one() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let link = seed_link(&db, "click2").await;

    let (status, _) = post(
        app(&db),
        "/links/click",
        &json!({ "link_id": link.id.as_str() }),
    )
    .await;

    assert_eq!(status, StatusCode::CREATED);
    let session: String =
        sqlx::query_scalar("SELECT session_id FROM link_clicks WHERE link_id = $1")
            .bind(link.id.as_str())
            .fetch_one(&*db.pool)
            .await
            .expect("read the recorded click back");
    assert!(!session.is_empty(), "a session id is minted for the click");

    db.cleanup().await;
}

#[tokio::test]
async fn recording_a_click_against_an_unknown_link_is_a_server_error() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let (status, body) = post(
        app(&db),
        "/links/click",
        &json!({ "link_id": "link-that-does-not-exist" }),
    )
    .await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error"], json!("Internal server error"));

    db.cleanup().await;
}

#[tokio::test]
async fn link_performance_reports_the_clicks_the_redirect_counted() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let link = seed_link(&db, "perf01").await;
    redirect(&db, "perf01").await;
    redirect(&db, "perf01").await;

    let (status, body) = get(
        app(&db),
        &format!("/links/{}/performance", link.id.as_str()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["click_count"], json!(2));
    assert_eq!(body["conversion_rate"], json!(0.0));

    db.cleanup().await;
}

// The two ways a click is recorded do different things to the denormalised
// counter: the redirect increments `campaign_links.click_count` in the same
// transaction, the API endpoint only appends to `link_clicks`. The performance
// endpoint reads the counter, so a click posted through the API is listed but
// not counted.
#[tokio::test]
async fn a_click_posted_through_the_api_is_listed_but_not_counted_by_the_performance_view() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let link = seed_link(&db, "asym01").await;
    record_click(&db, &link.id).await;

    let (_, performance) = get(
        app(&db),
        &format!("/links/{}/performance", link.id.as_str()),
    )
    .await;
    let (_, clicks) = get(app(&db), &format!("/links/{}/clicks", link.id.as_str())).await;

    assert_eq!(performance["click_count"], json!(0));
    assert_eq!(clicks.as_array().expect("an array of clicks").len(), 1);

    db.cleanup().await;
}

#[tokio::test]
async fn link_performance_is_a_404_for_an_unknown_link() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let (status, body) = get(app(&db), "/links/link-absent/performance").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], json!("Link not found"));

    db.cleanup().await;
}

#[tokio::test]
async fn link_clicks_lists_the_recorded_clicks() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let link = seed_link(&db, "clicks1").await;
    record_click(&db, &link.id).await;

    let (status, body) = get(app(&db), &format!("/links/{}/clicks", link.id.as_str())).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body.as_array().expect("an array of clicks").len(),
        1,
        "the one recorded click is listed"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn link_clicks_is_an_empty_list_for_an_unknown_link() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let (status, body) = get(app(&db), "/links/link-absent/clicks").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, json!([]));

    db.cleanup().await;
}

#[tokio::test]
async fn campaign_performance_aggregates_the_links_in_the_campaign() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let campaign = CampaignId::new("summer".to_owned());
    LinkRepository::new(Arc::clone(&db.pool))
        .create_link(&link_params("camp02").with_campaign_id(Some(campaign)))
        .await
        .expect("seed a campaign link");
    redirect(&db, "camp02").await;

    let (status, body) = get(app(&db), "/links/campaigns/summer/performance").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total_clicks"], json!(1));
    assert_eq!(body["link_count"], json!(1));

    db.cleanup().await;
}

#[tokio::test]
async fn campaign_performance_is_a_404_for_a_campaign_with_no_links() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let (status, body) = get(app(&db), "/links/campaigns/absent/performance").await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"], json!("Campaign not found"));

    db.cleanup().await;
}

#[tokio::test]
async fn the_content_journey_lists_the_links_a_page_sends_traffic_through() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let content = ContentRepository::new(Arc::clone(&db.pool))
        .create(&content_params(
            "journey",
            &SourceId::new("blog".to_owned()),
        ))
        .await
        .expect("seed the source article");
    let link = LinkRepository::new(Arc::clone(&db.pool))
        .create_link(&link_params("jrny01").with_source_content_id(Some(content.id.clone())))
        .await
        .expect("seed a link out of that article");
    record_click(&db, &link.id).await;

    let (status, body) = get(
        app(&db),
        &format!("/links/journey?content_id={}", content.id.as_str()),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().expect("an array of nodes").len(), 1);

    db.cleanup().await;
}

#[tokio::test]
async fn the_content_journey_requires_a_content_id() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let (status, _) = get(app(&db), "/links/journey").await;

    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "a journey with no subject is a client error, not an empty answer"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn search_returns_the_matching_articles() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    ContentRepository::new(Arc::clone(&db.pool))
        .create(&content_params(
            "governance-guide",
            &SourceId::new("blog".to_owned()),
        ))
        .await
        .expect("seed an article to find");

    let (status, body) = get(app(&db), "/search?q=governance-guide&limit=5").await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total"], json!(1));

    db.cleanup().await;
}

#[tokio::test]
async fn search_requires_a_query_string() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let (status, _) = get(app(&db), "/search").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);

    db.cleanup().await;
}

#[tokio::test]
async fn a_short_code_redirects_to_its_target_and_counts_the_click() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    seed_link(&db, "redir1").await;

    let response = api::redirect_router(Arc::clone(&db.pool))
        .oneshot(
            Request::builder()
                .uri("/redir1")
                .body(Body::empty())
                .expect("build the request"),
        )
        .await
        .expect("the router is infallible");

    assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(
        response
            .headers()
            .get(header::LOCATION)
            .and_then(|v| v.to_str().ok()),
        Some("https://example.com/redir1")
    );

    let clicks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM link_clicks")
        .fetch_one(&*db.pool)
        .await
        .expect("count the recorded clicks");
    assert_eq!(clicks, 1, "the redirect records the click it served");

    db.cleanup().await;
}

#[tokio::test]
async fn an_unknown_short_code_is_a_404() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let response = api::redirect_router(Arc::clone(&db.pool))
        .oneshot(
            Request::builder()
                .uri("/absent")
                .body(Body::empty())
                .expect("build the request"),
        )
        .await
        .expect("the router is infallible");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        body_json(response.into_body()).await["error"],
        json!("Link not found")
    );

    db.cleanup().await;
}

// A stored target with a non-web scheme is refused at redirect time, not at
// creation time: rows predate the check on the generate route, and a
// `javascript:` Location is what it exists to stop.
#[tokio::test]
async fn a_stored_target_with_a_dangerous_scheme_is_refused_at_redirect_time() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    LinkRepository::new(Arc::clone(&db.pool))
        .create_link(&systemprompt_web_shared::models::CreateLinkParams::new(
            "danger".to_owned(),
            "javascript:alert(1)".to_owned(),
            "cta",
        ))
        .await
        .expect("seed a link whose target predates the scheme check");

    let response = api::redirect_router(Arc::clone(&db.pool))
        .oneshot(
            Request::builder()
                .uri("/danger")
                .body(Body::empty())
                .expect("build the request"),
        )
        .await
        .expect("the router is infallible");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        body_json(response.into_body()).await["error"],
        json!("Invalid redirect target")
    );

    db.cleanup().await;
}

#[tokio::test]
async fn the_query_handler_searches_the_body_it_is_given() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    ContentRepository::new(Arc::clone(&db.pool))
        .create(&content_params(
            "query-target",
            &SourceId::new("blog".to_owned()),
        ))
        .await
        .expect("seed an article to find");

    let response = systemprompt_web_content::api::handlers::query_handler(
        State(state(&db)),
        JsonExtract(systemprompt_web_shared::models::SearchRequest {
            query: "query-target".to_owned(),
            filters: None,
            limit: Some(5),
        }),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(body_json(response.into_body()).await["total"], json!(1));

    db.cleanup().await;
}

#[tokio::test]
async fn listing_content_by_source_returns_only_that_source() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let repo = ContentRepository::new(Arc::clone(&db.pool));
    repo.create(&content_params(
        "in-blog",
        &SourceId::new("blog".to_owned()),
    ))
    .await
    .expect("seed a blog article");
    repo.create(&content_params(
        "in-docs",
        &SourceId::new("documentation".to_owned()),
    ))
    .await
    .expect("seed a documentation page");

    let response = systemprompt_web_content::api::handlers::list_content_handler(
        State(state(&db)),
        PathExtract(SourceId::new("blog".to_owned())),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body = body_json(response.into_body()).await;
    let rows = body.as_array().expect("an array of content");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["slug"], json!("in-blog"));

    db.cleanup().await;
}

#[tokio::test]
async fn getting_one_article_by_source_and_slug_returns_it() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    ContentRepository::new(Arc::clone(&db.pool))
        .create(&content_params("wanted", &SourceId::new("blog".to_owned())))
        .await
        .expect("seed the article");

    let response = systemprompt_web_content::api::handlers::get_content_handler(
        State(state(&db)),
        PathExtract((SourceId::new("blog".to_owned()), "wanted".to_owned())),
    )
    .await;

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        body_json(response.into_body()).await["slug"],
        json!("wanted")
    );

    db.cleanup().await;
}

#[tokio::test]
async fn getting_an_article_that_does_not_exist_is_a_404() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let response = systemprompt_web_content::api::handlers::get_content_handler(
        State(state(&db)),
        PathExtract((SourceId::new("blog".to_owned()), "absent".to_owned())),
    )
    .await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        body_json(response.into_body()).await["error"],
        json!("Content not found")
    );

    db.cleanup().await;
}

#[tokio::test]
async fn setting_a_session_issues_an_access_cookie_and_a_refresh_cookie() {
    crate::jobs_context::install_config();

    let (headers, _) = systemprompt_web_content::api::auth::set_session(
        HeaderMap::new(),
        axum::Json(systemprompt_web_content::api::auth::SetSessionRequest {
            access_token: "access-abc".to_owned(),
            expires_in: Some(120),
            refresh_token: Some("refresh-xyz".to_owned()),
        }),
    )
    .await;

    let cookies = set_cookies(&headers);
    assert_eq!(cookies.len(), 2, "both cookies are issued");
    assert!(
        cookies[0]
            .starts_with("access_token=access-abc; Path=/; HttpOnly; SameSite=Lax; Max-Age=120")
    );
    assert!(cookies[0].contains("; Secure"), "the test config is https");
    assert!(cookies[1].contains("refresh_token=refresh-xyz"));
    assert!(
        cookies[1].contains("Path=/api/public/auth"),
        "the refresh cookie is scoped to the refresh endpoint"
    );
}

#[tokio::test]
async fn a_session_with_no_refresh_token_issues_only_the_access_cookie() {
    crate::jobs_context::install_config();

    let (headers, body) = systemprompt_web_content::api::auth::set_session(
        HeaderMap::new(),
        axum::Json(systemprompt_web_content::api::auth::SetSessionRequest {
            access_token: "access-only".to_owned(),
            expires_in: None,
            refresh_token: None,
        }),
    )
    .await;

    assert!(body.ok);
    let cookies = set_cookies(&headers);
    assert_eq!(cookies.len(), 1);
    assert!(
        cookies[0].contains("Max-Age=3600"),
        "an unspecified lifetime defaults to an hour: {}",
        cookies[0]
    );
}

#[tokio::test]
async fn clearing_a_session_expires_both_cookies() {
    crate::jobs_context::install_config();

    let (headers, body) = systemprompt_web_content::api::auth::clear_session().await;

    assert!(body.ok);
    let cookies = set_cookies(&headers);
    assert_eq!(cookies.len(), 2);
    for cookie in &cookies {
        assert!(
            cookie.contains("Max-Age=0"),
            "every cleared cookie expires immediately: {cookie}"
        );
    }
}

fn set_cookies(headers: &HeaderMap) -> Vec<String> {
    headers
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|v| v.to_str().expect("cookies are ASCII").to_owned())
        .collect()
}
