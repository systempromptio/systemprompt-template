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
        let token = Self::extract_token(headers)
            .ok_or_else(|| anyhow!("Missing authorization header"))?;

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
            .map_or_else(|| ContextId::new(String::new()), |s| ContextId::new(s.to_string()))
    }

    fn extract_agent_name(headers: &HeaderMap) -> AgentName {
        headers
            .get("x-agent-name")
            .and_then(|h| h.to_str().ok())
            .map_or_else(AgentName::system, |s| AgentName::new(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extract_token() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer test_token".parse().unwrap());

        let token = AuthValidationService::extract_token(&headers);
        assert_eq!(token, Some("test_token"));
    }

    #[tokio::test]
    async fn test_extract_token_missing() {
        let headers = HeaderMap::new();

        let token = AuthValidationService::extract_token(&headers);
        assert_eq!(token, None);
    }

    #[tokio::test]
    async fn test_create_anonymous_context() {
        let headers = HeaderMap::new();

        let ctx = AuthValidationService::create_anonymous_context(&headers);
        assert_eq!(ctx.auth.user_type, UserType::Anon);
        assert!(ctx.auth.user_id.is_anonymous());
    }

    #[tokio::test]
    async fn test_validate_request_optional_no_token() {
        let service = AuthValidationService::new("secret".to_string());
        let headers = HeaderMap::new();

        let result = service.validate_request(&headers, AuthMode::Optional).await;
        assert!(result.is_ok());

        let ctx = result.unwrap();
        assert_eq!(ctx.auth.user_type, UserType::Anon);
    }

    #[tokio::test]
    async fn test_validate_request_required_no_token() {
        let service = AuthValidationService::new("secret".to_string());
        let headers = HeaderMap::new();

        let result = service.validate_request(&headers, AuthMode::Required).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Missing authorization header"));
    }

    #[tokio::test]
    async fn test_validate_request_with_valid_token() {
        use chrono::Utc;
        use jsonwebtoken::{encode, EncodingKey, Header};
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        struct TestClaims {
            sub: String,
            session_id: String,
            roles: Vec<String>,
            exp: i64,
            aud: Vec<String>,
        }

        let secret = "test-secret";
        let service = AuthValidationService::new(secret.to_string());

        let claims = TestClaims {
            sub: "user123".to_string(),
            session_id: "test-session".to_string(),
            roles: vec!["user".to_string()],
            exp: (Utc::now() + chrono::Duration::hours(1)).timestamp(),
            aud: vec!["a2a".to_string(), "api".to_string()],
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );

        let result = service.validate_request(&headers, AuthMode::Required).await;
        assert!(result.is_ok());

        let ctx = result.unwrap();
        assert_eq!(ctx.auth.user_id.as_str(), "user123");
        assert_eq!(ctx.auth.user_type, UserType::Standard);
    }

    #[tokio::test]
    async fn test_validate_request_with_admin_token() {
        use chrono::Utc;
        use jsonwebtoken::{encode, EncodingKey, Header};
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Serialize, Deserialize)]
        struct TestClaims {
            sub: String,
            session_id: String,
            roles: Vec<String>,
            exp: i64,
            aud: Vec<String>,
        }

        let secret = "test-secret";
        let service = AuthValidationService::new(secret.to_string());

        let claims = TestClaims {
            sub: "admin123".to_string(),
            session_id: "test-session".to_string(),
            roles: vec!["admin".to_string()],
            exp: (Utc::now() + chrono::Duration::hours(1)).timestamp(),
            aud: vec!["a2a".to_string(), "api".to_string()],
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap();

        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            format!("Bearer {}", token).parse().unwrap(),
        );

        let result = service.validate_request(&headers, AuthMode::Required).await;
        assert!(result.is_ok());

        let ctx = result.unwrap();
        assert_eq!(ctx.auth.user_id.as_str(), "admin123");
        assert_eq!(ctx.auth.user_type, UserType::Admin);
    }

    #[tokio::test]
    async fn test_validate_request_with_invalid_token() {
        let service = AuthValidationService::new("secret".to_string());

        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer invalid_token".parse().unwrap());

        let result = service.validate_request(&headers, AuthMode::Required).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid JWT token"));
    }

    #[tokio::test]
    async fn test_validate_request_optional_with_invalid_token() {
        let service = AuthValidationService::new("secret".to_string());

        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer invalid_token".parse().unwrap());

        let result = service.validate_request(&headers, AuthMode::Optional).await;
        assert!(result.is_ok());

        let ctx = result.unwrap();
        assert_eq!(ctx.auth.user_type, UserType::Anon);
    }

    #[tokio::test]
    async fn test_extract_trace_id_from_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-trace-id", "trace-123".parse().unwrap());

        let trace_id = AuthValidationService::extract_trace_id(&headers);
        assert_eq!(trace_id.as_str(), "trace-123");
    }

    #[tokio::test]
    async fn test_extract_trace_id_generates_if_missing() {
        let headers = HeaderMap::new();

        let trace_id = AuthValidationService::extract_trace_id(&headers);
        assert!(trace_id.as_str().starts_with("trace_"));
    }
}
