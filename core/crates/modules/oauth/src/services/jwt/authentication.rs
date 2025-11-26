use super::extraction::TokenExtractor;
use crate::services::validation::jwt as jwt_validation;
use anyhow::Result;
use axum::http::{HeaderMap, StatusCode};
use systemprompt_core_system::AppContext;
use systemprompt_models::auth::AuthenticatedUser;
use uuid::Uuid;

#[derive(Debug, Copy, Clone)]
pub struct AuthenticationService;

impl AuthenticationService {
    pub async fn authenticate(
        headers: &HeaderMap,
        context: &AppContext,
    ) -> Result<AuthenticatedUser, StatusCode> {
        let token = TokenExtractor::extract_bearer_token(headers)?;
        Self::validate_token_and_create_user(&token, context.jwt_secret())
    }

    pub async fn authenticate_with_token(
        token: &str,
        jwt_secret: &str,
    ) -> Result<AuthenticatedUser, StatusCode> {
        Self::validate_token_and_create_user(token, jwt_secret)
    }

    fn validate_token_and_create_user(
        token: &str,
        jwt_secret: &str,
    ) -> Result<AuthenticatedUser, StatusCode> {
        let claims = jwt_validation::validate_jwt_token(token, jwt_secret)
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::UNAUTHORIZED)?;

        let permissions = claims.get_permissions();

        Ok(AuthenticatedUser::new(
            user_id,
            claims.username.clone(),
            Some(claims.email),
            permissions,
        ))
    }
}
