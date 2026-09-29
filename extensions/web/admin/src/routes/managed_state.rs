//! Managed-resource services constructed once at the admin composition root.

use systemprompt::database::DbPool;
use systemprompt::marketplace::managed::ManagedRepository;

/// Why a state can fail to build: every core repository opens its own
/// handles from the shared [`DbPool`], and each constructor reports that.
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    #[error(transparent)]
    Managed(#[from] systemprompt::marketplace::managed::ManagedError),
    #[error(transparent)]
    Ai(#[from] systemprompt::ai::error::RepositoryError),
    #[error(transparent)]
    Users(#[from] systemprompt::users::UserError),
}

#[derive(Debug, Clone)]
pub(crate) struct ManagedState {
    pub(crate) owner: systemprompt::identifiers::UserId,
    pub(crate) repository: ManagedRepository,
    // Why: core's ledger of the `otlp_export` job, read by the observability
    // page; the job itself writes it.
    pub(crate) otlp_export: systemprompt::scheduler::OtlpExportStateRepository,
}

impl ManagedState {
    pub(crate) fn new(
        db: &DbPool,
        owner: systemprompt::identifiers::UserId,
    ) -> Result<Self, StateError> {
        let pool = db.write_pool().as_ref().clone();
        Ok(Self {
            owner,
            otlp_export: systemprompt::scheduler::OtlpExportStateRepository::new(pool),
            repository: ManagedRepository::new(db)?,
        })
    }
}
