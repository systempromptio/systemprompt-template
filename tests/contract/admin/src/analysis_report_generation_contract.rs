//! On-demand report generation records a terminal outcome when inference is
//! absent.

use axum::http::StatusCode;
use serde_json::Value;
use std::time::{Duration, Instant};

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

fn request_origin() -> String {
    let profile = systemprompt::config::ProfileBootstrap::get().expect("fixture profile");
    url::Url::parse(&profile.server.api_external_url)
        .expect("fixture origin")
        .origin()
        .ascii_serialization()
}

#[tokio::test(flavor = "multi_thread")]
async fn report_request_without_an_ai_provider_records_a_failed_terminal_audit_row() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let form = Call {
        method: "post",
        path: "/admin/analysis/reports",
        principal: Principal::Admin,
        content_type: Some("application/x-www-form-urlencoded"),
        body: Some("scope_kind=global&days=1"),
    };
    let origin = request_origin();
    let (status, headers) = app
        .response_headers_with(form, &[("origin", &origin)])
        .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    let location = headers.location.expect("queued report redirect");
    let report_id = location
        .strip_prefix("/admin/analysis/reports/")
        .expect("redirect points at the report row");

    let deadline = Instant::now() + Duration::from_secs(10);
    let error = loop {
        let status_path = format!("/admin/analysis/reports/{report_id}/status");
        let (status, body) = app.call(Call::get(&status_path, Principal::Admin)).await;
        assert_eq!(status, StatusCode::OK, "report status: {body}");
        let status: Value = serde_json::from_str(&body).expect("status JSON");
        let last_state = status["status"].as_str().unwrap_or("malformed");
        if status["status"] == "failed" {
            break Ok(status["error"]
                .as_str()
                .expect("failed report includes its reason")
                .to_owned());
        }
        if Instant::now() >= deadline {
            break Err(last_state.to_owned());
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    };
    let error = error.unwrap_or_else(|last_state| {
        panic!("report generation did not fail within ten seconds; last state: {last_state}")
    });
    assert!(error.contains("no AI provider is configured"));
    let row: (String, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT status, last_error, lease_token FROM analysis_reports WHERE id = $1",
    )
    .bind(report_id)
    .fetch_one(&*db.pool)
    .await
    .expect("queued report remains as an audit row");
    assert_eq!(row.0, "failed");
    assert_eq!(row.1.as_deref(), Some(error.as_str()));
    assert!(row.2.is_none(), "a terminal report must release its lease");

    db.cleanup().await;
}
