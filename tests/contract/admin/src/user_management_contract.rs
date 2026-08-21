//! HTTP-level authorization contract for user provisioning: invites, the
//! org-membership endpoints, and the narrow grant that lets an organization's
//! own admin manage their members' roles.
//!
//! These are the rules that decide whether one customer's administrator can
//! reach another customer's people, so they are asserted over the real router
//! with real tokens rather than at the repository layer, where the middleware
//! and handler guards would not run at all.

use axum::http::StatusCode;
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};
use systemprompt_security::{AdminTokenParams, JwtService};

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

// An organization admin: the `admin` role, owner of their own org, and
// deliberately NOT a member of the platform tenant.
struct OrgAdmin {
    token: String,
    org_slug: String,
}

async fn seed_org(pool: &PgPool, slug: &str) -> String {
    let id = format!("{slug}-{}", uuid::Uuid::new_v4().simple());
    sqlx::query(
        "INSERT INTO organizations (id, slug, name, status)
         VALUES ($1, $2, $2, 'active')",
    )
    .bind(&id)
    .bind(slug)
    .execute(pool)
    .await
    .expect("seed organization");
    id
}

async fn seed_member(pool: &PgPool, org_id: &str, roles: &[&str], org_role: &str) -> UserId {
    let user_id = UserId::new(format!("member-{}", uuid::Uuid::new_v4().simple()));
    let email = format!("{}@contract.test", user_id.as_str());
    sqlx::query(
        "INSERT INTO users (id, name, email, roles, email_verified)
         VALUES ($1, $1, $2, $3, true)",
    )
    .bind(user_id.as_str())
    .bind(&email)
    .bind(roles.iter().map(|r| (*r).to_owned()).collect::<Vec<_>>())
    .execute(pool)
    .await
    .expect("seed member");
    sqlx::query(
        "INSERT INTO organization_members (user_id, org_id, org_role) VALUES ($1, $2, $3)
         ON CONFLICT (user_id) DO UPDATE SET org_id = EXCLUDED.org_id, org_role = EXCLUDED.org_role",
    )
    .bind(user_id.as_str())
    .bind(org_id)
    .bind(org_role)
    .execute(pool)
    .await
    .expect("seed membership");
    user_id
}

fn mint(user_id: &UserId) -> String {
    let session_id = SessionId::new(uuid::Uuid::new_v4().to_string());
    JwtService::generate_admin_token(&AdminTokenParams {
        user_id,
        session_id: &session_id,
        email: &format!("{}@contract.test", user_id.as_str()),
        issuer: &globals::jwt_issuer(),
        duration: chrono::Duration::hours(1),
        client_id: None,
    })
    .expect("mint a session token")
    .as_str()
    .to_owned()
}

async fn seed_org_admin(pool: &PgPool, slug: &str) -> (OrgAdmin, String) {
    let org_id = seed_org(pool, slug).await;
    let admin = seed_member(pool, &org_id, &["admin", "user"], "owner").await;
    (
        OrgAdmin {
            token: mint(&admin),
            org_slug: slug.to_owned(),
        },
        org_id,
    )
}

