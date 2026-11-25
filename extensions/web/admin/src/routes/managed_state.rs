//! Managed-resource services constructed once at the admin composition root.

use systemprompt::database::DbPool;
use systemprompt::marketplace::managed::ManagedRepository;

#[derive(Debug, Clone)]
pub(crate) struct ManagedState {
    pub(crate) owner: systemprompt::identifiers::UserId,
    pub(crate) repository: ManagedRepository,
}

impl ManagedState {
    pub(crate) fn new(db: &DbPool, owner: systemprompt::identifiers::UserId) -> Self {
        Self {
            owner,
            repository: ManagedRepository::new(db),
        }
    }
}
