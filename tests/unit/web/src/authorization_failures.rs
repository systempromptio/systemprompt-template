//! Enforcement failures must be decisions, never a transport-level bypass.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::sync::Arc;
use systemprompt::analytics::AnalyticsService;
use systemprompt::analytics::repository::AnalyticsRepositories;
use systemprompt::database::Database;
use systemprompt::oauth::SessionCreationService;
use systemprompt::users::{UserRepository, UserService};
use tower::ServiceExt;

#[tokio::test]
async fn authz_database_failure_returns_http_200_with_an_explicit_deny() {
    let pool = Arc::new(
        sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://test:test@localhost:1/test")
            .unwrap(),
    );
    let database = Arc::new(Database::from_pools(pool.clone(), Some(pool.clone())));
    let users = UserService::new(Arc::new(UserRepository::new(&database).unwrap()));
    let repositories = AnalyticsRepositories::new(&database).unwrap();
    let analytics = AnalyticsService::new(None, None, &repositories);
    let sessions = Arc::new(SessionCreationService::new(
        Arc::new(analytics),
        Arc::new(users),
    ));
    let router = systemprompt_web_admin::hooks_webhook_router(pool.clone(), sessions);
    pool.close().await;
    let payload = serde_json::json!({
        "entity": systemprompt_security::authz::EntityRef::McpServer(systemprompt::identifiers::McpServerId::new("atlassian")),
        "user_id": "00000000-0000-0000-0000-000000000001",
        "trace_id": "00000000-0000-0000-0000-000000000002",
        "roles": ["admin"]
    });
    let response = router
        .oneshot(
            Request::builder()
                .uri("/govern/authz")
                .method("POST")
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(response.into_body(), 16384)
        .await
        .unwrap();
    let decision: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(decision["decision"], "deny");
    assert_eq!(decision["policy"], "authz");
    assert!(decision.to_string().contains("authorization_unavailable"));
}
