use super::extraction::TokenExtractor;
use crate::models::JwtClaims;
use crate::services::validation::{audience, jwt as jwt_validation};
use axum::http::{HeaderMap, StatusCode};
use std::str::FromStr;
use systemprompt_core_system::AppContext;
use systemprompt_models::auth::{AuthenticatedUser, JwtAudience};
use uuid::Uuid;

#[derive(Debug, Copy, Clone)]
pub struct AuthorizationService;

impl AuthorizationService {
    pub async fn authorize_service_access(
        headers: &HeaderMap,
        service_name: &str,
        context: &AppContext,
    ) -> Result<AuthenticatedUser, StatusCode> {
        let token = TokenExtractor::extract_bearer_token(headers)?;

        let claims = jwt_validation::validate_jwt_token(&token, context.jwt_secret())
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        if !audience::validate_service_access(&claims.aud, service_name) {
            return Err(StatusCode::FORBIDDEN);
        }

        Self::create_authenticated_user_from_claims(claims)
    }

    pub async fn authorize_required_audience(
        headers: &HeaderMap,
        required_audience: &str,
        context: &AppContext,
    ) -> Result<AuthenticatedUser, StatusCode> {
        let token = TokenExtractor::extract_bearer_token(headers)?;

        let claims = jwt_validation::validate_jwt_token(&token, context.jwt_secret())
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let required_aud =
            JwtAudience::from_str(required_audience).map_err(|_| StatusCode::BAD_REQUEST)?;

        if !audience::validate_required_audience(&claims.aud, required_aud) {
            return Err(StatusCode::FORBIDDEN);
        }

        Self::create_authenticated_user_from_claims(claims)
    }

    pub async fn authorize_any_audience(
        headers: &HeaderMap,
        allowed_audiences: &[&str],
        context: &AppContext,
    ) -> Result<AuthenticatedUser, StatusCode> {
        let token = TokenExtractor::extract_bearer_token(headers)?;

        let claims = jwt_validation::validate_jwt_token(&token, context.jwt_secret())
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let allowed_auds: Vec<JwtAudience> = allowed_audiences
            .iter()
            .filter_map(|s| JwtAudience::from_str(s).ok())
            .collect();

        if !audience::validate_any_audience(&claims.aud, &allowed_auds) {
            return Err(StatusCode::FORBIDDEN);
        }

        Self::create_authenticated_user_from_claims(claims)
    }

    fn create_authenticated_user_from_claims(
        claims: JwtClaims,
    ) -> Result<AuthenticatedUser, StatusCode> {
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
