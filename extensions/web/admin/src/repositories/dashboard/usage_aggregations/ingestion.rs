//! The ingestion outbox: hook events land durably first and the aggregates
//! are drained from it, so a crash between the two never double-counts.

use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

const DRAIN_BATCH: i32 = 100;

// Why: binds the session to the authenticated principal before anything is
// recorded against it, so a hook carrying someone else's session id is
// refused by the database rather than trusted by the handler.
pub(crate) async fn assert_ingestion_owner(
    pool: &PgPool,
    session_id: &SessionId,
    user_id: &UserId,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "SELECT assert_ingestion_owner($1, $2)",
        session_id.as_str(),
        user_id.as_str()
    )
    .fetch_one(pool)
    .await?;
    Ok(())
}

pub(crate) async fn drain_ingestion_outbox(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!("SELECT drain_ingestion_outbox($1)", DRAIN_BATCH)
        .fetch_one(pool)
        .await?;
    Ok(())
}
