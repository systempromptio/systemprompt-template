//! Console readers cannot use administrative mutation routes.

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};
use axum::http::StatusCode;

#[tokio::test]
async fn console_reader_cannot_revoke_sessions_or_post_to_admin_pages() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision_dashboard(&db.pool).await;
    let sessions = format!(
        "/api/public/admin/users/{}/sessions",
        credentials.non_admin_user_id
    );
    let app = App::new(&db.pool, credentials);
    let (read, _) = app
        .call(Call::get(&sessions, Principal::ProjectManager))
        .await;
    assert_eq!(read, StatusCode::OK, "console readers may inspect sessions");
    let (write, _) = app
        .call(Call::json(
            "delete",
            &sessions,
            Principal::ProjectManager,
            "{}",
        ))
        .await;
    assert_eq!(
        write,
        StatusCode::FORBIDDEN,
        "the read route must not inherit DELETE authority"
    );
    let (audit, _) = app
        .call(Call::get(
            "/api/public/admin/gateway/acl/detect",
            Principal::ProjectManager,
        ))
        .await;
    assert_eq!(
        audit,
        StatusCode::FORBIDDEN,
        "the audit-producing legacy GET remains admin-only"
    );
    let (ssr, _) = app
        .call(Call::json(
            "post",
            SSR_MUTATION,
            Principal::ProjectManager,
            "{}",
        ))
        .await;
    assert_eq!(
        ssr,
        StatusCode::FORBIDDEN,
        "console access must not authorize SSR mutations"
    );
    for path in [
        "/admin/contexts/00000000-0000-4000-8000-000000000000",
        "/admin/requests/unknown",
        "/admin/api/chain/unknown",
        "/admin/api/conversations/unknown/raw",
    ] {
        let (evidence, _) = app.call(Call::get(path, Principal::ProjectManager)).await;
        assert_eq!(
            evidence,
            StatusCode::FORBIDDEN,
            "console access cannot reveal raw evidence: {path}"
        );
    }
    db.cleanup().await;
}

const SSR_MUTATION: &str = "/admin/evals/run";
