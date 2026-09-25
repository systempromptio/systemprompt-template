//! The session mint every browser sign-in ends in: create the authenticated
//! session row, sign the JWT, and spell the `access_token` cookie that
//! carries it. ADFS and the development login link both land here, so the
//! two can never drift on what a session is.

use axum::http::HeaderMap;

use systemprompt::identifiers::UserId;

use systemprompt::analytics::SessionAnalyticsBuilder;
use systemprompt::identifiers::SessionSource;
use systemprompt::models::Config;
use systemprompt::models::auth::{AuthenticatedUser, Permission};
use systemprompt::oauth::SessionCreationService;
use systemprompt::oauth::services::{
    JwtConfig, JwtSigningParams, generate_access_token_jti, generate_jwt,
};

use super::secure_flag;
use crate::repositories::dev_login::DevLoginUser;
use crate::repositories::users::federated;

// Why: the account a session is minted for, reduced to what the mint needs.
// Each sign-in path resolves an identity its own way and carries its own
// side facts; none of those belong in the JWT.
#[derive(Debug, Clone)]
pub(crate) struct SessionSubject {
    pub user_id: UserId,
    pub email: String,
    pub display_name: String,
    pub roles: Vec<String>,
}

impl From<&federated::ResolvedFederatedUser> for SessionSubject {
    fn from(resolved: &federated::ResolvedFederatedUser) -> Self {
        Self {
            user_id: resolved.user_id.clone(),
            email: resolved.email.clone(),
            display_name: resolved.display_name.clone(),
            roles: resolved.roles.clone(),
        }
    }
}

impl From<DevLoginUser> for SessionSubject {
    fn from(user: DevLoginUser) -> Self {
        Self {
            user_id: user.user_id,
            email: user.email,
            display_name: user.display_name,
            roles: user.roles,
        }
    }
}

pub(crate) fn session_cookie(jwt: &str, max_age: i64) -> String {
    format!(
        "access_token={jwt}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{}",
        secure_flag()
    )
}

// Why: the permissions an assertion's mapped roles carry. The roles the AD
// group map produces are stored on the user row, but a JWT is checked on
// `permissions`, not on roles —
// `handlers/webhook/governance/scope.rs` grants admin scope on
// `Permission::Admin`, so a session minted as `User` for someone in the admins
// group signs a token that disagrees with the account it was issued for. `User`
// is always present: holding `admin` is an addition to being a signed-in user,
// never a replacement.
pub fn permissions_for_roles(roles: &[String]) -> Vec<Permission> {
    if crate::types::roles_grant_manage(roles) {
        vec![Permission::Admin, Permission::User]
    } else {
        vec![Permission::User]
    }
}

pub(crate) async fn mint_session(
    session_service: &SessionCreationService,
    subject: &SessionSubject,
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
    let uuid = subject
        .user_id
        .as_str()
        .parse()
        .unwrap_or_else(|_| uuid::Uuid::nil());
    let permissions = permissions_for_roles(&subject.roles);
    let user = AuthenticatedUser::new_with_roles(
        uuid,
        subject.display_name.clone(),
        subject.email.clone(),
        permissions.clone(),
        subject.roles.clone(),
    );

    let jwt_config = JwtConfig {
        permissions,
        audience: cfg.jwt_audiences.clone(),
        expires_in: chrono::Duration::seconds(cfg.jwt_access_token_expiration),
        resource: None,
        plugin_id: None,
        // Why: a browser session is minted for a person, not for a registered
        // OAuth client, so there is no client to name.
        client_id: None,
    };
    let signing = JwtSigningParams {
        issuer: &cfg.jwt_issuer,
    };
    let jti = generate_access_token_jti();
    let token =
        generate_jwt(&user, jwt_config, jti, &session_id, &signing).map_err(|e| e.to_string())?;

    Ok((token, cfg.jwt_access_token_expiration))
}
