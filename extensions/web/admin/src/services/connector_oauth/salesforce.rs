//! Salesforce ECA pre-authorization using the user's administratively mapped
//! identity for one org.

use super::{Grant, Provider};
use crate::error::{AdminError, AdminResult};
use crate::handlers::salesforce_auth::SalesforceConfig;
use crate::repositories::users::salesforce_identity;
use crate::services::salesforce_jwt_bearer;
use chrono::Utc;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

pub(super) async fn mint(
    pool: &PgPool,
    user: &UserId,
    provider: &Provider,
    generation: i64,
) -> AdminResult<Grant> {
    let org = provider.salesforce_org()?;
    let username = salesforce_identity::find_username(pool, user, provider.slug())
        .await?
        .ok_or_else(|| {
            AdminError::NotFound(
                "Salesforce identity is not provisioned; connect in the browser".into(),
            )
        })?;
    let cfg = SalesforceConfig {
        enabled: true,
        my_domain: org.domain()?,
        consumer_key: org.client_id()?,
    };
    let private_key = org.private_key()?;
    let token = salesforce_jwt_bearer::get_token(&cfg, &username, &private_key)
        .await
        .map_err(|_redacted_error| {
            AdminError::Upstream(
                "Salesforce pre-authorization failed; check External Client App settings".into(),
            )
        })?;
    Ok(Grant {
        configuration_binding: String::new(),
        authorization_issuer: String::new(),
        token_auth_method: String::new(),
        user: user.to_string(),
        provider: provider.clone(),
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
