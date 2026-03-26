use chrono::Duration;
use systemprompt::identifiers::{SessionId, UserId};
use systemprompt::models::auth::{Permission, RateLimitTier, UserType};
use systemprompt::models::{Config, SecretsBootstrap};
use systemprompt::security::{SessionGenerator, SessionParams};

pub fn generate_plugin_token(
    user_id: &UserId,
    email: &str,
    plugin_id: &str,
) -> Result<String, crate::error::MarketplaceError> {
    let jwt_secret = SecretsBootstrap::jwt_secret().map_err(|e| {
        crate::error::MarketplaceError::Internal(format!("Failed to load JWT secret: {e}"))
    })?;
    let config = Config::get().map_err(|e| {
        crate::error::MarketplaceError::Internal(format!("Failed to load config: {e}"))
    })?;
    let issuer = &config.jwt_issuer;

    let generator = SessionGenerator::new(jwt_secret, issuer);
    let session_id = SessionId::new(format!("plugin_{plugin_id}"));

    let params = SessionParams {
        user_id,
        session_id: &session_id,
        email,
        duration: Duration::days(365),
        user_type: UserType::Service,
        permissions: vec![Permission::Service],
        roles: vec![],
        rate_limit_tier: RateLimitTier::Service,
    };

    let token = generator.generate(&params).map_err(|e| {
        crate::error::MarketplaceError::Internal(format!("Failed to generate token: {e}"))
    })?;
    Ok(token.as_str().to_string())
}
