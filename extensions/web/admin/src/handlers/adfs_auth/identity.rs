//! Identity half of the ADFS callback: turn a verified assertion into gated
//! claims (allow-listed domain, mapped AD group) and resolve them to a local
//! user with the roles the groups map to.

use saml::{Identity, NameIdFormat};

use super::AdfsDeps;
use super::config::AdfsConfig;
use crate::repositories::users::{federated, revocation};
use crate::repositories::{groups, projects};

/// The subset of a verified assertion this login consumes: the subject and
/// the attribute statement, with the `IdP`'s claim URIs left as-is.
#[derive(Debug, Clone, Default)]
pub struct AssertionClaims {
    pub name_id: Option<String>,
    pub name_id_is_email: bool,
    pub attributes: Vec<(String, Vec<String>)>,
}

impl AssertionClaims {
    #[must_use]
    pub fn from_identity(identity: &Identity) -> Self {
        Self {
            name_id: Some(identity.name_id.value.clone()),
            name_id_is_email: identity.name_id.format == NameIdFormat::EmailAddress,
            attributes: identity
                .attributes
                .iter()
                .map(|a| (a.name.clone(), a.values.clone()))
                .collect(),
        }
    }

    // Why: Every value under the named attribute, in assertion order. AD FS
    // emits one `<AttributeValue>` per group.
    #[must_use]
    pub fn values(&self, attribute: &str) -> Vec<String> {
        self.attributes
            .iter()
            .filter(|(name, _)| name == attribute)
            .flat_map(|(_, values)| values.iter().cloned())
            .collect()
    }

    fn first(&self, attribute: &str) -> Option<String> {
        self.values(attribute)
            .into_iter()
            .map(|v| v.trim().to_owned())
            .find(|v| !v.is_empty())
    }

    // Why: The sign-in email: the configured email attribute, else an
    // email-format NameID. Lower-cased, since AD is case-insensitive and
    // `users.email` is stored lower.
    #[must_use]
    pub fn login_email(&self, cfg: &AdfsConfig) -> Option<String> {
        self.first(&cfg.email_attribute)
            .or_else(|| {
                self.name_id
                    .as_deref()
                    .filter(|_| self.name_id_is_email)
                    .map(ToOwned::to_owned)
            })
            .map(|e| e.trim().to_lowercase())
            .filter(|e| e.contains('@'))
    }

    #[must_use]
    pub fn display_name(&self, cfg: &AdfsConfig) -> Option<String> {
        self.first(&cfg.name_attribute)
    }

    // Why: The stable external subject: the NameID when the IdP sends one,
    // else the email — AD FS relying parties are routinely configured with
    // claims but no NameID, and the login email is the next-best stable key.
    #[must_use]
    pub fn external_sub(&self, email: &str) -> String {
        self.name_id
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map_or_else(|| email.to_owned(), ToOwned::to_owned)
    }
}

// Why: What survives the gate: the identity to resolve and the entitlements
// the assertion carried.
pub(super) struct GatedIdentity {
    pub external_sub: String,
    pub email: String,
    pub display_name: String,
    pub groups: Vec<String>,
    pub roles: Vec<String>,
}

// Why: Pure so the gate can be pinned by unit tests without a farm. AD is the
// corporate directory: an address it asserts is verified by construction, so
// there is no `email_verified` check here — the domain allow-list and the
// group map are the gate.
pub(super) fn gate_claims(
    cfg: &AdfsConfig,
    claims: &AssertionClaims,
) -> Result<GatedIdentity, &'static str> {
    let email = claims.login_email(cfg).ok_or("no_email")?;
    if !cfg.email_allowed(&email) {
        tracing::warn!(email, "ADFS login rejected: domain not allow-listed");
        return Err("forbidden");
    }
    let all_groups = claims.values(&cfg.groups_attribute);
    // Why: the refusal is "the directory vouched for nobody" — an assertion
    // with no group claim at all. A group the role map does not name is not a
    // refusal: which group a person is in is decided by the DB mapping in
    // services/web/config/groups.yaml, and the role map only says what that
    // membership additionally mints. Denying on an unmapped group would refuse
    // every member of a group IT added before we did.
    if all_groups.is_empty() && cfg.deny_without_group {
        tracing::warn!(email, "ADFS login rejected: assertion carried no AD group");
        return Err("no_group");
    }
    let groups = cfg.mapped_groups(&all_groups);
    let mut roles = cfg.roles_for_groups(&all_groups);
    if roles.is_empty() {
        roles.push("user".to_owned());
    }
    let display_name = claims.display_name(cfg).unwrap_or_else(|| email.clone());
    Ok(GatedIdentity {
        external_sub: claims.external_sub(&email),
        email,
        display_name,
        groups,
        roles,
    })
}

