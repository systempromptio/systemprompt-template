use anyhow::Result;
use axum::extract::Request;
use axum::http::HeaderMap;
use systemprompt_core_oauth::{
    extract_bearer_token, extract_cookie_token, validate_jwt_token, SessionCreationService,
};
use systemprompt_core_system::AppContext;
use systemprompt_core_users::repository::UserRepository;
use systemprompt_identifiers::{ClientId, SessionId, UserId};

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub is_new: bool,
    pub jwt_token: Option<String>,
}

pub async fn ensure_session(request: &Request, ctx: &AppContext) -> Result<SessionInfo> {
    let headers = request.headers();

    if let Ok(token) = extract_cookie_token(headers) {
        if let Ok(claims) = validate_jwt_token(&token, ctx.jwt_secret()) {
            return Ok(SessionInfo {
                session_id: SessionId::new(claims.session_id.unwrap_or_default()),
                user_id: UserId::new(claims.sub),
                is_new: false,
                jwt_token: Some(token),
            });
        }
    }

    if let Ok(token) = extract_bearer_token(headers) {
        if let Ok(claims) = validate_jwt_token(&token, ctx.jwt_secret()) {
            return Ok(SessionInfo {
                session_id: SessionId::new(claims.session_id.unwrap_or_default()),
                user_id: UserId::new(claims.sub),
                is_new: false,
                jwt_token: Some(token),
            });
        }
    }

    let session_service = SessionCreationService::new(
        ctx.analytics_service().clone(),
        UserRepository::new(ctx.db_pool().clone()),
    );

    let client_id = ClientId::new("sp_web".to_string());
    let session_info = session_service
        .create_anonymous_session(
            request.headers(),
            Some(request.uri()),
            &client_id,
            ctx.jwt_secret(),
        )
        .await?;

    Ok(SessionInfo {
        session_id: session_info.session_id,
        user_id: session_info.user_id,
        is_new: session_info.is_new,
        jwt_token: Some(session_info.jwt_token),
    })
}

pub async fn ensure_session_from_headers(
    headers: &HeaderMap,
    ctx: &AppContext,
) -> Result<SessionInfo> {
    if let Ok(token) = extract_cookie_token(headers) {
        if let Ok(claims) = validate_jwt_token(&token, ctx.jwt_secret()) {
            return Ok(SessionInfo {
                session_id: SessionId::new(claims.session_id.unwrap_or_default()),
                user_id: UserId::new(claims.sub),
                is_new: false,
                jwt_token: Some(token),
            });
        }
    }

    if let Ok(token) = extract_bearer_token(headers) {
        if let Ok(claims) = validate_jwt_token(&token, ctx.jwt_secret()) {
            return Ok(SessionInfo {
                session_id: SessionId::new(claims.session_id.unwrap_or_default()),
                user_id: UserId::new(claims.sub),
                is_new: false,
                jwt_token: Some(token),
            });
        }
    }

    let session_service = SessionCreationService::new(
        ctx.analytics_service().clone(),
        UserRepository::new(ctx.db_pool().clone()),
    );

    let client_id = ClientId::new("sp_web".to_string());
    let session_info = session_service
        .create_anonymous_session(headers, None, &client_id, ctx.jwt_secret())
        .await?;

    Ok(SessionInfo {
        session_id: session_info.session_id,
        user_id: session_info.user_id,
        is_new: session_info.is_new,
        jwt_token: Some(session_info.jwt_token),
    })
}
