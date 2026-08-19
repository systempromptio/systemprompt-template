//! `POST /admin/api/profile/salesforce/unlink` — detach the signed-in user's
//! Salesforce identity.

use axum::{Extension, Json};
use serde::Serialize;

use super::SalesforceDeps;
use crate::error::{AdminError, AdminResult};
use crate::repositories::users::{federated, passkey, salesforce_identity};
use crate::types::UserContext;

#[derive(Debug, Serialize)]
pub(crate) struct UnlinkResponse {
    unlinked: bool,
}

// Why: a user whose only credential is the Salesforce identity would lock
// themselves out by unlinking it, so a passkey must exist first.
pub(crate) async fn salesforce_unlink(
    Extension(user_ctx): Extension<UserContext>,
    Extension(deps): Extension<SalesforceDeps>,
) -> AdminResult<Json<UnlinkResponse>> {
    let passkeys = passkey::count_webauthn_credentials(&deps.write_pool, &user_ctx.user_id).await?;
    if passkeys == 0 {
        return Err(AdminError::Conflict(
            "Add a passkey before disconnecting Salesforce — it is currently your only way to sign in."
                .to_owned(),
        ));
    }

    let removed = federated::delete_federated_identities_for_issuer(
        &deps.write_pool,
        &user_ctx.user_id,
        deps.config.issuer(),
    )
    .await?;
    salesforce_identity::delete(&deps.write_pool, &user_ctx.user_id).await?;
    // Why: the `salesforce` subject dimension caches link state; without this
    // the disconnected user keeps the Salesforce entities until the TTL runs.
    crate::authz::salesforce::invalidate(&user_ctx.user_id).await;

    tracing::info!(user_id = %user_ctx.user_id, removed, "Salesforce identity unlinked");
    Ok(Json(UnlinkResponse { unlinked: true }))
}
