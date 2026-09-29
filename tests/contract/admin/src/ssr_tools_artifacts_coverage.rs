//! Populated SSR coverage for the shared Tools and Artifacts activity lens.

use axum::http::StatusCode;
use sqlx::PgPool;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal, seed};

async fn seed_artifact(pool: &PgPool) -> String {
    let user = seed::insert_user(
        pool,
        &seed::unique("artifact-user"),
        "artifact-user@contract.test",
    )
    .await;
    let execution = seed::unique("artifact-execution");
    let artifact = seed::unique("artifact");
    let payload = format!("{:064x}", 7_u8);
    sqlx::query(
        "INSERT INTO mcp_tool_executions
         (mcp_execution_id, tool_name, server_name, started_at, completed_at,
          execution_time_ms, input, status, user_id, source, correlation)
         VALUES ($1, 'report', 'contract-server', NOW(), NOW(), 42,
                 '{\"query\":\"quarterly\"}', 'success', $2, 'in_process', 'exact')",
    )
    .bind(&execution)
    .bind(user.as_str())
    .execute(pool)
    .await
    .expect("insert tool execution");
    sqlx::query(
        "INSERT INTO artifact_payloads (sha256, byte_len, body)
         VALUES ($1, 24, '{\"answer\":\"safe\"}')",
    )
    .bind(&payload)
    .execute(pool)
    .await
    .expect("insert artifact payload");
    sqlx::query(
        "INSERT INTO mcp_artifacts
         (artifact_id, mcp_execution_id, user_id, server_name, tool_name,
          artifact_type, title, source, data, payload_sha256, payload_bytes,
          is_structured, secret_redactions)
         VALUES ($1, $2, $3, 'contract-server', 'report', 'report',
                 'Quarterly report', 'in_process', '{\"answer\":\"safe\"}',
                 $4, 24, true, 1)",
    )
    .bind(&artifact)
    .bind(&execution)
    .bind(user.as_str())
    .bind(&payload)
    .execute(pool)
    .await
    .expect("insert artifact");
    artifact
}

#[tokio::test(flavor = "multi_thread")]
async fn populated_tools_and_artifact_views_render_the_same_call() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let artifact = seed_artifact(&db.pool).await;

    let (status, body) = app
        .call(Call::get("/admin/tools?search=quarterly", Principal::Admin))
        .await;
    assert_eq!(status, StatusCode::OK, "tools body: {body}");
    assert!(body.contains("report"), "tool name is rendered");
    assert!(body.contains("contract-server"), "server is rendered");
    assert!(
        body.contains("Quarterly report"),
        "artifact title is rendered"
    );

    let (status, body) = app
        .call(Call::get(
            "/admin/artifacts?search=quarterly",
            Principal::Admin,
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "artifacts body: {body}");
    assert!(
        body.contains("Quarterly report"),
        "artifact lens keeps the row"
    );

    let (status, body) = app
        .call(Call::get(
            &format!("/admin/artifacts/{artifact}"),
            Principal::Admin,
        ))
        .await;
    assert_eq!(status, StatusCode::OK, "artifact detail body: {body}");
    assert!(
        body.contains("Quarterly report"),
        "detail title is rendered"
    );
    assert!(
        body.contains("<code class=\"sp-code-inline\">report</code>"),
        "detail names the tool"
    );
    assert!(
        body.contains("&quot;answer&quot;: &quot;safe&quot;"),
        "stored body is pretty-printed"
    );
    assert!(
        body.contains("1 secret span(s) were redacted"),
        "secret redaction count is rendered with its label"
    );

    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn artifact_detail_requires_console_access_and_unknown_id_is_404() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let artifact = seed_artifact(&db.pool).await;

    let (status, _) = app
        .call(Call::get(
            &format!("/admin/artifacts/{artifact}"),
            Principal::NonAdmin,
        ))
        .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    let unknown = seed::unique("missing-artifact");
    let (status, body) = app
        .call(Call::get(
            &format!("/admin/artifacts/{unknown}"),
            Principal::Admin,
        ))
        .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "unknown artifact body: {body}"
    );

    db.cleanup().await;
}
