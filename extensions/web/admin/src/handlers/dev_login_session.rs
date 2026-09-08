//! Creates a normal authenticated session for a development login code.

use axum::http::HeaderMap;

use sqlx::PgPool;
use std::sync::Arc;

use systemprompt::analytics::SessionAnalyticsBuilder;
use systemprompt::identifiers::SessionSource;
use systemprompt::models::Config;
use systemprompt::models::auth::{Permission, RateLimitTier, UserType};
use systemprompt::oauth::SessionCreationService;
use systemprompt_security::{SessionGenerator, SessionParams};


use crate::repositories::dev_login::DevLoginUser;

pub(crate) fn session_cookie(jwt: &str, max_age: i64) -> String {
    format!(
        "access_token={jwt}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{}",
        if systemprompt::config::ProfileBootstrap::get()
            .is_ok_and(|p| p.server.api_external_url.starts_with("https://"))
        {
            "; Secure"
        } else {
            ""
        }
    )
}

// Why: the permissions match the current account roles rather than the
// roles it held when the development login code was issued —
// `handlers/webhook/governance/scope.rs` grants admin scope on
// `Permission::Admin`, so a session minted as `User` for someone in the admins
// group signs a token that disagrees with the account it was issued for. `User`
// is always present: holding `admin` is an addition to being a signed-in user,
// never a replacement.
pub fn permissions_for_roles(roles: &[String]) -> Vec<Permission> {
    if roles
        .iter()
        .any(|r| matches!(r.as_str(), "admin" | "platform_admin"))
    {
        vec![Permission::Admin, Permission::User]
    } else {
        vec![Permission::User]
    }
}

pub(crate) async fn mint_session(
    session_service: &SessionCreationService,
    subject: &DevLoginUser,
    headers: &HeaderMap,
) -> Result<(String, i64), String> {
    let session_id = session_service
        .create_authenticated_session(
            &subject.user_id,
            &SessionAnalyticsBuilder::new(headers).build(),
            SessionSource::Oauth,
        )
        .await
        .map_err(|e| e.to_string())?;

    let cfg = Config::get().map_err(|e| e.to_string())?;
    let permissions = permissions_for_roles(&subject.roles);
    let admin = permissions.contains(&Permission::Admin);
    let token = SessionGenerator::new(&cfg.jwt_issuer)
        .generate(&SessionParams {
            user_id: &subject.user_id,
            session_id: &session_id,
            email: &subject.email,
            duration: chrono::Duration::seconds(cfg.jwt_access_token_expiration),
            user_type: UserType::from_permissions(&permissions),
            permissions,
            roles: subject.roles.clone(),
            attributes: std::collections::BTreeMap::new(),
            rate_limit_tier: if admin {
                RateLimitTier::Admin
            } else {
                RateLimitTier::User
            },
        })
        .map_err(|error| error.to_string())?;
    let token = token.as_str().to_owned();

    Ok((token, cfg.jwt_access_token_expiration))
}

// Why: reuse core's normal session service without depending on any optional
// installation-specific identity provider. These handles share the primary
// pool.
pub(super) fn session_service(pool: &Arc<PgPool>) -> Result<SessionCreationService, String> {
    let db = Arc::new(systemprompt::database::Database::from_pools(
        Arc::clone(pool),
        Some(Arc::clone(pool)),
    ));
    let users = systemprompt::users::UserRepository::new(&db).map_err(|e| e.to_string())?;
    let analytics = systemprompt::analytics::repository::AnalyticsRepositories::new(&db)
        .map_err(|e| e.to_string())?;
    Ok(SessionCreationService::new(
        Arc::new(systemprompt::analytics::AnalyticsService::new(
            None, None, &analytics,
        )),
        Arc::new(systemprompt::users::UserService::new(Arc::new(users))),
    ))
}
