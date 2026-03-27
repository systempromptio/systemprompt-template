use axum::http::HeaderMap;
use systemprompt::models::{Config, SecretsBootstrap};

pub(super) fn extract_bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get("authorization")
        .and_then(|v| {
            v.to_str()
                .map_err(|e| {
                    tracing::warn!(error = %e, "Non-ASCII authorization header");
                })
                .ok()
        })
        .and_then(|v| v.strip_prefix("Bearer "))
}

pub(super) fn get_jwt_config() -> Result<(String, String), anyhow::Error> {
    let secret = SecretsBootstrap::jwt_secret()?.to_string();
    let issuer = Config::get()?.jwt_issuer.clone();
    Ok((secret, issuer))
}
