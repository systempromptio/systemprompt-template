//! HTTP coverage of the contexts time window and the explicit All time escape.
use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};
use axum::http::StatusCode;

#[tokio::test]
async fn contexts_defaults_to_thirty_days_and_all_time_preserves_old_conversations() {
    assert!(globals::init());
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let user = seed::insert_user(&db.pool, "window-user", "window-user@contract.test").await;
    sqlx::query("INSERT INTO ai_requests(id,request_id,user_id,context_id,provider,model,actor_kind,actor_id,status,created_at)
        SELECT 'window-request-'||n,'window-request-'||n,$1,'window-context','anthropic','test-model','user',$1,'completed',NOW()-INTERVAL '60 days'
        FROM generate_series(1,2) n").bind(user.as_str()).execute(&*db.pool).await.unwrap();
    let app = App::new(&db.pool, credentials);
    for path in [
        "/admin/contexts",
        "/admin/contexts?since=invalid",
        "/admin/contexts?since=30d&preset=all",
    ] {
        let (status, body) = app.call(Call::get(path, Principal::Admin)).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        assert!(body.contains("value=\"30d\" selected"), "{path}");
        assert!(!body.contains("data-user-id=\"window-user\""));
    }
    for path in [
        "/admin/contexts?since=all",
        "/admin/contexts?preset=all",
        "/admin/contexts?since=all&view=all",
    ] {
        let (status, body) = app.call(Call::get(path, Principal::Admin)).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        assert!(body.contains("data-user-id=\"window-user\""), "{path}");
        assert!(body.contains("value=\"all\" selected"));
    }
    db.cleanup().await;
}

// Opt-in end-to-end HTTP handler timings against the benchmark database.
// This includes authentication, SQL, template rendering and body collection;
// it excludes a TCP socket and the standalone binary's outer middleware.
#[tokio::test(flavor = "multi_thread", worker_threads = 10)]
#[ignore = "requires a seeded disposable DASHBOARD_BENCH_DATABASE_URL; run in release mode"]
async fn benchmark_dashboard_http() {
    use std::sync::Arc;
    use std::time::Instant;
    assert!(globals::init());
    let url = std::env::var("DASHBOARD_BENCH_DATABASE_URL").expect("benchmark database URL");
    let parsed = url::Url::parse(&url).unwrap();
    assert!(parsed.path().ends_with("_test"), "refuse a live database");
    let pool = Arc::new(
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(30)
            .connect(&url)
            .await
            .unwrap(),
    );
    let credentials = principal::provision(&pool).await;
    let app = Arc::new(App::new(&pool, credentials));
    for path in [
        "/admin/contexts?view=all",
        "/admin/contexts",
        "/admin/users",
        "/admin/sessions",
        "/admin/history",
        "/admin/analytics",
        "/admin/groups",
        "/admin/projects",
    ] {
        let (status, _) = app.call(Call::get(path, Principal::Admin)).await;
        assert_eq!(status, StatusCode::OK, "{path}");
        let mut tasks = tokio::task::JoinSet::new();
        for _ in 0..10 {
            let app = Arc::clone(&app);
            tasks.spawn(async move {
                let mut samples = Vec::new();
                for _ in 0..3 {
                    let start = Instant::now();
                    let (status, body) = app.call(Call::get(path, Principal::Admin)).await;
                    assert_eq!(status, StatusCode::OK, "{path}");
                    assert!(body.contains("</html>"));
                    samples.push(start.elapsed().as_secs_f64() * 1000.0);
                }
                samples
            });
        }
        let mut samples = Vec::new();
        while let Some(result) = tasks.join_next().await {
            samples.extend(result.unwrap());
        }
        samples.sort_by(f64::total_cmp);
        println!(
            "DASHBOARD_HTTP {path} p50_ms={:.2} p95_ms={:.2} readers=10 samples=30",
            samples[15], samples[28]
        );
    }
    pool.close().await;
}
