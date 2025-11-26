use crate::services::shared::error::{AgentServiceError, Result};
use jsonwebtoken::{decode, DecodingKey, Validation};
pub use systemprompt_core_oauth::models::JwtClaims;

pub struct JwtValidator {
    decoding_key: DecodingKey,
    validation: Validation,
}

impl std::fmt::Debug for JwtValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtValidator")
            .field("validation", &self.validation)
            .field("decoding_key", &"<decoding_key>")
            .finish()
    }
}

impl JwtValidator {
    pub fn new(secret: String) -> Self {
        Self {
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
            validation: Validation::default(),
        }
    }

    pub fn validate_token(&self, token: &str) -> Result<JwtClaims> {
        decode::<JwtClaims>(token, &self.decoding_key, &self.validation)
            .map(|data| data.claims)
            .map_err(|e| AgentServiceError::Authentication(format!("invalid token: {}", e)))
    }
}

pub fn extract_bearer_token(authorization_header: &str) -> Result<&str> {
    if let Some(token) = authorization_header.strip_prefix("Bearer ") {
        Ok(token)
    } else {
        Err(AgentServiceError::Authentication(
            "invalid authorization header format".to_string(),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct AgentSessionUser {
    pub id: String,
    pub username: String,
    pub user_type: String,
    pub roles: Vec<String>,
}

impl From<JwtClaims> for AgentSessionUser {
    fn from(claims: JwtClaims) -> Self {
        Self {
            id: claims.sub.clone(),
            username: claims.username.clone(),
            user_type: claims.user_type.to_string(),
            roles: claims.get_scopes(),
        }
    }
}
