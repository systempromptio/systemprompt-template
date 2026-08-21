//! Organization membership management: list organizations, move a user
//! between them, change their org role.
//!
//! Listing and cross-organization moves are platform-admin only. An
//! organization's own admin (`org_role` owner/admin, `admin` role, not
//! platform) may additionally set member↔admin for existing members of their
//! own org — never `owner`, never a current owner, never across orgs, and
//! never adding a non-member (membership additions stay with invites and
//! platform admins). Membership is otherwise set only as a side effect of
//! user creation or SSO just-in-time provisioning.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Extension, Path, State};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::activity::{self, ActivityEntity, NewActivity};
use crate::error::{AdminError, AdminResult};
use crate::repositories::organizations::{crud, seats};
use crate::types::UserContext;

fn require_platform_admin(user_ctx: &UserContext) -> AdminResult<()> {
    if user_ctx.is_platform_admin {
        Ok(())
    } else {
        Err(AdminError::Forbidden(
            "Platform admin access required".to_owned(),
        ))
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct OrganizationView {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub plan_name: Option<String>,
    pub status: String,
    pub is_platform: bool,
    pub seats_used: i64,
    pub seat_limit: Option<i32>,
}

pub(crate) async fn list_organizations_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
) -> AdminResult<Response> {
    require_platform_admin(&user_ctx)?;
    let orgs = crud::list_organizations(&pool).await?;
    let body: Vec<OrganizationView> = orgs
        .into_iter()
        .map(|o| OrganizationView {
            id: o.id,
            slug: o.slug,
            name: o.name,
            plan_name: o.plan_name,
            status: o.status,
            is_platform: o.is_platform,
            seats_used: o.seats_used,
            seat_limit: o.seat_limit,
        })
        .collect();
    Ok(Json(body).into_response())
}

#[derive(Debug, Deserialize)]
pub(crate) struct SetOrganizationRequest {
    /// Organization slug — the URL-facing immutable key.
    pub org: String,
    /// owner | admin | member; defaults to member.
    pub org_role: Option<String>,
}

pub(crate) async fn set_user_organization_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(user_id_raw): Path<String>,
    Json(body): Json<SetOrganizationRequest>,
) -> AdminResult<Response> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden("Admin access required".to_owned()));
    }
    let user_id = UserId::new(user_id_raw);

    let role = body.org_role.as_deref().unwrap_or("member");
    if !matches!(role, "owner" | "admin" | "member") {
        return Err(AdminError::BadRequest(
            "org_role must be owner, admin, or member".to_owned(),
        ));
    }

    let org = crud::find_organization_by_slug(&pool, body.org.trim())
        .await?
        .ok_or_else(|| {
            AdminError::NotFound(format!("No organization with slug '{}'.", body.org.trim()))
        })?;

    let target_role = crud::find_member_role(&pool, &user_id, &org.id).await?;

    if user_ctx.is_platform_admin {
        // Why: moving into a new org mints a seat there, so the target's
        // limit is checked; a role change inside the same org must not be
        // blocked by a full plan (see set_membership's contract).
        if target_role.is_none() {
            seats::assert_seat_available(&pool, &org.id).await?;
        }
    } else {
        require_org_admin_grant(&pool, &user_ctx, &org, role, target_role.as_deref()).await?;
    }
    crud::set_membership(&pool, &user_id, &org.id, role).await?;

    let p = Arc::clone(&pool);
    let uid = user_ctx.user_id.clone();
    let target = user_id.clone();
    let label = format!("{} → {} ({role})", target.as_str(), org.slug);
    tokio::spawn(async move {
        activity::record(
            &p,
            NewActivity::entity_updated(&uid, ActivityEntity::User, target.as_str(), &label),
        )
        .await;
    });

    Ok(Json(serde_json::json!({
        "user_id": user_id.as_str(),
        "org_id": org.id,
        "org_slug": org.slug,
        "org_role": role,
    }))
    .into_response())
}

// Why: the narrow non-platform grant — an org's own admin may toggle
// member↔admin for people already inside their org. Everything else is
// refused: `owner` (both setting it and demoting one) stays a platform act,
// as do cross-org moves and membership additions, so this path never needs a
// seat check (role changes never mint seats).
async fn require_org_admin_grant(
    pool: &PgPool,
    user_ctx: &UserContext,
    org: &crud::OrganizationSummary,
    requested_role: &str,
    target_role: Option<&str>,
) -> AdminResult<()> {
    let own_slug = crud::find_organization_for_user(pool, &user_ctx.user_id)
        .await?
        .ok_or_else(|| {
            AdminError::Forbidden(
                "You are not a member of an organization, so there are no memberships to manage."
                    .to_owned(),
            )
        })?;
    if own_slug != org.slug {
        return Err(AdminError::Forbidden(
            "Organization admins may only manage their own organization's members".to_owned(),
        ));
    }
    let caller_role = crud::find_member_role(pool, &user_ctx.user_id, &org.id).await?;
    if !matches!(caller_role.as_deref(), Some("owner" | "admin")) {
        return Err(AdminError::Forbidden(
            "Managing member roles requires the owner or admin org role".to_owned(),
        ));
    }
    if requested_role == "owner" {
        return Err(AdminError::BadRequest(
            "Only platform administrators may assign the owner role".to_owned(),
        ));
    }
    match target_role {
        None => Err(AdminError::Forbidden(
            "Only platform administrators (or an invite) may add members to an organization"
                .to_owned(),
        )),
        Some("owner") => Err(AdminError::Forbidden(
            "Only platform administrators may change an owner's role".to_owned(),
        )),
        Some(_) => Ok(()),
    }
}
