//! Managed-resource services constructed once at the admin composition root.

use sqlx::PgPool;
use systemprompt::marketplace::managed::ManagedRepository;

#[derive(Debug, Clone)]
pub(crate) struct ManagedState {
    pub(crate) owner: systemprompt::identifiers::UserId,
    pub(crate) repository: ManagedRepository,
}

impl ManagedState {
    pub(crate) const fn new(pool: PgPool, owner: systemprompt::identifiers::UserId) -> Self {
        Self {
            owner,
            repository: ManagedRepository::new(pool),
        }
    }
}
