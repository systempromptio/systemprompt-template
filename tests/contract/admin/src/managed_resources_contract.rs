//! Managed authoring preserves identity and instruction bytes through HTTP.

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};
use axum::http::StatusCode;

fn request<'a>(method: &'a str, path: &'a str, principal: Principal) -> Call<'a> {
    Call {
        method,
        path,
        principal,
        content_type: None,
        body: None,
    }
}

#[tokio::test]
async fn baseline_capture_requires_origin_and_organizational_revisions_are_shared_by_admins() {
    assert!(globals::init());
    let db = TempDb::create().await.expect("PostgreSQL required");
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let path = "/api/public/admin/managed/baselines/super-admin";
    let (status, _) = app.call(request("post", path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let origin = systemprompt::config::ProfileBootstrap::get()
        .unwrap()
        .server
        .api_external_url
        .clone();
    for principal in [Principal::ProjectManager, Principal::NonAdmin] {
        let (status, _) = app
            .response_headers_with(request("post", path, principal), &[("origin", &origin)])
            .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }
    let (status, _) = app
        .response_headers_with(
            request("post", path, Principal::Admin),
            &[("origin", &origin)],
        )
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let id: String = sqlx::query_scalar("SELECT v.id FROM managed_revisions v JOIN managed_resources r ON r.id=v.resource_id WHERE r.resource_key='admin_daily_brief'")
        .fetch_one(&*db.pool).await.expect("captured daily brief");
    let path = format!("/api/public/admin/managed/revisions/{id}");
    let (status, _) = app
        .call(request("get", &path, Principal::PlatformAdmin))
        .await;
    assert_eq!(
        status,
        StatusCode::OK,
        "authorized administrators share organizational revisions without changing the owner"
    );
    let (status, _) = app.call(request("get", &path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK);
    let path = format!("/admin/analysis/revisions/{id}/edit");
    let (status, body) = app.call(request("get", &path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert!(
        body.contains("---\nname: admin-daily-brief\ndescription:"),
        "layout must not indent editable source text"
    );
    let (status, _) = app
        .call(request("get", &path, Principal::PlatformAdmin))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_form_bytes(&app, &path, &origin).await;
    assert_bundle(&app, &id).await;
    db.cleanup().await;
}

async fn assert_form_bytes(app: &App, path: &str, origin: &str) {
    let baseline_text = std::fs::read_to_string(
        globals::repo_root().join("services/skills/admin_daily_brief/SKILL.md"),
    )
    .unwrap();
    let expected = format!("{baseline_text}\nReport missing evidence explicitly.\n");
    let transmitted = expected.replace('\n', "\r\n");
    let encoded = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("path", "SKILL.md")
        .append_pair("content", &transmitted)
        .append_pair("rationale", "Verify browser form transport")
        .finish();
    let (status, location) = app
        .redirect_with_headers(
            Call {
                method: "post",
                path,
                principal: Principal::Admin,
                content_type: Some("application/x-www-form-urlencoded"),
                body: Some(&encoded),
            },
            &[("origin", origin)],
        )
        .await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert!(location.contains("/compare/"));
    let candidate = location.rsplit('/').next().unwrap();
    let path = format!("/api/public/admin/managed/revisions/{candidate}/files");
    let (status, body) = app.call(request("get", &path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let value: serde_json::Value = serde_json::from_str(&body).unwrap();
    let bytes: Vec<u8> = serde_json::from_value(value["SKILL.md"]["bytes"].clone()).unwrap();
    assert_eq!(bytes, expected.as_bytes());
}

async fn assert_bundle(app: &App, id: &str) {
    use systemprompt::marketplace::managed::RevisionBundle;
    for path in [
        format!("/admin/analysis/revisions/{id}"),
        format!("/admin/analysis/revisions/{id}/edit"),
        format!("/admin/analysis/revisions/{id}/compare/{id}"),
    ] {
        let (status, _) = app
            .call(request("get", &path, Principal::ProjectManager))
            .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{path}");
    }
    let path = format!("/api/public/admin/managed/revisions/{id}/bundle");
    let (status, body) = app.call(request("get", &path, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK);
    let bundle: RevisionBundle = serde_json::from_str(&body).unwrap();
    assert_eq!(bundle.root.as_str(), id);
    assert_eq!(bundle.canonical_bytes().unwrap(), body.as_bytes());
    let (status, _) = app
        .call(request("get", &path, Principal::PlatformAdmin))
        .await;
    assert_eq!(status, StatusCode::OK);
}
