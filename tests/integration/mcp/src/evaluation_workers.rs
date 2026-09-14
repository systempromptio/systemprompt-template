use axum::body::Body;
use axum::http::{Request, StatusCode};
use systemprompt::evaluation::repository::experiments::WorkerRepository;
use systemprompt::identifiers::UserId;
use systemprompt_api::routes::evaluation::{EvaluationWorkerState, router};
use tower::ServiceExt;

#[tokio::test]
async fn evaluation_worker_transport_requires_environment_scoped_revocable_credentials() {
    let db = template_test_common::TempDb::create()
        .await
        .expect("PostgreSQL required");
    let owner = UserId::new("worker-enrollment-owner");
    sqlx::query(
        "INSERT INTO users(id,name,email) VALUES($1,'evaluator','worker-test@example.invalid')",
    )
    .bind(owner.as_str())
    .execute(&*db.pool)
    .await
    .unwrap();
    let workers = WorkerRepository::new((*db.pool).clone());
    let credential = workers
        .create(&owner, "test-environment", "linux-evaluator")
        .await
        .unwrap();
    assert!(!format!("{credential:?}").contains(credential.expose_token()));
    assert!(
        workers
            .authenticate(credential.expose_token(), "other-environment")
            .await
            .is_err()
    );
    let state = EvaluationWorkerState::builder((*db.pool).clone())
        .environment("test-environment".into())
        .build()
        .unwrap();
    let app = router(state);
    let response = app
        .clone()
        .oneshot(Request::post("/claim").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let claim = || {
        Request::post("/claim")
            .header(
                "authorization",
                format!("Bearer {}", credential.expose_token()),
            )
            .body(Body::empty())
            .unwrap()
    };
    let response = app.clone().oneshot(claim()).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    workers.revoke(&owner, &credential.id).await.unwrap();
    let response = app.oneshot(claim()).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    db.cleanup().await;
}
