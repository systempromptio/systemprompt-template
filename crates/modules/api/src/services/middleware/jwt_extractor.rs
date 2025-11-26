use anyhow::{anyhow, Result};
use async_trait::async_trait;
use axum::http::HeaderMap;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use std::sync::Arc;
use uuid::Uuid;

use systemprompt_core_database::DbPool;
use systemprompt_core_logging::{LogContext, LogLevel, LogService};
use systemprompt_core_oauth::models::JwtClaims;
use systemprompt_core_users::repository::UserRepository;
use systemprompt_identifiers::{AgentName, ClientId, ContextId, SessionId, TraceId, UserId};
use systemprompt_models::auth::UserType;
use systemprompt_models::execution::context::{ContextExtractionError, RequestContext};

#[derive(Debug, Clone)]
pub struct JwtUserContext {
    pub user_id: UserId,
    pub session_id: SessionId,
    pub role: systemprompt_models::auth::Permission,
    pub user_type: UserType,
    pub client_id: Option<ClientId>,
}

pub struct JwtExtractor {
    decoding_key: DecodingKey,
    validation: Validation,
}

impl std::fmt::Debug for JwtExtractor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtExtractor")
            .field("decoding_key", &"<DecodingKey>")
            .field("validation", &self.validation)
            .finish()
    }
}

impl JwtExtractor {
    pub fn new(jwt_secret: &str) -> Self {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.validate_aud = false;

        Self {
            decoding_key: DecodingKey::from_secret(jwt_secret.as_bytes()),
            validation,
        }
    }

    pub fn validate_token(&self, token: &str) -> Result<(), String> {
        match decode::<JwtClaims>(token, &self.decoding_key, &self.validation) {
            Ok(_) => Ok(()),
            Err(err) => {
                let reason = err.to_string();
                if reason.contains("InvalidSignature") || reason.contains("invalid signature") {
                    Err("Invalid signature".to_string())
                } else if reason.contains("ExpiredSignature") || reason.contains("token expired") {
                    Err("Token expired".to_string())
                } else if reason.contains("MissingRequiredClaim") || reason.contains("missing") {
                    Err("Missing required claim".to_string())
                } else {
                    Err("Invalid token".to_string())
                }
            },
        }
    }

    pub fn extract_user_context(&self, token: &str) -> Result<JwtUserContext> {
        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &self.validation)?;

        let session_id_str = token_data
            .claims
            .session_id
            .ok_or_else(|| anyhow!("JWT must contain session_id claim"))?;

        let role = token_data
            .claims
            .scope
            .first()
            .ok_or_else(|| anyhow!("JWT must contain valid scope claim"))?
            .clone();

        let client_id = token_data.claims.client_id.map(|cid| ClientId::new(cid));

        Ok(JwtUserContext {
            user_id: UserId::new(token_data.claims.sub),
            session_id: SessionId::new(session_id_str),
            role,
            user_type: token_data.claims.user_type,
            client_id,
        })
    }
}

#[derive(Debug, Clone)]
pub struct JwtContextExtractor {
    jwt_extractor: Arc<JwtExtractor>,
    db_pool: DbPool,
}

impl JwtContextExtractor {
    pub fn new(jwt_secret: String, db_pool: DbPool) -> Self {
        let jwt_extractor = Arc::new(JwtExtractor::new(&jwt_secret));

        Self {
            jwt_extractor,
            db_pool,
        }
    }

    fn extract_token_from_authorization(
        headers: &HeaderMap,
    ) -> Result<String, ContextExtractionError> {
        let auth_header = headers
            .get("authorization")
            .ok_or(ContextExtractionError::MissingAuthHeader)?;

        let auth_str =
            auth_header
                .to_str()
                .map_err(|e| ContextExtractionError::InvalidHeaderValue {
                    header: "authorization".to_string(),
                    reason: e.to_string(),
                })?;

        if !auth_str.starts_with("Bearer ") {
            return Err(ContextExtractionError::InvalidToken(
                "Authorization header must start with 'Bearer '".to_string(),
            ));
        }

        Ok(auth_str[7..].to_string())
    }

    fn extract_token_from_cookie(headers: &HeaderMap) -> Result<String, ContextExtractionError> {
        let cookie_header = headers
            .get("cookie")
            .ok_or(ContextExtractionError::MissingAuthHeader)?
            .to_str()
            .map_err(|e| ContextExtractionError::InvalidHeaderValue {
                header: "cookie".to_string(),
                reason: e.to_string(),
            })?;

        for cookie in cookie_header.split(';') {
            let cookie = cookie.trim();
            if let Some(value) = cookie.strip_prefix("access_token=") {
                return Ok(value.to_string());
            }
        }

        Err(ContextExtractionError::MissingAuthHeader)
    }

