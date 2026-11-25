//! The session mint every browser sign-in ends in: create the authenticated
//! session row, sign the JWT, and spell the `access_token` cookie that
//! carries it. ADFS and the development login link both land here, so the
//! two can never drift on what a session is.

use systemprompt::identifiers::UserId;
use systemprompt::models::auth::Permission;

use super::secure_flag;
use crate::repositories::dev_login::DevLoginUser;
use crate::repositories::users::federated;

// Why: the account a session is minted for, reduced to what the mint needs.
// Each sign-in path resolves an identity its own way and carries its own
// side facts; none of those belong in the JWT.
#[derive(Debug, Clone)]
pub(crate) struct SessionSubject {
    pub user_id: UserId,
}

impl From<&federated::ResolvedFederatedUser> for SessionSubject {
    fn from(resolved: &federated::ResolvedFederatedUser) -> Self {
        Self {
            user_id: resolved.user_id.clone(),
        }
    }
}

impl From<DevLoginUser> for SessionSubject {
    fn from(user: DevLoginUser) -> Self {
        Self {
            user_id: user.user_id,
        }
    }
}

pub(crate) fn session_cookie(jwt: &str, max_age: i64, secure: bool) -> String {
    format!(
        "access_token={jwt}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{}",
        secure_flag(secure)
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

// Why: a browser session is a core-audience access token, and an install
// member holds no token authority: `SessionCreationService` needs one to be
// built, and the conferred minter refuses every core audience. A sign-in
// that reaches this point has verified its assertion and resolved its
// account, and is refused here rather than signed with anything a core
// verifier would not trust. Core's `SsoSlot` is the seam a browser sign-in
// provider is contributed through.
pub(crate) fn mint_session(subject: &SessionSubject) -> Result<(String, i64), String> {
    tracing::warn!(
        user_id = %subject.user_id,
        "browser session refused: an install member cannot mint a core session token"
    );
    Err("browser session minting is not conferred on an install member".to_owned())
}
