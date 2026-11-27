//! Credentials the suite presents, and the request authenticator that admits
//! exactly those.
//!
//! The admin handlers verify every bearer through the injected
//! `DynRequestAuthenticator`. The suite injects a table: each token it issues
//! maps to fixed claims and the audiences it was issued for, and a token
//! asked for under an audience it does not carry is refused exactly as a
//! real one would be. Role membership is not carried here: the middleware
//! reads it from the `users` rows the suite seeds.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, OnceLock};

use systemprompt::identifiers::{JwtToken, SessionId, UserId};
use systemprompt::models::auth::{
    DynRequestAuthenticator, JwtAudience, JwtUserContext, Permission, RequestAuthenticator,
};
use systemprompt::models::execution::context::{ContextExtractionError, RequestContext};
use systemprompt::traits::AuthUser;

struct Issued {
    audiences: Vec<JwtAudience>,
    context: JwtUserContext,
    user: AuthUser,
}

fn registry() -> &'static Mutex<HashMap<String, Issued>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, Issued>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub struct Grant<'a> {
    pub subject: &'a str,
    pub email: &'a str,
    pub permission: Permission,
    pub scopes: Vec<Permission>,
    pub audiences: Vec<JwtAudience>,
}

pub fn issue(grant: &Grant<'_>) -> String {
    let now = chrono::Utc::now().timestamp();
    let user_id = UserId::new(grant.subject.to_owned());
    let context = JwtUserContext {
        user_id: user_id.clone(),
        session_id: SessionId::new(uuid::Uuid::new_v4().to_string()),
        role: grant.permission,
        scope: grant.scopes.clone(),
        user_type: grant.permission.as_user_type(),
        client_id: None,
        act_chain: Vec::new(),
        attributes: BTreeMap::new(),
        jti: None,
        exp: now + 3600,
        purpose: None,
        interaction: None,
        browser_session: true,
        ui_session: false,
        auth_time: now,
    };
    let user = AuthUser {
        id: user_id,
        name: grant.subject.to_owned(),
        email: grant.email.to_owned(),
        roles: vec!["user".to_owned()],
        is_active: true,
        is_service_account: false,
    };
    let token = format!("contract-{}", uuid::Uuid::new_v4().simple());
    registry()
        .lock()
        .expect("the token registry is not poisoned")
        .insert(
            token.clone(),
            Issued {
                audiences: grant.audiences.clone(),
                context,
                user,
            },
        );
    token
}

pub fn session_of(token: &str) -> SessionId {
    registry()
        .lock()
        .expect("the token registry is not poisoned")
        .get(token)
        .map(|issued| issued.context.session_id.clone())
        .expect("the token was issued by this suite")
}

fn refused(token: &JwtToken) -> ContextExtractionError {
    ContextExtractionError::InvalidToken(format!("{} was not issued", token.as_str()).into())
}

#[derive(Debug)]
struct IssuedTokens;

#[async_trait::async_trait]
impl RequestAuthenticator for IssuedTokens {
    async fn authenticate(
        &self,
        _headers: &axum::http::HeaderMap,
        _audiences: &[JwtAudience],
    ) -> Result<RequestContext, ContextExtractionError> {
        Err(ContextExtractionError::MissingAuthHeader)
    }

    async fn decode_for_audiences(
        &self,
        token: &JwtToken,
        audiences: &[JwtAudience],
    ) -> Result<(JwtUserContext, AuthUser), ContextExtractionError> {
        let registry = registry()
            .lock()
            .expect("the token registry is not poisoned");
        let issued = registry.get(token.as_str()).ok_or_else(|| refused(token))?;
        if !issued.audiences.iter().any(|aud| audiences.contains(aud)) {
            return Err(refused(token));
        }
        Ok((issued.context.clone(), issued.user.clone()))
    }

    async fn decode_for_gateway(
        &self,
        token: &JwtToken,
    ) -> Result<(JwtUserContext, AuthUser), ContextExtractionError> {
        Err(refused(token))
    }

    async fn load_user(
        &self,
        _user_id: &UserId,
        _route_context: &str,
    ) -> Result<AuthUser, ContextExtractionError> {
        Err(ContextExtractionError::MissingAuthHeader)
    }
}

pub fn authenticator() -> DynRequestAuthenticator {
    Arc::new(IssuedTokens)
}
