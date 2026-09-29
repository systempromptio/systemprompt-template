//! Managed-resource services constructed once at the admin composition root.

use systemprompt::database::DbPool;
use systemprompt::marketplace::managed::ManagedRepository;

/// Why a state can fail to build: every core repository opens its own
/// handles from the shared [`DbPool`], and each constructor reports that.
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error(transparent)]
    Managed(#[from] systemprompt::marketplace::managed::ManagedError),
}

// Why: the analysis suite's revision, inventory and publication pages read
// core's managed-resource ledger through this one handle. Astound's copy also
// carries the `otlp_export` job ledger for the observability page, which joins
// with the Stage-3 phase-9 port.
#[derive(Debug, Clone)]
pub(crate) struct ManagedState {
    pub(crate) owner: systemprompt::identifiers::UserId,
    pub(crate) repository: ManagedRepository,
}

impl ManagedState {
    pub(crate) fn new(
        db: &DbPool,
        owner: systemprompt::identifiers::UserId,
    ) -> Result<Self, StateError> {
        Ok(Self {
            owner,
            repository: ManagedRepository::new(db)?,
        })
    }
}
