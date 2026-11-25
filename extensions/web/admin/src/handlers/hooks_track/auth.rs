//! Hook token validation for `/hooks/track`.
//!
//! Hook JWTs carry `aud=hook` and scope `hook:track`, distinct from the API and
//! plugin audiences. Core's request authenticator verifies the token, its
//! user and its session; the plugin the event is attributed to is the
//! `plugin_id` the hook URL carries.

use crate::env::AdminEnv;

use crate::error::{AdminError, AdminResult};
use axum::http::HeaderMap;
use systemprompt::identifiers::{PluginId, UserId};
use systemprompt::models::auth::{JwtAudience, Permission};

pub(super) async fn extract_and_validate_jwt(
    env: &AdminEnv,
    headers: &HeaderMap,
    plugin_id: Option<PluginId>,
) -> AdminResult<(UserId, PluginId, String)> {
    let token = crate::services::auth::bearer_from(headers)
        .ok_or_else(|| AdminError::Unauthorized("Missing Authorization header".to_owned()))?;
    let (context, _) =
        crate::services::auth::verify_bearer(env, token, &[JwtAudience::Hook]).await?;
    if !context.scope.contains(&Permission::HookTrack) {
        return Err(AdminError::Unauthorized(
            "Hook token lacks hook:track".to_owned(),
        ));
    }
    let plugin_id =
        plugin_id.ok_or_else(|| AdminError::BadRequest("plugin_id is required".to_owned()))?;
    Ok((context.user_id, plugin_id, token.to_owned()))
}
