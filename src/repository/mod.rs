use systemprompt_core_database::DbPool;

pub struct AdminRepository {
    _db_pool: DbPool,
}

impl AdminRepository {
    #[must_use] pub fn new(db_pool: DbPool) -> Self {
        Self { _db_pool: db_pool }
    }
}
