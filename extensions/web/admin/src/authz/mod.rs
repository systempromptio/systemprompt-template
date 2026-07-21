//! Subject dimensions this template adds to core's authorization resolver.
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

use std::sync::{Arc, OnceLock};

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

static REGISTRY: OnceLock<Registry> = OnceLock::new();

/// Builds every registered provider once, against the first pool to ask.
///
/// The providers are stateless apart from their pool handle and their caches,
/// so binding them once per process — rather than per request — is what makes
/// the enforcement path cheap enough to run on every tool call.
///
/// The audit sink in the context is a [`NullAuditSink`]: providers look
/// attributes up, they do not decide, so they have nothing to audit. The
/// decision that uses their output is audited by its own call site.
fn registry(pool: &PgPool) -> &'static Registry {
    REGISTRY.get_or_init(|| {
        let providers = discover_subject_providers(&AuthzHookContext {
            pool: Arc::new(pool.clone()),
            sink: Arc::new(NullAuditSink),
        });
        Registry {
            dimensions: dimensions_of(&providers),
            providers,
        }
    })
}

/// The dimension ladder to hand [`resolve`][systemprompt_security::authz::resolve].
pub fn dimensions(pool: &PgPool) -> &'static [SubjectDimension] {
    &registry(pool).dimensions
}

/// The subject's values for every registered dimension. The one async step in
/// the authorization path; call it once per request and reuse the result
/// across entities.
pub async fn subject_attributes_for(pool: &PgPool, user_id: &UserId) -> SubjectAttributes {
    gather_subject_attributes(&registry(pool).providers, user_id).await
}
