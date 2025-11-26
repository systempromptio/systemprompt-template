use crate::TokenValidator;
use async_trait::async_trait;
use systemprompt_models::auth::{AuthError, AuthenticatedUser};
use uuid::Uuid;

use crate::services::validation::jwt;

#[derive(Clone, Debug)]
pub struct JwtTokenValidator {
    jwt_secret: String,
}

impl JwtTokenValidator {
    pub const fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }
}

#[async_trait]
impl TokenValidator for JwtTokenValidator {
    async fn validate_token(&self, token: &str) -> Result<AuthenticatedUser, AuthError> {
        let claims = jwt::validate_jwt_token(token, &self.jwt_secret).map_err(|e| {
            AuthError::AuthenticationFailed {
                message: format!("JWT validation failed: {e}"),
            }
        })?;

        let user_id =
            Uuid::parse_str(&claims.sub).map_err(|e| AuthError::AuthenticationFailed {
                message: format!("Invalid user ID in token: {e}"),
            })?;

        let permissions = claims.get_permissions();

        Ok(AuthenticatedUser::new(
            user_id,
            claims.username.clone(),
            Some(claims.email),
            permissions,
        ))
    }
}
