use crate::repository::OAuthRepository;
use crate::services::validate_jwt_token;
use anyhow::Result;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Form, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]

pub struct IntrospectRequest {
    pub token: String,
    pub token_type_hint: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
}

#[derive(Debug, Serialize)]

pub struct IntrospectResponse {
    pub active: bool,
    pub scope: Option<String>,
    pub client_id: Option<String>,
    pub username: Option<String>,
    pub token_type: Option<String>,

    pub exp: Option<i64>,

    pub iat: Option<i64>,
    pub sub: Option<String>,
    #[serde(default)]
    pub aud: Vec<String>,
    pub iss: Option<String>,
    pub jti: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct IntrospectError {
    pub error: String,
    pub error_description: Option<String>,
}

pub async fn handle_introspect(
    State(ctx): State<systemprompt_core_system::AppContext>,
    Form(request): Form<IntrospectRequest>,
) -> impl IntoResponse {
    let repo = OAuthRepository::new(ctx.db_pool().clone());
    if let Some(client_id) = &request.client_id {
        if validate_client_credentials(&repo, client_id, request.client_secret.as_deref())
            .await
            .is_err()
        {
            let error = IntrospectError {
                error: "invalid_client".to_string(),
                error_description: Some("Invalid client credentials".to_string()),
            };
            return (StatusCode::UNAUTHORIZED, Json(error)).into_response();
        }
    }

    match introspect_token(&repo, &request.token).await {
        Ok(response) => (StatusCode::OK, Json(response)).into_response(),
        Err(error) => {
            let error = IntrospectError {
                error: "server_error".to_string(),
                error_description: Some(error.to_string()),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error)).into_response()
        },
    }
}

async fn validate_client_credentials(
    repo: &OAuthRepository,
    client_id: &str,
    client_secret: Option<&str>,
) -> Result<()> {
    let client = repo
        .find_client(client_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Client not found"))?;

    if let Some(secret) = client_secret {
        use crate::services::verify_client_secret;
        let hash = client
            .client_secret_hash
            .as_deref()
            .ok_or_else(|| anyhow::anyhow!("Client has no secret hash configured"))?;
        if !verify_client_secret(secret, hash)? {
            return Err(anyhow::anyhow!("Invalid client secret"));
        }
    } else {
        return Err(anyhow::anyhow!("Client secret required"));
    }

    Ok(())
}

async fn introspect_token(_repo: &OAuthRepository, token: &str) -> Result<IntrospectResponse> {
    let jwt_secret = &systemprompt_core_system::Config::global().jwt_secret;
    match validate_jwt_token(token, jwt_secret) {
        Ok(claims) => Ok(IntrospectResponse {
            active: true,
            scope: Some(systemprompt_models::auth::permissions_to_string(
                &claims.scope,
            )),
            client_id: claims.client_id.clone(),
            username: Some(claims.username),
            token_type: Some("Bearer".to_string()),
            exp: Some(claims.exp),
            iat: Some(claims.iat),
            sub: Some(claims.sub),
            aud: claims.aud.iter().map(ToString::to_string).collect(),
            iss: Some(claims.iss),
            jti: Some(claims.jti),
        }),
        Err(_) => Ok(IntrospectResponse {
            active: false,
            scope: None,
            client_id: None,
            username: None,
            token_type: None,
            exp: None,
            iat: None,
            sub: None,
            aud: Vec::new(),
            iss: None,
            jti: None,
        }),
    }
}
