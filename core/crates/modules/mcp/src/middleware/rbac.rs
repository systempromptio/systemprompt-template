use rmcp::{service::RequestContext as McpContext, ErrorData as McpError, RoleServer};
use systemprompt_core_config::services::ConfigLoader;
use systemprompt_core_logging::LogService;
use systemprompt_core_oauth::services::validation::jwt::validate_jwt_token;
use systemprompt_core_system::{Config, RequestContext};
use systemprompt_identifiers::UserId;
use systemprompt_models::auth::AuthenticatedUser;

use super::context_extraction::extract_request_context;
use super::jwt::extract_bearer_token;

#[derive(Debug, Clone)]
pub struct AuthenticatedRequestContext {
    pub context: RequestContext,
    pub auth_token: String,
}

impl AuthenticatedRequestContext {
    pub const fn new(context: RequestContext, auth_token: String) -> Self {
        Self {
            context,
            auth_token,
        }
    }

    pub fn token(&self) -> &str {
        &self.auth_token
    }
}

impl std::ops::Deref for AuthenticatedRequestContext {
    type Target = RequestContext;

    fn deref(&self) -> &Self::Target {
        &self.context
    }
}

#[derive(Debug)]
pub enum AuthResult {
    Anonymous(RequestContext),
    Authenticated(AuthenticatedRequestContext),
}

impl AuthResult {
    pub const fn context(&self) -> &RequestContext {
        match self {
            Self::Anonymous(ctx) => ctx,
            Self::Authenticated(auth_ctx) => &auth_ctx.context,
        }
    }

    pub fn context_mut(&mut self) -> &mut RequestContext {
        match self {
            Self::Anonymous(ctx) => ctx,
            Self::Authenticated(auth_ctx) => &mut auth_ctx.context,
        }
    }

    pub fn expect_authenticated(self, msg: &str) -> AuthenticatedRequestContext {
        match self {
            Self::Authenticated(auth_ctx) => auth_ctx,
            Self::Anonymous(_) => panic!("{}", msg),
        }
    }
}

pub async fn enforce_rbac_from_registry(
    ctx: &McpContext<RoleServer>,
    server_name: &str,
    logger: Option<&LogService>,
) -> Result<AuthResult, McpError> {
    let services_config = ConfigLoader::load().await.map_err(|e| {
        McpError::internal_error(format!("Failed to load services config: {e}"), None)
    })?;

    let deployment = services_config
        .mcp_servers
        .get(server_name)
        .ok_or_else(|| {
            McpError::internal_error(
                format!("MCP server '{server_name}' not found in registry"),
                None,
            )
        })?;

    let oauth_config = &deployment.oauth;

    let mut request_context = extract_request_context(ctx)?;

    if !oauth_config.required {
        return Ok(AuthResult::Anonymous(request_context));
    }

    let token = extract_bearer_token(ctx)?.ok_or_else(|| {
        if let Some(log) = logger {
            let error_msg = format!(
                "Authentication required for server '{server_name}': No Bearer token provided"
            );
            let _ = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { log.error("mcp_rbac", &error_msg).await })
            });
        }
        McpError::invalid_request(
            format!(
                "Authentication required. Server '{server_name}' requires OAuth but no Bearer token provided."
            ),
            None,
        )
    })?;

    let jwt_secret = &Config::global().jwt_secret;
    let claims = validate_jwt_token(&token, jwt_secret).map_err(|e| {
        if let Some(log) = logger {
            let error_msg = format!("JWT validation failed for server '{server_name}': {e}");
            let _ = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { log.error("mcp_rbac", &error_msg).await })
            });
        }
        McpError::invalid_request(format!("Invalid JWT token: {e}"), None)
    })?;

    if !claims.aud.contains(&oauth_config.audience) {
        if let Some(log) = logger {
            let error_msg = format!(
                "Invalid audience for server '{}': Expected '{}', got: {:?}",
                server_name, oauth_config.audience, claims.aud
            );
            let _ = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { log.error("mcp_rbac", &error_msg).await })
            });
        }
        return Err(McpError::invalid_request(
            format!(
                "Invalid audience. Expected '{}', got: {:?}",
                oauth_config.audience, claims.aud
            ),
            None,
        ));
    }

    let user_scopes = claims.get_scopes();
    let required_scopes = &oauth_config.scopes;

    let has_required_scope = required_scopes.iter().any(|required_scope| {
        let required_str = required_scope.to_string();
        user_scopes.contains(&required_str)
    });

    if !has_required_scope {
        if let Some(log) = logger {
            let error_msg = format!(
                "Insufficient permissions for server '{server_name}': Required one of {required_scopes:?}, but user has: {user_scopes:?}"
            );
            let _ = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { log.error("mcp_rbac", &error_msg).await })
            });
        }
        return Err(McpError::invalid_request(
            format!(
                "Insufficient permissions. User must have one of: {required_scopes:?}, but has: {user_scopes:?}"
            ),
            None,
        ));
    }

    let user_id = claims.sub.parse().map_err(|e| {
        if let Some(log) = logger {
            let error_msg = format!("Invalid user ID in JWT for server '{server_name}': {e}");
            let _ = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { log.error("mcp_rbac", &error_msg).await })
            });
        }
        McpError::internal_error(format!("Invalid user ID in JWT: {e}"), None)
    })?;

    let permissions = claims.get_permissions();

    let authenticated_user = AuthenticatedUser::new(
        user_id,
        claims.username.clone(),
        Some(claims.email.clone()),
        permissions,
    );

    request_context = request_context
        .with_user(authenticated_user)
        .with_user_id(UserId::from(claims.sub.clone()))
        .with_user_type(claims.user_type);

    Ok(AuthResult::Authenticated(AuthenticatedRequestContext::new(
        request_context,
        token,
    )))
}
