//! The seven principals every route is driven under, and the credentials that
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
    // A valid session for a user holding `developer`. Distinct from NonAdmin
    // because `developer` names what someone builds and must reach no admin
    // route at all — a band that would otherwise be untested.
    Developer,
    // A valid session for a user holding `admin`.
    Admin,
    // A valid session for a user holding `platform_admin`, the only role that
    // reaches the directory-shaped controls (AD mappings, granting
    // platform_admin itself).
    PlatformAdmin,
    // A valid session for a user holding the semi-admin `project_manager`
    // role: the read-only admin dashboard, no write route, no admin MCP.
    ProjectManager,
    // A valid session for a user holding `knowledge_worker`, the Cowork
    // entitlement role. Like `developer` it names what someone reaches in a
    // client, not a console band: it must reach no admin route, without granting console access.
    KnowledgeWorker,
}

impl Principal {
    pub const ALL: [Self; 3] = [Self::Anonymous, Self::NonAdmin, Self::Admin];

    pub const ALL_DASHBOARD: [Self; 7] = [
        Self::Anonymous,
        Self::NonAdmin,
        Self::Developer,
        Self::Admin,
        Self::PlatformAdmin,
        Self::ProjectManager,
        Self::KnowledgeWorker,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Anonymous => "anonymous",
            Self::NonAdmin => "non-admin",
            Self::Developer => "developer",
            Self::Admin => "admin",
            Self::PlatformAdmin => "platform-admin",
            Self::ProjectManager => "project-manager",
            Self::KnowledgeWorker => "knowledge-worker",
        }
    }
}

// Bearer tokens for the two authenticated principals, minted once and reused
// across the whole table.
pub struct Credentials {
    pub non_admin: String,
    pub developer: String,
    pub admin: String,
    pub platform_admin: String,
    pub project_manager: String,
    pub knowledge_worker: String,
    // The non-admin's own user id, so a case can seed a row that principal
    // genuinely owns and drive an owner-facing route as its owner.
    pub non_admin_user_id: UserId,
}

impl Credentials {
    pub fn token_for(&self, principal: Principal) -> Option<&str> {
        match principal {
            Principal::Anonymous => None,
            Principal::NonAdmin => Some(&self.non_admin),
            Principal::Developer => Some(&self.developer),
            Principal::Admin => Some(&self.admin),
            Principal::PlatformAdmin => Some(&self.platform_admin),
            Principal::ProjectManager => Some(&self.project_manager),
            Principal::KnowledgeWorker => Some(&self.knowledge_worker),
        }
    }
}

// Why: preserve the original route corpus's two-account fixtures. The new
// dashboard contracts explicitly opt into the broader privilege matrix.
pub async fn provision(pool: &PgPool) -> Credentials {
    let (non_admin, non_admin_user_id) =
        provision_one(pool, "contract-user", &["user"], false).await;
    let (admin, _) = provision_one(pool, "contract-admin", &["admin", "user"], false).await;
    Credentials {
        non_admin,
        admin,
        non_admin_user_id,
        developer: String::new(),
        platform_admin: String::new(),
        project_manager: String::new(),
        knowledge_worker: String::new(),
    }
}

// The group the non-console principals are placed in.
//
// Why a real group rather than none: a caller in no group is derived into
// `unassigned`, which is a legitimate but atypical state. Every scoped listing
// narrows to the caller's groups, so seeding a real membership is what makes
// the scoped routes return something and their statuses meaningful.
const CONTRACT_GROUP: &str = "contract-group";

// Seed one account per role band and mint a token for each.
//
// The admin carries no group, as an operator account that never came through
// AD FS would; everyone else is placed in `contract-group` as an
// SSO-provisioned account is. Roles alone decide what a principal reaches.
pub async fn provision_dashboard(pool: &PgPool) -> Credentials {
    sqlx::query(
        "INSERT INTO groups (id, name, source) VALUES ($1, 'Contract group', 'dashboard')
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(CONTRACT_GROUP)
    .execute(pool)
    .await
    .expect("seed the contract group");

    let (non_admin, non_admin_user_id) =
        provision_one(pool, "contract-user", &["user"], true).await;
    let (developer, _) = provision_one(pool, "contract-dev", &["developer", "user"], true).await;
    let (admin, _) = provision_one(pool, "contract-admin", &["admin", "user"], false).await;
    let (platform_admin, _) = provision_one(
        pool,
        "contract-platform",
        &["platform_admin", "admin", "user"],
        false,
    )
    .await;
    let (project_manager, _) =
        provision_one(pool, "contract-pm", &["project_manager", "user"], true).await;
    let (knowledge_worker, knowledge_worker_user_id) =
        provision_one(pool, "contract-kw", &["knowledge_worker", "user"], true).await;
    // `knowledge_worker` is never asserted by the directory: it is granted by
    // hand, and the manual-roles table is what the recomputation job reads
    // back, so the seed writes the same row an admin's grant would.
    sqlx::query(
        "INSERT INTO user_manual_roles (user_id, role) VALUES ($1, 'knowledge_worker')
         ON CONFLICT DO NOTHING",
    )
    .bind(knowledge_worker_user_id.as_str())
    .execute(pool)
    .await
    .expect("record the knowledge worker's manual role");
    Credentials {
        non_admin,
        developer,
        admin,
        platform_admin,
        project_manager,
        knowledge_worker,
        non_admin_user_id,
    }
}

async fn provision_one(
    pool: &PgPool,
    name: &str,
    roles: &[&str],
    in_group: bool,
) -> (String, UserId) {
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
    sqlx::query("INSERT INTO user_profile_ext (user_id) VALUES ($1) ON CONFLICT DO NOTHING")
        .bind(user_id.as_str())
        .execute(pool)
        .await
        .expect("seed the principal's profile row");
    if in_group {
        sqlx::query(
            "INSERT INTO group_members (group_id, user_id, source) VALUES ($1, $2, 'adfs')",
        )
        .bind(CONTRACT_GROUP)
        .bind(user_id.as_str())
        .execute(pool)
        .await
        .expect("place the principal in the contract group");
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

    (token.as_str().to_owned(), user_id)
}
