use anyhow::{anyhow, Result};
use axum::http::HeaderMap;
use systemprompt_identifiers::{AgentName, ContextId, SessionId, TraceId, UserId};
use systemprompt_models::auth::UserType;
use systemprompt_models::execution::context::RequestContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMode {
    Required,
    Optional,
    Disabled,
}

#[derive(Debug, Clone)]
pub struct TokenClaims {
    pub user_id: String,
    pub session_id: String,
    pub user_type: UserType,
    pub roles: Vec<String>,
}

#[derive(Debug)]
pub struct AuthValidationService {
    jwt_secret: String,
}

impl AuthValidationService {
    pub const fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }

    pub async fn validate_request(
        &self,
        headers: &HeaderMap,
        mode: AuthMode,
    ) -> Result<RequestContext> {
        match mode {
            AuthMode::Required => self.validate_and_fail_fast(headers).await,
            AuthMode::Optional => self.try_validate_or_anonymous(headers).await,
            AuthMode::Disabled => Ok(Self::create_test_context()),
        }
    }

    async fn validate_and_fail_fast(&self, headers: &HeaderMap) -> Result<RequestContext> {
        let token =
            Self::extract_token(headers).ok_or_else(|| anyhow!("Missing authorization header"))?;

        let claims = self.validate_token(token).await?;
        Self::create_context_from_claims(&claims, token, headers)
    }

    async fn try_validate_or_anonymous(&self, headers: &HeaderMap) -> Result<RequestContext> {
        match Self::extract_token(headers) {
            Some(token) => match self.validate_token(token).await {
                Ok(claims) => Self::create_context_from_claims(&claims, token, headers),
                Err(_) => Ok(Self::create_anonymous_context(headers)),
            },
            None => Ok(Self::create_anonymous_context(headers)),
        }
    }

    fn extract_token(headers: &HeaderMap) -> Option<&str> {
        headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
    }

    async fn validate_token(&self, token: &str) -> Result<TokenClaims> {
        use jsonwebtoken::{decode, DecodingKey, Validation};
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        struct Claims {
            sub: String,
            session_id: Option<String>,
            roles: Option<Vec<String>>,
            exp: i64,
        }

        let mut validation = Validation::default();
        validation.set_audience(&["a2a", "api", "mcp"]);

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(|e| anyhow!("Invalid JWT token: {e}"))?;

        let claims = token_data.claims;

        let user_type = if claims
            .roles
            .as_ref()
            .is_some_and(|r| r.contains(&"admin".to_string()))
        {
            UserType::Admin
        } else {
            UserType::Standard
        };

        Ok(TokenClaims {
            user_id: claims.sub,
            session_id: claims.session_id.unwrap_or_else(|| "unknown".to_string()),
            user_type,
            roles: claims.roles.unwrap_or_default(),
        })
    }

    fn create_context_from_claims(
        claims: &TokenClaims,
        token: &str,
        headers: &HeaderMap,
    ) -> Result<RequestContext> {
        let session_id = SessionId::new(claims.session_id.clone());
        let user_id = UserId::new(claims.user_id.clone());

        let trace_id = Self::extract_trace_id(headers);
        let context_id = Self::extract_context_id(headers);
        let agent_name = Self::extract_agent_name(headers);

        let ctx = RequestContext::new(session_id, trace_id, context_id, agent_name)
            .with_user_id(user_id)
            .with_auth_token(token)
            .with_user_type(claims.user_type);

        Ok(ctx)
    }

    fn create_anonymous_context(headers: &HeaderMap) -> RequestContext {
        let trace_id = Self::extract_trace_id(headers);
        let context_id = Self::extract_context_id(headers);
        let agent_name = Self::extract_agent_name(headers);

        RequestContext::new(
            SessionId::new("anonymous".to_string()),
            trace_id,
            context_id,
            agent_name,
        )
        .with_user_id(UserId::anonymous())
        .with_user_type(UserType::Anon)
    }

    fn create_test_context() -> RequestContext {
        RequestContext::new(
            SessionId::new("test".to_string()),
            TraceId::new("test-trace".to_string()),
            ContextId::new("test-context".to_string()),
            AgentName::new("test-agent".to_string()),
        )
        .with_user_id(UserId::new("test-user".to_string()))
        .with_user_type(UserType::Standard)
    }

    fn extract_trace_id(headers: &HeaderMap) -> TraceId {
        headers
            .get("x-trace-id")
            .and_then(|h| h.to_str().ok())
            .map_or_else(
                || TraceId::new(format!("trace_{}", uuid::Uuid::new_v4())),
                |s| TraceId::new(s.to_string()),
            )
    }

    fn extract_context_id(headers: &HeaderMap) -> ContextId {
        headers
            .get("x-context-id")
            .and_then(|h| h.to_str().ok())
            .map_or_else(
                || ContextId::new(String::new()),
                |s| ContextId::new(s.to_string()),
            )
    }

    fn extract_agent_name(headers: &HeaderMap) -> AgentName {
        headers
            .get("x-agent-name")
            .and_then(|h| h.to_str().ok())
            .map_or_else(AgentName::system, |s| AgentName::new(s.to_string()))
    }
}
