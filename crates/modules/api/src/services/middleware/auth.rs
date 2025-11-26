use axum::{extract::Request, http::HeaderMap, middleware, middleware::Next, response::Response};

#[derive(Debug, Clone)]
pub struct ApiAuthMiddlewareConfig {
    pub public_paths: Vec<String>,
}

impl Default for ApiAuthMiddlewareConfig {
    fn default() -> Self {
        Self {
            public_paths: vec![
                "/api/v1/core/oauth/session".to_string(),
                "/api/v1/core/oauth/register".to_string(),
                "/api/v1/core/oauth/authorize".to_string(),
                "/api/v1/core/oauth/token".to_string(),
                "/api/v1/core/oauth/callback".to_string(),
                "/api/v1/core/oauth/consent".to_string(),
                "/api/v1/core/oauth/webauthn/complete".to_string(),
                "/.well-known".to_string(),
                "/api/v1/stream".to_string(),
                "/api/v1/core/contexts/webhook".to_string(),
                "/api/v1".to_string(),
            ],
        }
    }
}

impl ApiAuthMiddlewareConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_public_path(&self, path: &str) -> bool {
        if !path.starts_with("/api") && !path.starts_with("/.well-known") {
            return true;
        }

        self.public_paths.iter().any(|p| path.starts_with(p)) || path.starts_with("/.well-known")
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AuthMiddleware;

impl AuthMiddleware {
    pub fn apply_auth_layer(router: axum::Router) -> axum::Router {
        router.layer(middleware::from_fn(move |req, next| {
            let config = ApiAuthMiddlewareConfig::default();
            async move { auth_middleware(config, req, next).await }
        }))
    }
}

pub async fn auth_middleware(
    config: ApiAuthMiddlewareConfig,
    mut req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path();

    if config.is_public_path(path) {
        return next.run(req).await;
    }

    if let Some(user) = extract_optional_user(req.headers()).await {
        req.extensions_mut().insert(user);
    }

    next.run(req).await
}

async fn extract_optional_user(
    headers: &HeaderMap,
) -> Option<systemprompt_core_system::AuthenticatedUser> {
    use systemprompt_core_oauth::{extract_bearer_token, extract_cookie_token, validate_jwt_token};
    use systemprompt_models::auth::UserType;
    use uuid::Uuid;

    let token = extract_cookie_token(headers)
        .or_else(|_| extract_bearer_token(headers))
        .ok()?;

    if token.trim().is_empty() {
        return None;
    }

    let jwt_secret = &systemprompt_core_system::Config::global().jwt_secret;
    let claims = validate_jwt_token(&token, jwt_secret).ok()?;

    let user_id = Uuid::parse_str(&claims.sub).ok()?;

    let email = if claims.email.is_empty() || claims.user_type == UserType::Anon {
        None
    } else {
        Some(claims.email.clone())
    };

    let permissions = claims.scope;

    Some(systemprompt_core_system::AuthenticatedUser::new(
        user_id,
        claims.username.clone(),
        email,
        permissions,
    ))
}
