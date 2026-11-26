//! Subject dimensions this extension adds to core's authorization resolver.
//!
//! Core resolves `user` and `role` and deliberately knows nothing else. Every
//! other dimension an operator wants to write rules against is a tenant
//! concept, declared here: a [`SubjectDimension`] describing where it sits in
//! the precedence ladder, and a [`SubjectAttributeProvider`] that looks up
//! the values a user holds for it.
//!
//! We declare four: [`project`], [`group`], [`connector`] and
//! [`organization`]. They form a ladder with core's — user (0), `project`
//! (140), `group` (150), `connector` (160), role (200), `organization` (300)
//! — where a lower number is the narrower, higher-priority scope. Groups
//! carry people and marketplace entitlement; projects are work attribution.
//! Both are DB rows an operator edits, and the directory's AD groups map into
//! them rather than being a dimension of their own. `connector` is the MCP
//! servers a person holds a ready connection to. Core's `users` member
//! provides the `role` dimension itself (a reserved dimension no member may
//! declare), so the gateway's quota resolver finds a `subject: role` window
//! through core's provider.
//! Adding another — cost centre, clearance, jurisdiction — means writing a
//! provider beside them and one `subject_attributes` line in
//! [`contribute_subject_attributes`]; no core change, and no edit to
//! the resolve call sites, because they all read the registry through
//! [`subject_attributes_for`] and [`dimensions`].
mod account;
pub use account::{AccountStanding, account_scope, account_standing};
pub(crate) mod catalog;
pub mod connector;
pub mod group;
pub mod organization;
pub mod primary;
pub mod project;

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::database::DbPool;
use systemprompt::extension::{Contributions, FactoryError, Resolved};
use systemprompt::governance::authz::{
    SubjectAttributeSource, SubjectAttributes, SubjectDimension, SubjectSourceCell,
};
use systemprompt::identifiers::UserId;
use systemprompt::traits::authz::SubjectAttributeProvider;

use crate::authz::connector::ConnectorAttributeProvider;
use crate::authz::group::GroupAttributeProvider;
use crate::authz::organization::OrganizationAttributeProvider;
use crate::authz::project::ProjectAttributeProvider;

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
            Arc::new(ConnectorAttributeProvider::new(
                pg_pool(r)?,
                r.get::<systemprompt::manifest::services::InstallServicesView>()?,
            ));
        Ok(provider)
    })
    .subject_attributes(|_r: &Resolved<'_>| {
        let provider: Arc<dyn SubjectAttributeProvider> = Arc::new(OrganizationAttributeProvider);
        Ok(provider)
    });
}

// Why: the source the console stamps on a rule it writes; core keeps any
// `console:`-prefixed row through a services reconcile.
pub const CONSOLE_SOURCE: &str = "console:admin";

fn pg_pool(r: &Resolved<'_>) -> Result<Arc<PgPool>, FactoryError> {
    Ok(r.get::<DbPool>()?.pool())
}

// Why: every dimension the composition registered, core's and this member's,
// read through the subject source the kernel binds once the set is built.
pub fn dimensions(subjects: &SubjectSourceCell) -> Result<Vec<SubjectDimension>, sqlx::Error> {
    Ok(subjects
        .get()
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?
        .dimensions()
        .to_vec())
}

// Why: The subject's values for every registered dimension. The one async step
// in the authorization path; call it once per request and reuse the result
// across entities.
pub async fn subject_attributes_for(
    subjects: &SubjectSourceCell,
    user_id: &UserId,
) -> Result<SubjectAttributes, sqlx::Error> {
    subjects
        .get()
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))?
        .gather(user_id)
        .await
        .map_err(|e| sqlx::Error::Protocol(e.to_string()))
}