#[tokio::test]
async fn an_org_admin_may_not_invite_with_elevated_roles() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let (org_admin, _) = seed_org_admin(&db.pool, "acme").await;

    let (status, body) = app
        .call_with_bearer(
            Call::json(
                "post",
                "/api/public/admin/invites",
                Principal::Anonymous,
                r#"{"email":"newcomer@acme.test","roles":["admin"]}"#,
            ),
            &org_admin.token,
        )
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN, "body: {body}");
    assert!(
        body.to_lowercase().contains("platform"),
        "the refusal must name the reason: {body}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn an_org_admins_invite_lands_in_their_own_org_whatever_org_they_ask_for() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let (org_admin, _) = seed_org_admin(&db.pool, "acme").await;
    seed_org(&db.pool, "rival").await;

    let (status, body) = app
        .call_with_bearer(
            Call::json(
                "post",
                "/api/public/admin/invites",
                Principal::Anonymous,
                r#"{"email":"newcomer@acme.test","org":"rival"}"#,
            ),
            &org_admin.token,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");

    // The `?org` was ignored, not honoured: the invite must be visible in the
    // caller's own organization's list.
    let (status, listed) = app
        .call_with_bearer(
            Call::get("/api/public/admin/invites", Principal::Anonymous),
            &org_admin.token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        listed.contains("newcomer@acme.test"),
        "the invite belongs to {}: {listed}",
        org_admin.org_slug
    );
    db.cleanup().await;
}

#[tokio::test]
async fn a_platform_admin_must_name_the_organization_they_invite_into() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (status, _) = app
        .call(Call::json(
            "post",
            "/api/public/admin/invites",
            Principal::Admin,
            r#"{"email":"someone@acme.test"}"#,
        ))
        .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "a platform admin has no implicit organization to invite into"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn invite_listing_is_scoped_to_the_callers_organization() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let (acme_admin, _) = seed_org_admin(&db.pool, "acme").await;
    let (rival_admin, _) = seed_org_admin(&db.pool, "rival").await;

    for (admin, email) in [
        (&acme_admin, "one@acme.test"),
        (&rival_admin, "two@rival.test"),
    ] {
        let (status, body) = app
            .call_with_bearer(
                Call::json(
                    "post",
                    "/api/public/admin/invites",
                    Principal::Anonymous,
                    &format!(r#"{{"email":"{email}"}}"#),
                ),
                &admin.token,
            )
            .await;
        assert_eq!(status, StatusCode::CREATED, "body: {body}");
    }

    let (_, listed) = app
        .call_with_bearer(
            Call::get("/api/public/admin/invites", Principal::Anonymous),
            &acme_admin.token,
        )
        .await;
    assert!(listed.contains("one@acme.test"));
    assert!(
        !listed.contains("two@rival.test"),
        "another tenant's invites must not be listed: {listed}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn listing_organizations_is_platform_admin_only() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let (org_admin, _) = seed_org_admin(&db.pool, "acme").await;

    let (status, _) = app
        .call_with_bearer(
            Call::get(
                "/api/public/admin/management/organizations",
                Principal::Anonymous,
            ),
            &org_admin.token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    let (status, _) = app
        .call(Call::get(
            "/api/public/admin/management/organizations",
            Principal::Admin,
        ))
        .await;
    assert_eq!(status, StatusCode::OK);
    db.cleanup().await;
}

#[tokio::test]
async fn an_org_admin_may_promote_their_own_member_but_not_beyond() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let (org_admin, org_id) = seed_org_admin(&db.pool, "acme").await;
    let member = seed_member(&db.pool, &org_id, &["user"], "member").await;
    let owner = seed_member(&db.pool, &org_id, &["user"], "owner").await;

    let path = format!(
        "/api/public/admin/management/users/{}/organization",
        member.as_str()
    );
    let body = format!(r#"{{"org":"{}","org_role":"admin"}}"#, org_admin.org_slug);
    let (status, response) = app
        .call_with_bearer(
            Call::json("put", &path, Principal::Anonymous, &body),
            &org_admin.token,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "member -> admin is allowed: {response}");

    let body = format!(r#"{{"org":"{}","org_role":"owner"}}"#, org_admin.org_slug);
    let (status, _) = app
        .call_with_bearer(
            Call::json("put", &path, Principal::Anonymous, &body),
            &org_admin.token,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::BAD_REQUEST,
        "assigning owner stays a platform act"
    );

    let owner_path = format!(
        "/api/public/admin/management/users/{}/organization",
        owner.as_str()
    );
    let body = format!(r#"{{"org":"{}","org_role":"member"}}"#, org_admin.org_slug);
    let (status, _) = app
        .call_with_bearer(
            Call::json("put", &owner_path, Principal::Anonymous, &body),
            &org_admin.token,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "an owner's role may not be changed by an org admin"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn an_org_admin_may_not_touch_another_organizations_member() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let (acme_admin, _) = seed_org_admin(&db.pool, "acme").await;
    let rival_id = seed_org(&db.pool, "rival").await;
    let outsider = seed_member(&db.pool, &rival_id, &["user"], "member").await;

    let path = format!(
        "/api/public/admin/management/users/{}/organization",
        outsider.as_str()
    );

    // Naming their own org: the target is not a member of it, so this would be
    // an addition, which org admins may not perform.
    let body = format!(r#"{{"org":"{}","org_role":"member"}}"#, acme_admin.org_slug);
    let (status, _) = app
        .call_with_bearer(
            Call::json("put", &path, Principal::Anonymous, &body),
            &acme_admin.token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "no adding outsiders");

    // Naming the other org: a cross-tenant move.
    let (status, _) = app
        .call_with_bearer(
            Call::json(
                "put",
                &path,
                Principal::Anonymous,
                r#"{"org":"rival","org_role":"admin"}"#,
            ),
            &acme_admin.token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN, "no cross-tenant moves");
    db.cleanup().await;
}

#[tokio::test]
async fn regenerating_an_invite_issues_a_new_path_and_refuses_across_tenants() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);
    let (acme_admin, _) = seed_org_admin(&db.pool, "acme").await;
    let (rival_admin, _) = seed_org_admin(&db.pool, "rival").await;

    let (status, created) = app
        .call_with_bearer(
            Call::json(
                "post",
                "/api/public/admin/invites",
                Principal::Anonymous,
                r#"{"email":"regen@acme.test"}"#,
            ),
            &acme_admin.token,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "body: {created}");
    let minted: serde_json::Value = serde_json::from_str(&created).expect("invite json");
    let invite_id = minted["id"].as_str().expect("invite id").to_owned();
    let original_path = minted["invite_path"].as_str().expect("invite path").to_owned();

    let path = format!("/api/public/admin/invites/{invite_id}/regenerate");

    let (status, _) = app
        .call_with_bearer(
            Call::json("post", &path, Principal::Anonymous, "{}"),
            &rival_admin.token,
        )
        .await;
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "another tenant's invite is not visible to regenerate"
    );

    let (status, regenerated) = app
        .call_with_bearer(
            Call::json("post", &path, Principal::Anonymous, "{}"),
            &acme_admin.token,
        )
        .await;
    assert_eq!(status, StatusCode::CREATED, "body: {regenerated}");
    let fresh: serde_json::Value = serde_json::from_str(&regenerated).expect("invite json");
    assert_ne!(
        fresh["invite_path"].as_str().expect("invite path"),
        original_path,
        "a regenerated invite must carry a new token"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn creating_a_user_returns_a_sign_in_link_when_a_domain_claims_them() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    // An org that claims the email domain, so the created user has somewhere
    // to belong and an invite can be minted for them.
    let org_id = seed_org(&db.pool, "claimed").await;
    sqlx::query("UPDATE organizations SET email_domains = ARRAY['claimed.test'] WHERE id = $1")
        .bind(&org_id)
        .execute(&*db.pool)
        .await
        .expect("claim the domain");

    let (status, body) = app
        .call(Call::json(
            "post",
            "/api/public/admin/users",
            Principal::Admin,
            r#"{"user_id":"claimed-newcomer","display_name":"Newcomer","email":"newcomer@claimed.test","roles":["user"],"department":"Engineering"}"#,
        ))
        .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let created: serde_json::Value = serde_json::from_str(&body).expect("created json");
    assert!(
        created["invite_path"]
            .as_str()
            .is_some_and(|p| p.starts_with("/admin/invite/")),
        "a created account needs a credential-bootstrap link: {body}"
    );

    let department: String = sqlx::query_scalar(
        "SELECT department FROM user_profile_ext WHERE user_id = 'claimed-newcomer'",
    )
    .fetch_one(&*db.pool)
    .await
    .expect("the department was persisted");
    assert_eq!(department, "Engineering");
    db.cleanup().await;
}

#[tokio::test]
async fn creating_a_user_on_an_unclaimed_domain_explains_the_missing_link() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (status, body) = app
        .call(Call::json(
            "post",
            "/api/public/admin/users",
            Principal::Admin,
            r#"{"user_id":"orphan-newcomer","display_name":"Orphan","email":"orphan@nobody-claims-this.test","roles":["user"]}"#,
        ))
        .await;
    assert_eq!(status, StatusCode::CREATED, "body: {body}");
    let created: serde_json::Value = serde_json::from_str(&body).expect("created json");
    assert!(created["invite_path"].is_null(), "no org, no invite: {body}");
    assert!(
        created["invite_note"].as_str().is_some_and(|n| !n.is_empty()),
        "the operator must be told why: {body}"
    );
    db.cleanup().await;
}
