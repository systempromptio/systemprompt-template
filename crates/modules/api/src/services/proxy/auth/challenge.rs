use axum::{body::Body, http::StatusCode, response::Response};
use serde_json::json;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::AppContext;

#[derive(Debug, Clone, Copy)]
pub struct OAuthChallengeBuilder;

impl OAuthChallengeBuilder {
    pub async fn build_challenge_response(
        service_name: &str,
        ctx: &AppContext,
        status_code: StatusCode,
    ) -> Result<Response<Body>, StatusCode> {
        let log = LogService::system(ctx.db_pool().clone());
        let _ = log
            .warn(
                "api_oauth_challenge",
                &format!(
                    "Building OAuth challenge for service: {} (status: {})",
                    service_name, status_code
                ),
            )
            .await;

        let oauth_base_url = &ctx.config().api_server_url;

        let auth_header_value = format!(
            "Bearer realm=\"{}\", as_uri=\"{}/.well-known/oauth-authorization-server\", error=\"invalid_token\"",
            service_name, oauth_base_url
        );

        let error_body = json!({
            "error": if status_code == StatusCode::UNAUTHORIZED { "invalid_token" } else { "insufficient_scope" },
            "error_description": if status_code == StatusCode::UNAUTHORIZED {
                "The access token is missing or invalid"
            } else {
                "The access token does not have the required scope"
            },
            "server": service_name,
            "authorization_url": format!("{}/.well-known/oauth-authorization-server", oauth_base_url)
        });

        // OAuth challenge response details logged above

        Response::builder()
            .status(status_code)
            .header("Content-Type", "application/json")
            .header("WWW-Authenticate", auth_header_value)
            .body(Body::from(error_body.to_string()))
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
    }
}