    fn extract_token(headers: &HeaderMap) -> Result<String, ContextExtractionError> {
        Self::extract_token_from_authorization(headers)
            .or_else(|_| Self::extract_token_from_cookie(headers))
    }

    async fn extract_jwt_context(
        &self,
        headers: &HeaderMap,
    ) -> Result<JwtUserContext, ContextExtractionError> {
        let token = Self::extract_token(headers)?;
        self.jwt_extractor
            .extract_user_context(&token)
            .map_err(|e| ContextExtractionError::InvalidToken(e.to_string()))
    }

    pub async fn extract_standard(
        &self,
        headers: &HeaderMap,
    ) -> Result<RequestContext, ContextExtractionError> {
        let has_auth = headers.get("authorization").is_some();
        let has_user_id_header = headers.get("x-user-id").is_some();
        let has_session_header = headers.get("x-session-id").is_some();
        let has_context_headers = has_user_id_header && has_session_header;

        // Security: Context headers without JWT = spoofing attempt
        if has_context_headers && !has_auth {
            return Err(ContextExtractionError::ForbiddenHeader {
                header: "X-User-ID/X-Session-ID".to_string(),
                reason: "Context headers require valid JWT for authentication".to_string(),
            });
        }

        // Extract and validate JWT (required for both external and internal calls)
        let jwt_context = self.extract_jwt_context(headers).await?;

        if jwt_context.session_id.as_str().is_empty() {
            return Err(ContextExtractionError::MissingSessionId);
        }

        if jwt_context.user_id.as_str().is_empty() {
            return Err(ContextExtractionError::MissingUserId);
        }

        let user_repo = UserRepository::new(self.db_pool.clone());
        let user_exists = user_repo
            .get_by_id(jwt_context.user_id.as_str())
            .await
            .map_err(|e| {
                ContextExtractionError::DatabaseError(format!(
                    "Failed to check user existence: {}",
                    e
                ))
            })?;

        if user_exists.is_none() {
            let log_context = LogContext::new()
                .with_session_id(jwt_context.session_id.as_str())
                .with_user_id(jwt_context.user_id.as_str());
            let logger = LogService::new(self.db_pool.clone(), log_context);
            logger
                .log(
                    LogLevel::Info,
                    "auth",
                    &format!(
                        "JWT validation failed: User {} no longer exists in database",
                        jwt_context.user_id.as_str()
                    ),
                    None,
                )
                .await
                .ok();

            return Err(ContextExtractionError::UserNotFound(format!(
                "User {} no longer exists",
                jwt_context.user_id.as_str()
            )));
        }

        // Internal call: Use forwarded headers (agent already validated user)
        // External call: Use JWT claims directly
        let session_id = if has_session_header {
            headers
                .get("x-session-id")
                .and_then(|h| h.to_str().ok())
                .map(|s| SessionId::new(s.to_string()))
                .unwrap_or_else(|| jwt_context.session_id.clone())
        } else {
            jwt_context.session_id.clone()
        };

        let user_id = if has_user_id_header {
            headers
                .get("x-user-id")
                .and_then(|h| h.to_str().ok())
                .map(|s| UserId::new(s.to_string()))
                .unwrap_or_else(|| jwt_context.user_id.clone())
        } else {
            jwt_context.user_id.clone()
        };

        let trace_id = headers
            .get("x-trace-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| TraceId::new(s.to_string()))
            .unwrap_or_else(|| TraceId::new(format!("trace_{}", Uuid::new_v4())));

        let context_id = headers
            .get("x-context-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| ContextId::new(s.to_string()))
            .unwrap_or_else(|| ContextId::new(String::new()));

        let task_id = headers
            .get("x-task-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| systemprompt_identifiers::TaskId::new(s.to_string()));

        let auth_token = headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(|s| s.to_string());

        let agent_name = headers
            .get("x-agent-name")
            .and_then(|h| h.to_str().ok())
            .map(|s| AgentName::new(s.to_string()))
            .unwrap_or_else(|| AgentName::system());

        let mut request_context = RequestContext::new(session_id, trace_id, context_id, agent_name);

        request_context = request_context
            .with_user_id(user_id)
            .with_user_type(jwt_context.user_type);

        if let Some(client_id) = jwt_context.client_id {
            request_context = request_context.with_client_id(client_id);
        }

        if let Some(t_id) = task_id {
            request_context = request_context.with_task_id(t_id);
        }

        if let Some(token) = auth_token {
            request_context = request_context.with_auth_token(token);
        }

        Ok(request_context)
    }

    pub async fn extract_mcp_a2a(
        &self,
        headers: &HeaderMap,
    ) -> Result<RequestContext, ContextExtractionError> {
        self.extract_standard(headers).await
    }
}

use axum::body::Body;
use axum::extract::Request;
use systemprompt_core_system::middleware::ContextExtractor;

#[async_trait]
impl ContextExtractor for JwtContextExtractor {
    async fn extract_from_headers(
        &self,
        headers: &HeaderMap,
    ) -> Result<RequestContext, ContextExtractionError> {
        // Delegate to extract_standard which handles both external and internal calls
        self.extract_standard(headers).await
    }

    async fn extract_from_request(
        &self,
        request: Request<Body>,
    ) -> Result<(RequestContext, Request<Body>), ContextExtractionError> {
        use systemprompt_core_system::middleware::sources::PayloadSource;

        let headers = request.headers().clone();

        let has_auth = headers.get("authorization").is_some();
        let has_context_id_header = headers.get("x-context-id").is_some();

        // For A2A routes: context ID must be in body (not header)
        if has_context_id_header && !has_auth {
            return Err(ContextExtractionError::ForbiddenHeader {
                header: "X-Context-ID".to_string(),
                reason:
                    "Context ID must be in request body (A2A spec). Use contextId field in message."
                        .to_string(),
            });
        }

        let jwt_context = self.extract_jwt_context(&headers).await?;

        if jwt_context.session_id.as_str().is_empty() {
            return Err(ContextExtractionError::MissingSessionId);
        }

        if jwt_context.user_id.as_str().is_empty() {
            return Err(ContextExtractionError::MissingUserId);
        }

        let user_repo = UserRepository::new(self.db_pool.clone());
        let user_exists = user_repo
            .get_by_id(jwt_context.user_id.as_str())
            .await
            .map_err(|e| {
                ContextExtractionError::DatabaseError(format!(
                    "Failed to check user existence: {}",
                    e
                ))
            })?;

        if user_exists.is_none() {
            let log_context = LogContext::new()
                .with_session_id(jwt_context.session_id.as_str())
                .with_user_id(jwt_context.user_id.as_str());
            let logger = LogService::new(self.db_pool.clone(), log_context);
            logger
                .log(
                    LogLevel::Info,
                    "auth",
                    &format!(
                        "JWT validation failed: User {} no longer exists in database (A2A route)",
                        jwt_context.user_id.as_str()
                    ),
                    None,
                )
                .await
                .ok();

            return Err(ContextExtractionError::UserNotFound(format!(
                "User {} no longer exists",
                jwt_context.user_id.as_str()
            )));
        }

        let trace_id = headers
            .get("x-trace-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| TraceId::new(s.to_string()))
            .unwrap_or_else(|| TraceId::new(format!("trace_{}", Uuid::new_v4())));

        let (body_bytes, reconstructed_request) =
            PayloadSource::read_and_reconstruct(request).await?;
        let context_id_str = PayloadSource::extract_context_id(&body_bytes).await?;
        let context_id = ContextId::new(context_id_str);

        let task_id = headers
            .get("x-task-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| systemprompt_identifiers::TaskId::new(s.to_string()));

        let auth_token = headers
            .get("authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(|s| s.to_string());

        let agent_name = headers
            .get("x-agent-name")
            .and_then(|h| h.to_str().ok())
            .map(|s| AgentName::new(s.to_string()))
            .unwrap_or_else(|| AgentName::system());

        let mut request_context = RequestContext::new(
            jwt_context.session_id.clone(),
            trace_id,
            context_id,
            agent_name,
        );

        request_context = request_context
            .with_user_id(jwt_context.user_id.clone())
            .with_user_type(jwt_context.user_type);

        if let Some(client_id) = jwt_context.client_id {
            request_context = request_context.with_client_id(client_id);
        }

        if let Some(t_id) = task_id {
            request_context = request_context.with_task_id(t_id);
        }

        if let Some(token) = auth_token {
            request_context = request_context.with_auth_token(token);
        }

        Ok((request_context, reconstructed_request))
    }

    async fn extract_user_only(
        &self,
        headers: &HeaderMap,
    ) -> Result<RequestContext, ContextExtractionError> {
        // Delegate to extract_standard which handles both external and internal calls
        self.extract_standard(headers).await
    }
}
