pub mod manifest;
pub mod plugin_walker;
pub mod types;
pub mod whoami;

use axum::body::Body;
use axum::http::{HeaderMap, Response, StatusCode};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt::models::auth::JwtAudience;
use systemprompt::models::{Config, SecretsBootstrap};
use systemprompt::oauth::validate_jwt_token;

use crate::handlers::shared;

use self::types::UserSection;

pub(super) fn validate_cowork_jwt(
    headers: &HeaderMap,
) -> Result<UserId, Box<Response<Body>>> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            shared::boxed_error_response(StatusCode::UNAUTHORIZED, "Missing Authorization header")
        })?;

    let jwt_secret = SecretsBootstrap::jwt_secret().map_err(|e| {
        tracing::error!(error = %e, "Failed to load JWT secret");
        shared::boxed_error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Internal configuration error",
        )
    })?;

    let jwt_issuer = Config::get()
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to load config");
            shared::boxed_error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal configuration error",
            )
        })?
        .jwt_issuer
        .clone();

    let claims = validate_jwt_token(token, jwt_secret, &jwt_issuer, &[JwtAudience::Api])
        .map_err(|e| {
            tracing::warn!(error = %e, "Cowork JWT validation failed");
            shared::boxed_error_response(StatusCode::UNAUTHORIZED, "Invalid or expired token")
        })?;

    Ok(UserId::new(&claims.sub))
}

pub(super) async fn load_user_section(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<UserSection>, sqlx::Error> {
    sqlx::query!(
        r#"SELECT id, name, email, display_name,
                  COALESCE(roles, '{}') as "roles!: Vec<String>"
           FROM users WHERE id = $1"#,
        user_id.as_str(),
    )
    .fetch_optional(pool)
    .await
    .map(|opt| {
        opt.map(|r| UserSection {
            id: r.id,
            name: r.name,
            email: r.email,
            display_name: r.display_name,
            roles: r.roles,
        })
    })
}
