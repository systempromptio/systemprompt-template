use axum::http::{HeaderMap, StatusCode};
use systemprompt_core_logging::LogService;
use systemprompt_core_oauth::services::AuthService;
use systemprompt_core_system::{AppContext, RequestContext};
use systemprompt_models::auth::AuthenticatedUser;

#[derive(Debug, Clone, Copy)]
pub struct AuthValidator {}

impl AuthValidator {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn validate_service_access(
        headers: &HeaderMap,
        service_name: &str,
        ctx: &AppContext,
        req_context: Option<&RequestContext>,
    ) -> Result<AuthenticatedUser, StatusCode> {
        let logger = if let Some(req_ctx) = req_context {
            LogService::new(ctx.db_pool().clone(), req_ctx.log_context())
        } else {
            LogService::system(ctx.db_pool().clone())
        };

        let debug_auth = std::env::var("DEBUG_AUTH_LOGGING")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);

        if debug_auth {
            let trace_id = req_context
                .map(|rc| rc.trace_id().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            logger
                .info(
                    "proxy_auth",
                    &format!(
                        "🔍 auth validation starting for {} [trace: {}]",
                        service_name, trace_id
                    ),
                )
                .await
                .ok();
        }

        let result = AuthService::authorize_service_access(headers, service_name, ctx).await;

        match &result {
            Ok(user) => {
                if debug_auth {
                    let trace_id = req_context
                        .map(|rc| rc.trace_id().to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    logger
                        .info(
                            "proxy_auth",
                            &format!(
                                "✅ {} auth success for {} [trace: {}]",
                                service_name, user.username, trace_id
                            ),
                        )
                        .await
                        .ok();
                }
            },
            Err(status) => {
                let trace_id = req_context
                    .map(|rc| rc.trace_id().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                logger
                    .warn(
                        "proxy_auth",
                        &format!(
                            "❌ {} auth failed with status {} [trace: {}]",
                            service_name, status, trace_id
                        ),
                    )
                    .await
                    .ok();
            },
        }

        result
    }

    pub fn has_mcp_session(headers: &HeaderMap) -> bool {
        headers.contains_key("mcp-session-id")
            || headers.contains_key("x-mcp-session")
            || headers.contains_key("x-mcp-session-id")
    }

    pub fn requires_auth(auth_type: &str) -> bool {
        auth_type == "oauth"
    }
}
