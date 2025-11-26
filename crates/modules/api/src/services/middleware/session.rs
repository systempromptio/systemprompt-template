use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use systemprompt_core_oauth::services::SessionCreationService;
use systemprompt_core_system::{services::AnalyticsService, AppContext};
use systemprompt_core_users::repository::UserRepository;
use systemprompt_identifiers::{AgentName, ClientId, ContextId, SessionId, TraceId, UserId};
use systemprompt_models::{auth::UserType, execution::context::RequestContext};
use uuid::Uuid;

use super::jwt_extractor::JwtExtractor;

// Note: We manually implement Clone to ensure session_creation_service
// is not cloned - each middleware instance shares the SAME Arc
#[derive(Clone, Debug)]
pub struct SessionMiddleware {
    jwt_extractor: Arc<JwtExtractor>,
    analytics_service: Arc<AnalyticsService>,
    jwt_secret: String,
    session_creation_service: Arc<SessionCreationService>,
}

impl SessionMiddleware {
    pub fn new(ctx: Arc<AppContext>) -> Self {
        let jwt_extractor = Arc::new(JwtExtractor::new(&ctx.config().jwt_secret));
        let session_creation_service = Arc::new(SessionCreationService::new(
            ctx.analytics_service().clone(),
            UserRepository::new(ctx.db_pool().clone()),
        ));

        Self {
            jwt_extractor,
            analytics_service: ctx.analytics_service().clone(),
            jwt_secret: ctx.config().jwt_secret.clone(),
            session_creation_service,
        }
    }

    pub async fn handle(&self, mut request: Request, next: Next) -> Result<Response, StatusCode> {
        let headers = request.headers();
        let uri = request.uri().clone();
        let method = request.method().clone();

        let should_skip = Self::should_skip_session_tracking(uri.path());

        let trace_id = headers
            .get("x-trace-id")
            .and_then(|h| h.to_str().ok())
            .map(|s| TraceId::new(s.to_string()))
            .unwrap_or_else(|| TraceId::new(format!("trace_{}", Uuid::new_v4())));

        let (req_ctx, jwt_cookie) = if should_skip {
            let ctx = RequestContext::new(
                SessionId::new(format!("untracked_{}", Uuid::new_v4())),
                trace_id,
                ContextId::new(String::new()),
                AgentName::system(),
            )
            .with_user_id(UserId::new("anonymous".to_string()))
            .with_user_type(UserType::Anon)
            .with_tracked(false);
            (ctx, None)
        } else {
            let analytics = self
                .analytics_service
                .extract_analytics(headers, Some(&uri));
            let is_bot = AnalyticsService::is_bot(&analytics);

            if is_bot {
                let ctx = RequestContext::new(
                    SessionId::new(format!("bot_{}", Uuid::new_v4())),
                    trace_id,
                    ContextId::new(String::new()),
                    AgentName::system(),
                )
                .with_user_id(UserId::new("bot".to_string()))
                .with_user_type(UserType::Anon)
                .with_tracked(false);
                (ctx, None)
            } else {
                let token_result = Self::extract_token(headers);

                let (session_id, user_id, jwt_token, jwt_cookie) = match token_result {
                    Ok(token) => match self.jwt_extractor.extract_user_context(&token) {
                        Ok(jwt_context) => {
                            (jwt_context.session_id, jwt_context.user_id, token, None)
                        },
                        Err(_) => {
                            let (sid, uid, new_token) =
                                self.create_new_session(headers, &uri, &method).await?;
                            (sid, uid, new_token.clone().unwrap_or_default(), new_token)
                        },
                    },
                    Err(_) => {
                        let (sid, uid, new_token) =
                            self.create_new_session(headers, &uri, &method).await?;
                        (sid, uid, new_token.clone().unwrap_or_default(), new_token)
                    },
                };

                let ctx = RequestContext::new(
                    session_id,
                    trace_id,
                    ContextId::new(String::new()),
                    AgentName::system(),
                )
                .with_user_id(user_id)
                .with_auth_token(jwt_token)
                .with_user_type(UserType::Anon)
                .with_tracked(true);
                (ctx, jwt_cookie)
            }
        };

        request.extensions_mut().insert(req_ctx);

        let mut response = next.run(request).await;

        if let Some(token) = jwt_cookie {
            let cookie = format!(
                "access_token={}; HttpOnly; SameSite=Lax; Path=/; Max-Age=604800",
                token
            );
            if let Ok(cookie_value) = cookie.parse() {
                response
                    .headers_mut()
                    .insert(header::SET_COOKIE, cookie_value);
            }
        }

        Ok(response)
    }

    async fn create_new_session(
        &self,
        headers: &http::HeaderMap,
        uri: &http::Uri,
        method: &http::Method,
    ) -> Result<(SessionId, UserId, Option<String>), StatusCode> {
        let client_id = ClientId::new("sp_web".to_string());

        match self
            .session_creation_service
            .create_anonymous_session(headers, Some(uri), &client_id, &self.jwt_secret)
            .await
        {
            Ok(session_info) => Ok((
                session_info.session_id,
                session_info.user_id,
                if session_info.is_new {
                    Some(session_info.jwt_token)
                } else {
                    None
                },
            )),
            Err(e) => {
                tracing::error!(
                    error = %e,
                    path = %uri.path(),
                    method = %method,
                    "Failed to create session - database unavailable"
                );
                Err(StatusCode::SERVICE_UNAVAILABLE)
            },
        }
    }

    fn extract_token(headers: &http::HeaderMap) -> Result<String, ()> {
        if let Some(auth_header) = headers.get(header::AUTHORIZATION) {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    return Ok(token.to_string());
                }
            }
        }

        if let Some(cookie_header) = headers.get(header::COOKIE) {
            if let Ok(cookie_str) = cookie_header.to_str() {
                for cookie in cookie_str.split(';') {
                    let cookie = cookie.trim();
                    if let Some(value) = cookie.strip_prefix("access_token=") {
                        return Ok(value.to_string());
                    }
                }
            }
        }

        Err(())
    }

    fn should_skip_session_tracking(path: &str) -> bool {
        if path.starts_with("/api/") {
            return true;
        }

        if path.starts_with("/_next/") {
            return true;
        }

        if path.starts_with("/static/")
            || path.starts_with("/assets/")
            || path.starts_with("/images/")
        {
            return true;
        }

        if path == "/health" || path == "/ready" || path == "/healthz" {
            return true;
        }

        if path == "/favicon.ico"
            || path == "/robots.txt"
            || path == "/sitemap.xml"
            || path == "/manifest.json"
        {
            return true;
        }

        if let Some(last_segment) = path.rsplit('/').next() {
            if last_segment.contains('.') {
                let extension = last_segment.rsplit('.').next().unwrap_or("");
                match extension {
                    "html" | "htm" => {},
                    _ => return true,
                }
            }
        }

        false
    }
}
