pub mod plugin_file;
pub mod types;
pub mod whoami;

use axum::body::Body;
use axum::http::{HeaderMap, Response, StatusCode};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt::models::auth::JwtAudience;
use systemprompt::models::Config;
use systemprompt::oauth::validate_jwt_token;

use crate::handlers::shared;

use self::types::UserSection;

pub(super) fn validate_cowork_jwt(headers: &HeaderMap) -> Result<UserId, Box<Response<Body>>> {
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| {
            shared::boxed_error_response(StatusCode::UNAUTHORIZED, "Missing Authorization header")
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

    let claims = validate_jwt_token(token, &jwt_issuer, &[JwtAudience::Bridge])
        .map_err(|err| {
            tracing::warn!(error = %err, "Cowork JWT validation failed");
            shared::boxed_error_response(StatusCode::UNAUTHORIZED, "Invalid or expired token")
        })?;

    Ok(UserId::new(&claims.sub))
}

pub(super) async fn load_user_section(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<UserSection>, sqlx::Error> {
    let row = crate::repositories::cowork_grp::find_cowork_user(pool, user_id.as_str()).await?;
    Ok(row.map(|r| UserSection {
        id: r.id,
        name: r.name,
        email: r.email,
        display_name: r.display_name,
        roles: r.roles,
    }))
}
