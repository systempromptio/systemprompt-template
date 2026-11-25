//! Subject dimensions this extension adds to core's authorization resolver.
//!
//! Core resolves `user` and `role` and deliberately knows nothing else. Every
//! other dimension an operator wants to write rules against is a tenant
//! concept, declared here: a [`SubjectDimension`] describing where it sits in
//! the precedence ladder, and a [`SubjectAttributeProvider`][p] that looks up
//! the values a user holds for it.
//!
//! We declare four: [`project`], [`group`], [`connector`] and
//! [`organization`]. They form a ladder with core's — user (0), `project`
//! (140), `group` (150), `connector` (160), role (200), `organization` (300)
//! — where a lower number is the narrower, higher-priority scope. Groups
//! carry people and marketplace entitlement; projects are work attribution.
//! Both are DB rows an operator edits, and the directory's AD groups map into
//! them rather than being a dimension of their own. `connector` is the MCP
//! servers a person holds a ready connection to. A fifth provider, [`role`],
//! re-exposes core's own `role` dimension: the resolver ignores it, but the
//! gateway's quota resolver finds subjects only through this registry, so
//! without it a `subject: role` window has nothing to key on. The same goes
//! for `organization`, which is why it exists as a dimension at all.
//! Adding another — cost centre, clearance, jurisdiction — means writing a
//! provider beside them and one `subject_attributes` line in
//! [`contribute_subject_attributes`]; no core change, and no edit to
//! the resolve call sites, because they all read the registry through
//! [`subject_attributes_for`] and [`dimensions`].
//!
//! [p]: systemprompt::governance::authz::SubjectAttributeProvider

mod account;
pub use account::{AccountStanding, account_scope, account_standing};
pub(crate) mod catalog;
pub mod connector;
pub mod group;
pub mod organization;
pub mod primary;
pub mod project;
pub mod role;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::extension::{Contributions, FactoryError, Resolved};
use systemprompt::governance::authz::{
    AuthzHookContext, NullAuditSink, SharedSubjectAttributeProvider, SubjectAttributes,
    SubjectDimension, dimensions_of, discover_subject_providers, gather_subject_attributes,
};
use systemprompt::identifiers::UserId;
use systemprompt::traits::authz::SubjectAttributeProvider;

use crate::authz::connector::ConnectorAttributeProvider;
use crate::authz::group::GroupAttributeProvider;
use crate::authz::organization::OrganizationAttributeProvider;
use crate::authz::project::ProjectAttributeProvider;
use crate::authz::role::RoleAttributeProvider;

pub fn contribute_subject_attributes(c: &mut Contributions<'_>) {
    c.subject_attributes(|r: &Resolved<'_>| {
        let provider: Arc<dyn SubjectAttributeProvider> =
            Arc::new(GroupAttributeProvider::new(pg_pool(r)?));
        Ok(provider)
    })
    .subject_attributes(|r: &Resolved<'_>| {
        let provider: Arc<dyn SubjectAttributeProvider> =
            Arc::new(ProjectAttributeProvider::new(pg_pool(r)?));
        Ok(provider)
    })
    .subject_attributes(|r: &Resolved<'_>| {
        let provider: Arc<dyn SubjectAttributeProvider> =
            Arc::new(ConnectorAttributeProvider::new(pg_pool(r)?));
        Ok(provider)
    })
    .subject_attributes(|_r: &Resolved<'_>| {
        let provider: Arc<dyn SubjectAttributeProvider> = Arc::new(OrganizationAttributeProvider);
        Ok(provider)
    })
    .subject_attributes(|r: &Resolved<'_>| {
        let provider: Arc<dyn SubjectAttributeProvider> =
            Arc::new(RoleAttributeProvider::new(pg_pool(r)?));
        Ok(provider)
    });
}

fn pg_pool(r: &Resolved<'_>) -> Result<Arc<PgPool>, FactoryError> {
    Ok(r.get::<DbPool>()?.pool())
}

struct Registry {
    providers: Vec<SharedSubjectAttributeProvider>,
    dimensions: Vec<SubjectDimension>,
}

static REGISTRIES: OnceLock<Mutex<HashMap<String, &'static Registry>>> = OnceLock::new();

// Why: keyed per database, not once per process. The providers capture the
// pool they are built with, so a single OnceLock would bind every later
// caller to whichever pool arrived first — wrong in any process that talks
// to more than one database, which is exactly what the integration suite's
// per-test throwaway databases do. One registry per distinct database; the
// leak is bounded by the number of distinct databases a process ever opens
// (one in production).
// Why: shared with the marketplace-parent cache in the authz webhook so the
// two per-database keying schemes cannot drift apart.
pub(crate) fn database_key(pool: &PgPool) -> String {
    let options = pool.connect_options();
    format!(
        "{}:{}/{}",
        options.get_host(),
        options.get_port(),
        options.get_database().unwrap_or_default()
    )
}

fn registry(pool: &PgPool) -> &'static Registry {
    let key = database_key(pool);
    let map = REGISTRIES.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = map
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(existing) = guard.get(&key) {
        return existing;
    }
    let providers = discover_subject_providers(&AuthzHookContext {
        pool: Arc::new(pool.clone()),
        sink: Arc::new(NullAuditSink),
    });
    let built: &'static Registry = Box::leak(Box::new(Registry {
        dimensions: dimensions_of(&providers),
        providers,
    }));
    guard.insert(key, built);
    built
}

pub fn dimensions(pool: &PgPool) -> &'static [SubjectDimension] {
    &registry(pool).dimensions
}

// Why: The subject's values for every registered dimension. The one async step
// in the authorization path; call it once per request and reuse the result
// across entities.
pub async fn subject_attributes_for(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<SubjectAttributes, sqlx::Error> {
    gather_subject_attributes(&registry(pool).providers, user_id)
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))
}
