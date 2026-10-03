//! Managed-resource services constructed once at the admin composition root.

use systemprompt::database::DbPool;
use systemprompt::marketplace::managed::ManagedRepository;

#[derive(Debug, Clone)]
pub(crate) struct ManagedState {
    pub(crate) owner: systemprompt::identifiers::UserId,
    pub(crate) repository: ManagedRepository,
    // Why: core's ledger of the `otlp_export` job, read by the observability
    // page; the job itself writes it.
    pub(crate) otlp_export: systemprompt::scheduler::OtlpExportStateRepository,
}

impl ManagedState {
    pub(crate) fn new(db: &DbPool, owner: systemprompt::identifiers::UserId) -> Self {
        let pool = db.write_pool().as_ref().clone();
        Self {
            owner,
            otlp_export: systemprompt::scheduler::OtlpExportStateRepository::new(pool),
            repository: ManagedRepository::new(db),
        }
    }
}
