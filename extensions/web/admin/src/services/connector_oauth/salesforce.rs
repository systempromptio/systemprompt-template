//! Salesforce ECA pre-authorization using the user's administratively mapped
//! identity.

use super::{Grant, Provider, config};
use crate::error::{AdminError, AdminResult};
use crate::handlers::salesforce_auth::SalesforceConfig;
use crate::repositories::users::salesforce_identity;
use crate::services::salesforce_jwt_bearer;
use chrono::Utc;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

pub(super) async fn mint(pool: &PgPool, user: &UserId, generation: i64) -> AdminResult<Grant> {
    let username = salesforce_identity::find_username(pool, user)
        .await?
        .ok_or_else(|| {
            AdminError::NotFound(
                "Salesforce identity is not provisioned; connect in the browser".into(),
            )
        })?;
    let cfg = SalesforceConfig {
        enabled: true,
        my_domain: config::salesforce_domain()?,
        consumer_key: config::secret("salesforce_mcp_client_id")?,
    };
    let token = salesforce_jwt_bearer::get_token(&cfg, &username)
        .await
        .map_err(|_redacted_error| {
            AdminError::Upstream(
                "Salesforce pre-authorization failed; check External Client App settings".into(),
            )
        })?;
    Ok(Grant {
        user: user.to_string(),
        provider: Provider::Salesforce,
        client: cfg.consumer_key,
        client_secret: String::new(),
        verifier: String::new(),
        access_token: token.access_token,
        refresh_token: None,
        expires_at: Utc::now().timestamp() + 60,
        token_endpoint: String::new(),
        generation,
        session: None,
        auth_method: "jwt_bearer".into(),
        account_id: String::new(),
        account_name: username,
        resource_id: String::new(),
        resource_name: cfg.my_domain,
        authorization_scheme: "Bearer".into(),
    })
}