// Why: Gate the verified claims and resolve them to a local user. Each step
// logs its own failure and collapses to a login *reason*.
pub(super) async fn resolve_identity(
    deps: &AdfsDeps,
    idp_entity_id: &str,
    claims: &AssertionClaims,
) -> Result<federated::ResolvedFederatedUser, &'static str> {
    let cfg = &deps.config;
    let gated = match gate_claims(cfg, claims) {
        Ok(gated) => gated,
        Err(reason) => {
            revoke_on_denial(deps, cfg, claims, reason).await;
            return Err(reason);
        },
    };

    let federated_claims = federated::FederatedClaims {
        issuer: idp_entity_id,
        external_sub: &gated.external_sub,
        email: &gated.email,
        display_name: &gated.display_name,
        roles: &gated.roles,
    };
    let resolved =
        federated::resolve_federated_user(&deps.write_pool, &federated_claims, cfg.auto_provision)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to resolve federated ADFS user");
                "error"
            })?
            .ok_or_else(|| {
                tracing::warn!(
                    email = %gated.email,
                    "ADFS login rejected: auto-provisioning disabled and no existing account"
                );
                "not_provisioned"
            })?;

    // Why: the assertion's groups are replayed onto both memberships as the
    // directory-sourced half, replacing what the last sign-in wrote. A
    // failure here must not break login — the roles already carry the coarse
    // entitlement.
    if let Err(e) = groups::members::replace_directory_group_memberships(
        &deps.write_pool,
        &resolved.user_id,
        &gated.groups,
    )
    .await
    {
        tracing::warn!(error = %e, user_id = %resolved.user_id, "Failed to persist ADFS group memberships");
    }
    if let Err(e) = projects::members::replace_directory_project_memberships(
        &deps.write_pool,
        &resolved.user_id,
        &gated.groups,
    )
    .await
    {
        tracing::warn!(error = %e, user_id = %resolved.user_id, "Failed to persist ADFS project memberships");
    }
    crate::authz::group::invalidate(&resolved.user_id).await;
    crate::authz::project::invalidate(&resolved.user_id).await;

    // Why: `permissions_for_roles` baked `Permission::Admin` into every token
    // minted while the user was still in the admins group, and those tokens
    // outlive this assertion. A demotion has to reach them, or the directory
    // has not actually demoted anyone until they expire.
    if resolved.lost_admin {
        match revocation::revoke_user_access(&deps.write_pool, &resolved.user_id).await {
            Ok(counts) => tracing::warn!(
                user_id = %resolved.user_id,
                sessions = counts.sessions,
                api_keys = counts.api_keys,
                "Revoked credentials after ADFS removed the admin role"
            ),
            Err(e) => tracing::error!(
                error = %e,
                user_id = %resolved.user_id,
                "Failed to revoke credentials after an ADFS admin demotion"
            ),
        }
    }

    Ok(resolved)
}

// Why: a refused sign-in is only half of a deprovision. The person still holds
// a cookie session and a linked bridge from the last time the directory did
// vouch for them, and neither expires on its own. Failures are logged and
// swallowed: the caller owes the user a `?sso=` redirect either way, and a
// revocation error must never become a 500 on the login path.
//
// `no_email` carries no handle to revoke on; `not_provisioned` is reached only
// when no local account exists at all.
async fn revoke_on_denial(
    deps: &AdfsDeps,
    cfg: &AdfsConfig,
    claims: &AssertionClaims,
    reason: &str,
) {
    if !matches!(reason, "no_group" | "forbidden") {
        return;
    }
    let Some(email) = claims.login_email(cfg) else {
        return;
    };
    match revocation::revoke_access_by_email(&deps.write_pool, &email).await {
        Ok(Some((user_id, counts))) if !counts.is_empty() => tracing::warn!(
            %user_id,
            reason,
            sessions = counts.sessions,
            api_keys = counts.api_keys,
            device_certs = counts.device_certs,
            exchange_codes = counts.exchange_codes,
            "Revoked credentials after ADFS refused the sign-in"
        ),
        Ok(_) => {},
        Err(e) => {
            tracing::error!(error = %e, reason, "Failed to revoke credentials after a refused ADFS sign-in");
        },
    }
}
