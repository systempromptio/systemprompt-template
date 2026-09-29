//! A report page is the audit record of an AI reading.  These contracts seed
//! completed, pending, and failed records directly so rendering coverage never
//! needs a provider call.

use axum::http::StatusCode;
use serde_json::json;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

fn inputs() -> serde_json::Value {
    json!({
        "digest": {
            "window_start": "2025-01-01T00:00:00Z",
            "window_end": "2025-01-31T00:00:00Z",
            "totals": {
                "conversations": 13, "people": 4, "turns": 38, "tokens": 12000,
                "cost_microdollars": 2500000, "errors": 2, "denied": 1,
                "safety_findings": 1, "artifacts": 3, "tool_calls": 9, "judged": 8,
                "completion_avg": 62.5, "achieved": 5, "partial": 2,
                "abandoned": 1, "unclear": 0
            },
            "skills": [{
                "skill": "contract-plugin:contract-skill", "invocations": 9,
                "conversations": 7, "cost_microdollars": 1500000,
                "completion_avg": 55.0, "errors": 2
            }],
            "models": [{
                "model": "contract-model", "conversations": 13, "requests": 22,
                "cost_microdollars": 2500000, "errors": 2, "completion_avg": 62.5
            }],
            "people": [], "worst": [], "best": [], "outliers": [], "denials": [],
            "intents": [], "clients": []
        },
        "filter_query": "?group=contract-group&outcome=partial",
        "filter_label": "Contract filtered view"
    })
}

async fn insert_report(
    pool: &sqlx::PgPool,
    id: &str,
    status: &str,
    findings: Option<serde_json::Value>,
    error: Option<&str>,
) {
    sqlx::query(
        "INSERT INTO analysis_reports
             (id, scope_kind, scope_label, window_start, window_end, status, requested_by,
              provider, model, input_tokens, output_tokens, cost_microdollars, inputs, findings,
              last_error, generated_at, created_at, updated_at)
         VALUES ($1, 'filter', 'Contract filtered view', '2025-01-01T00:00:00Z',
                 '2025-01-31T00:00:00Z', $2, 'contract-admin', 'contract-provider',
                 'contract-model', 200, 100, 2500000, $3::jsonb, $4::jsonb, $5,
                 CASE WHEN $2 = 'generated' THEN now() ELSE NULL END,
                 now(), now())",
    )
    .bind(id)
    .bind(status)
    .bind(inputs().to_string())
    .bind(findings.map(|value| value.to_string()))
    .bind(error)
    .execute(pool)
    .await
    .expect("seed analysis report");
}

fn request_origin() -> String {
    let profile = systemprompt::config::ProfileBootstrap::get().expect("fixture profile");
    url::Url::parse(&profile.server.api_external_url)
        .expect("fixture origin")
        .origin()
        .ascii_serialization()
}

#[tokio::test(flavor = "multi_thread")]
async fn report_history_and_detail_render_the_saved_audit_record() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let generated = seed::unique("generated-report");
    let pending = seed::unique("pending-report");
    let failed = seed::unique("failed-report");
    insert_report(
        &db.pool,
        &generated,
        "generated",
        Some(json!({
            "headline": "Contract reliability needs attention",
            "assessment": "degraded",
            "themes": [{
                "kind": "reliability", "severity": "err", "title": "Contract failure theme",
                "detail": "Two requests failed after the scoped tool was denied.",
                "evidence": [{
                    "kind": "skill", "label": "contract-plugin:contract-skill",
                    "href": "/admin/analysis/skills?skill=contract-plugin%3Acontract-skill"
                }]
            }],
            "recommendations": [{
                "action": "Repair the contract skill", "rationale": "It owns the failures.",
                "priority": "high"
            }]
        })),
        None,
    )
    .await;
    insert_report(&db.pool, &pending, "pending", None, None).await;
    insert_report(
        &db.pool,
        &failed,
        "failed",
        None,
        Some("contract provider was unavailable"),
    )
    .await;

    let (status, body) = app
        .call(Call::get("/admin/analysis/reports", Principal::Admin))
        .await;
    assert_eq!(status, StatusCode::OK, "report history: {body}");
    for report_id in [&generated, &pending, &failed] {
        assert!(
            body.contains(&format!("/admin/analysis/reports/{report_id}")),
            "history links to report {report_id}: {body}"
        );
    }
    assert!(body.contains("Contract reliability needs attention"));
    assert!(body.contains("writing…"));
    assert!(body.contains("contract provider was unavailable"));

    let path = format!("/admin/analysis/reports/{generated}");
    let (status, body) = app.call(Call::get(&path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK, "generated report: {body}");
    assert!(body.contains("Contract reliability needs attention"));
    assert!(body.contains("Degraded"));
    assert!(body.contains("Contract failure theme"));
    assert!(body.contains("Two requests failed after the scoped tool was denied."));
    assert!(body.contains("Repair the contract skill"));
    assert!(body.contains("contract-plugin:contract-skill"));
    assert!(body.contains("from a filtered view"));
    assert!(
        body.contains("TOTALS conversations&#x3D;13 people&#x3D;4 turns&#x3D;38"),
        "the retained digest, rather than an ambient counter, is rendered: {body}"
    );

    let pending_path = format!("/admin/analysis/reports/{pending}");
    let (status, body) = app.call(Call::get(&pending_path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK, "pending report: {body}");
    assert!(body.contains("Writing this report…"));
    assert!(body.contains(&format!(
        "data-report-poll=\"/admin/analysis/reports/{pending}/status\""
    )));

    let failed_path = format!("/admin/analysis/reports/{failed}");
    let (status, body) = app.call(Call::get(&failed_path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK, "failed report: {body}");
    assert!(body.contains("Could not write this report"));
    assert!(body.contains("contract provider was unavailable"));
    assert!(body.contains("Retry"));
    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn report_form_is_console_only_and_requires_a_scoped_target() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (status, headers) = app
        .response_headers(Call::get("/admin/analysis/reports", Principal::NonAdmin))
        .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert_eq!(
        headers.location.as_deref(),
        Some("/admin/profile"),
        "the SSR gate returns a signed-in non-console account to its profile instead of exposing reports"
    );

    let form = Call {
        method: "post",
        path: "/admin/analysis/reports",
        principal: Principal::Admin,
        content_type: Some("application/x-www-form-urlencoded"),
        body: Some("scope_kind=marketplace&days=30"),
    };
    let origin = request_origin();
    let (status, _) = app
        .response_headers_with(form, &[("origin", &origin)])
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    db.cleanup().await;
}
