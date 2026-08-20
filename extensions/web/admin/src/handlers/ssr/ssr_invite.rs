//! `/admin/invite/{token}` — the public invite-accept page.
//!
//! Public by design: the token in the URL is the authorization. The page
//! validates it server-side so the invitee sees who invited them and to what
//! before creating a passkey; the actual provisioning happens when the page's
//! script posts the token to `/admin/auth/invite/accept`.

use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::response::{Html, IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::oauth::services::webauthn::hash_token;

use crate::error::AdminHtmlResult;
use crate::repositories::users::invites;
use crate::templates::AdminTemplateEngine;

use super::{branding_context, context};

#[derive(Debug, Serialize)]
struct InviteAcceptContext<'a> {
    #[serde(flatten)]
    shell: context::BrandingShell<'a>,
    valid: bool,
    email: String,
    org_name: String,
    department: String,
    token: String,
}

pub(crate) async fn invite_accept_page(
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(token): Path<String>,
) -> AdminHtmlResult<Response> {
    let token = token.trim().to_owned();
    let invite = if token.is_empty() {
        None
    } else {
        invites::find_valid_invite_by_hash(&pool, &hash_token(&token))
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "invite lookup failed");
                None
            })
    };

    let ctx = InviteAcceptContext {
        shell: branding_context(&engine),
        valid: invite.is_some(),
        email: invite.as_ref().map(|i| i.email.clone()).unwrap_or_default(),
        department: invite
            .as_ref()
            .map(|i| i.department.clone())
            .unwrap_or_default(),
        org_name: invite
            .as_ref()
            .map(|i| i.org_name.clone())
            .unwrap_or_default(),
        token,
    };
    let html = engine.render("invite-accept", &ctx)?;
    Ok(Html(html).into_response())
}
