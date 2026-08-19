//! The three principals every route is driven under, and the credentials that
//! distinguish them.
//!
//! Role membership is deliberately *not* carried in the JWT:
//! `user_context_middleware` reads `users.roles` from the database, so the
//! admin / non-admin split is seeded as table rows and the middleware resolves
//! it the same way it does in production. The token only has to validate and
//! carry a subject.

use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};
use systemprompt_security::{AdminTokenParams, JwtService};

use crate::globals;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Principal {
    // No cookie and no `Authorization` header.
    Anonymous,
    // A valid session for a user holding only the `user` role.
    NonAdmin,
    // A valid session for a user holding `admin`.
    Admin,
}

impl Principal {
    pub const ALL: [Self; 3] = [Self::Anonymous, Self::NonAdmin, Self::Admin];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Anonymous => "anonymous",
            Self::NonAdmin => "non-admin",
            Self::Admin => "admin",
        }
    }
}

// Bearer tokens for the two authenticated principals, minted once and reused
// across the whole table.
pub struct Credentials {
    pub non_admin: String,
    pub admin: String,
}

impl Credentials {
    pub fn token_for(&self, principal: Principal) -> Option<&str> {
        match principal {
            Principal::Anonymous => None,
            Principal::NonAdmin => Some(&self.non_admin),
            Principal::Admin => Some(&self.admin),
        }
    }
}

// Seed one admin and one plain user, and mint a token for each.
//
// The admin is joined to the platform tenant. The `admin` role alone is what a
// customer's own administrator holds, so without that membership the admin
// principal cannot reach the cross-customer console — and every route behind
// `require_platform_admin_middleware` would be pinned at 403 by a suite that
// never exercised it.
pub async fn provision(pool: &PgPool) -> Credentials {
    let non_admin = provision_one(pool, "contract-user", &["user"], false).await;
    let admin = provision_one(pool, "contract-admin", &["admin", "user"], true).await;
    Credentials { non_admin, admin }
}

async fn provision_one(pool: &PgPool, name: &str, roles: &[&str], platform: bool) -> String {
    let user_id = UserId::new(format!("{name}-{}", uuid::Uuid::new_v4().simple()));
    let email = format!("{name}@contract.test");

    sqlx::query(
        "INSERT INTO users (id, name, email, roles, email_verified)
         VALUES ($1, $2, $3, $4, true)",
    )
    .bind(user_id.as_str())
    .bind(user_id.as_str().to_owned())
    .bind(&email)
    .bind(roles.iter().map(|r| (*r).to_owned()).collect::<Vec<_>>())
    .execute(pool)
    .await
    .expect("seed contract principal");

    if platform {
        join_platform_tenant(pool, &user_id).await;
    }

    let session_id = SessionId::new(uuid::Uuid::new_v4().to_string());
    let token = JwtService::generate_admin_token(&AdminTokenParams {
        user_id: &user_id,
        session_id: &session_id,
        email: &email,
        issuer: &globals::jwt_issuer(),
        duration: chrono::Duration::hours(1),
        client_id: None,
    })
    .expect("mint a session token");

    token.as_str().to_owned()
}

// Join a user to the platform organization, creating it when the fixture
// database has none.
//
// The unique index on `is_platform` means at most one can exist, so this
// adopts whichever tenant is already there rather than insisting on its own.
async fn join_platform_tenant(pool: &PgPool, user_id: &UserId) {
    let org_id: String = sqlx::query_scalar(
        "INSERT INTO organizations (id, slug, name, is_platform)
         VALUES ('contract-platform', 'contract-platform', 'Contract Platform', TRUE)
         ON CONFLICT DO NOTHING
         RETURNING id",
    )
    .fetch_optional(pool)
    .await
    .expect("seed platform tenant")
    .unwrap_or_else(String::new);

    let org_id = if org_id.is_empty() {
        sqlx::query_scalar("SELECT id FROM organizations WHERE is_platform LIMIT 1")
            .fetch_one(pool)
            .await
            .expect("an existing platform tenant")
    } else {
        org_id
    };

    sqlx::query(
        "INSERT INTO organization_members (user_id, org_id, org_role)
         VALUES ($1, $2, 'owner')
         ON CONFLICT (user_id) DO UPDATE SET org_id = EXCLUDED.org_id",
    )
    .bind(user_id.as_str())
    .bind(&org_id)
    .execute(pool)
    .await
    .expect("join the platform tenant");
}
