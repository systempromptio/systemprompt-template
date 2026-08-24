//! Subject dimensions this extension adds to core's authorization resolver.
//!
//! Core resolves `user` and `role` and deliberately knows nothing else. Every
//! other dimension an operator wants to write rules against is a tenant
//! concept, declared here: a [`SubjectDimension`] describing where it sits in
//! the precedence ladder, and a [`SubjectAttributeProvider`][p] that looks up
//! the values a user holds for it.
//!
//! We currently declare one, [`department`]. Adding a second — cost centre,
//! clearance, jurisdiction — means writing a provider beside it and one
//! `register_subject_attribute_provider!` call; no core change, and no edit to
//! the resolve call sites, because they all read the registry through
//! [`subject_attributes_for`] and [`dimensions`].
//!
//! [p]: systemprompt_security::authz::SubjectAttributeProvider

pub mod department;

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_security::authz::{
    AuthzHookContext, NullAuditSink, SharedSubjectAttributeProvider, SubjectAttributes,
    SubjectDimension, dimensions_of, discover_subject_providers, gather_subject_attributes,
};

use crate::authz::department::DepartmentAttributeProvider;

systemprompt_security::register_subject_attribute_provider!(|ctx| {
    let provider: SharedSubjectAttributeProvider =
        Arc::new(DepartmentAttributeProvider::new(Arc::clone(&ctx.pool)));
    provider
});

struct Registry {
    providers: Vec<SharedSubjectAttributeProvider>,
    dimensions: Vec<SubjectDimension>,
}

// Why: the providers close over the pool they were built with, so a single
// process-wide registry answers every later caller from whichever database
// asked first. One server has one database and never noticed; a test process
// has one per test, and every test after the first silently resolved its
// users against another test's database and saw no attributes at all.
//
// Keyed by the database the pool points at, and leaked so the borrow can stay
// `'static` for the call sites: one entry per distinct database, which is one
// in production.
static REGISTRIES: LazyLock<Mutex<HashMap<String, &'static Registry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn database_key(pool: &PgPool) -> String {
    let opts = pool.connect_options();
    format!(
        "{}:{}/{}",
        opts.get_host(),
        opts.get_port(),
        opts.get_database().unwrap_or_default()
    )
}

fn registry(pool: &PgPool) -> &'static Registry {
    let key = database_key(pool);
    let mut registries = REGISTRIES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    if let Some(existing) = registries.get(&key) {
        return existing;
    }

    let providers = discover_subject_providers(&AuthzHookContext {
        pool: Arc::new(pool.clone()),
        sink: Arc::new(NullAuditSink),
    });
    let registry: &'static Registry = Box::leak(Box::new(Registry {
        dimensions: dimensions_of(&providers),
        providers,
    }));
    registries.insert(key, registry);
    registry
}

pub fn dimensions(pool: &PgPool) -> &'static [SubjectDimension] {
    &registry(pool).dimensions
}

// Why: The subject's values for every registered dimension. The one async step
// in the authorization path; call it once per request and reuse the result
// across entities.
pub async fn subject_attributes_for(pool: &PgPool, user_id: &UserId) -> SubjectAttributes {
    gather_subject_attributes(&registry(pool).providers, user_id).await
}
