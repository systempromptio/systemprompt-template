//! Query-local settings for interactive dashboard aggregation.
//!
//! At one million requests, LLVM JIT compilation alone cost ~2 seconds for
//! the conversation page. Disable it only inside these read transactions;
//! pooled connections revert to their original setting on commit/rollback.
use sqlx::{PgPool, Postgres, Transaction};

pub(crate) async fn begin(pool: &PgPool) -> Result<Transaction<'static, Postgres>, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    sqlx::query!("SET LOCAL jit = off")
        .execute(&mut *transaction)
        .await?;
    Ok(transaction)
}

// Why: resolve a JSON row's key to the ID decoded by SQLx in the same
// statement, preserving the database decoding contract without unchecked
// constructors.
pub(crate) fn context_id(
    key: &str,
    ids: &[systemprompt::identifiers::ContextId],
) -> Result<systemprompt::identifiers::ContextId, sqlx::Error> {
    ids.iter()
        .find(|id| id.as_str() == key)
        .cloned()
        .ok_or_else(|| sqlx::Error::Decode("page context ID missing from database result".into()))
}
