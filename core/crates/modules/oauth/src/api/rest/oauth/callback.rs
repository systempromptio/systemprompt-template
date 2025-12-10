use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect};
use serde::Deserialize;
use std::str::FromStr;
use systemprompt_core_users::repository::UserRepository;
use systemprompt_models::auth::{parse_permissions, AuthenticatedUser, Permission};

use crate::repository::OAuthRepository;

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: Option<String>,
}

pub async fn handle_callback(
    Query(params): Query<CallbackQuery>,
    State(ctx): State<systemprompt_core_system::AppContext>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let repo = OAuthRepository::new(ctx.db_pool().clone());
    let config = systemprompt_core_system::Config::global();

    let server_base_url = &config.api_external_url;
    let redirect_uri = format!("{server_base_url}/api/v1/core/oauth/callback");

    let browser_client = match find_browser_client(&repo, &redirect_uri).await {
        Ok(client) => client,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to find OAuth client: {e}"),
            )
                .into_response();
        },
    };

    let token_response = match exchange_code_for_token(
        &repo,
        &params.code,
        &browser_client.client_id,
        &redirect_uri,
        &ctx,
        &headers,
    )
    .await
    {
        Ok(response) => response,
        Err(e) => {
            return (
                StatusCode::UNAUTHORIZED,
                format!("Failed to exchange code for token: {e}"),
            )
                .into_response();
        },
    };

    let redirect_destination = params
        .state
        .as_deref()
        .filter(|s| !s.is_empty())
        .unwrap_or("/");

    let cookie = format!(
        "access_token={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=3600",
        token_response.access_token
    );

    let mut response = Redirect::to(redirect_destination).into_response();
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.parse().unwrap());

    response
}

async fn find_browser_client(
    repo: &OAuthRepository,
    redirect_uri: &str,
) -> anyhow::Result<BrowserClient> {
    let clients = repo.list_clients().await?;

    for client in clients {
        if client.redirect_uris.contains(&redirect_uri.to_string())
            && (client.scopes.contains(&"admin".to_string())
                || client.scopes.contains(&"user".to_string()))
        {
            return Ok(BrowserClient {
                client_id: client.client_id,
            });
        }
    }

    Err(anyhow::anyhow!("No suitable browser client found"))
}

async fn exchange_code_for_token(
    repo: &OAuthRepository,
    code: &str,
    client_id: &str,
    redirect_uri: &str,
    ctx: &systemprompt_core_system::AppContext,
    headers: &HeaderMap,
) -> anyhow::Result<TokenResponse> {
    use crate::services::{
        generate_access_token_jti, generate_jwt, generate_secure_token, JwtConfig,
    };

    let (user_id, scope) = repo
        .validate_authorization_code(code, client_id, Some(redirect_uri), None)
        .await?;

    let user = load_authenticated_user(&user_id, ctx.db_pool().clone()).await?;

    let permissions = parse_permissions(&scope)?;

    // Create session for authenticated user with analytics
    let user_id_typed = systemprompt_identifiers::UserId::new(user_id.clone());
    let session_service = crate::services::SessionCreationService::new(
        ctx.analytics_service().clone(),
        UserRepository::new(ctx.db_pool().clone()),
    );
    let session_id = session_service
        .create_authenticated_session(&user_id_typed, headers)
        .await?;

    let access_token_jti = generate_access_token_jti();
    let config = JwtConfig {
        permissions: permissions.clone(),
        ..Default::default()
    };
    let jwt_secret = &systemprompt_core_system::Config::global().jwt_secret;
    let access_token = generate_jwt(&user, config, access_token_jti, &session_id, jwt_secret)?;

    let refresh_token_value = generate_secure_token("rt");
    let refresh_expires_at = chrono::Utc::now().timestamp() + (86400 * 30);

    repo.store_refresh_token(
        &refresh_token_value,
        client_id,
        &user_id,
        &scope,
        refresh_expires_at,
    )
    .await?;

    Ok(TokenResponse { access_token })
}

async fn load_authenticated_user(
    user_id: &str,
    db_pool: systemprompt_core_database::DbPool,
) -> anyhow::Result<AuthenticatedUser> {
    let user_repo = UserRepository::new(db_pool.clone());

    let user = user_repo
        .get_by_id(user_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("User not found: {user_id}"))?;

    let permissions: Vec<Permission> = user
        .roles
        .iter()
        .filter_map(|s| Permission::from_str(s).ok())
        .collect();

    let user_uuid = uuid::Uuid::parse_str(user.id.as_ref())
        .map_err(|_| anyhow::anyhow!("Invalid user UUID: {}", user.id))?;

    let email = if user.email.is_empty() {
        None
    } else {
        Some(user.email)
    };

    Ok(AuthenticatedUser::new(
        user_uuid,
        user.name,
        email,
        permissions,
    ))
}

#[derive(Debug)]
struct BrowserClient {
    client_id: String,
}

#[derive(Debug, serde::Deserialize)]
struct TokenResponse {
    access_token: String,
}
