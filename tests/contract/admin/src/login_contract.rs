//! The sign-in page resolves an existing session before rendering its SSO
//! prompt, so a signed-in browser returns to the appropriate landing page.

use axum::http::StatusCode;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

const LOGIN: &str = "/admin/login";

#[tokio::test(flavor = "multi_thread")]
async fn login_redirects_existing_sessions_to_their_landing_page() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping login redirect suite");
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (anonymous_status, _) = app.call(Call::get(LOGIN, Principal::Anonymous)).await;
    assert_eq!(anonymous_status, StatusCode::OK);

    for principal in [
        Principal::Admin,
        Principal::PlatformAdmin,
        Principal::ProjectManager,
    ] {
        let (status, target) = app.redirect_of(Call::get(LOGIN, principal)).await;
        assert_eq!(status, StatusCode::SEE_OTHER, "{principal:?}");
        assert_eq!(target, "/admin/", "{principal:?}");
    }

    for principal in [
        Principal::NonAdmin,
        Principal::Developer,
        Principal::KnowledgeWorker,
    ] {
        let (status, target) = app.redirect_of(Call::get(LOGIN, principal)).await;
        assert_eq!(status, StatusCode::SEE_OTHER, "{principal:?}");
        assert_eq!(target, "/admin/profile", "{principal:?}");
    }

    db.cleanup().await;
}
