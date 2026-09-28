//! The sweep's view of membership: rows whose window has closed, stamped
//! `revoked_at` so they stop counting and stay as the record of what was held.
//!
//! The read predicate already hides an expired row the moment its window
//! passes, so this is not what makes expiry bind — it is what makes it
//! durable and visible, and what tells the caller whose scope defaults to
//! recompute. Directory-sourced rows never carry a window and are never
//! touched here.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

/// The people whose membership rows the sweep just revoked.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ExpiredMemberships {
    pub group_rows: u64,
    pub project_rows: u64,
    pub users: Vec<UserId>,
}

pub async fn revoke_expired_memberships(pool: &PgPool) -> Result<ExpiredMemberships, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let groups = sqlx::query_scalar!(
        r#"UPDATE group_members SET revoked_at = CURRENT_TIMESTAMP
            WHERE revoked_at IS NULL AND valid_until IS NOT NULL
              AND valid_until <= CURRENT_TIMESTAMP
            RETURNING user_id AS "user_id!: UserId""#
    )
    .fetch_all(&mut *tx)
    .await?;
    let projects = sqlx::query_scalar!(
        r#"UPDATE project_members SET revoked_at = CURRENT_TIMESTAMP
            WHERE revoked_at IS NULL AND valid_until IS NOT NULL
              AND valid_until <= CURRENT_TIMESTAMP
            RETURNING user_id AS "user_id!: UserId""#
    )
    .fetch_all(&mut *tx)
    .await?;
    tx.commit().await?;

    let group_rows = u64::try_from(groups.len()).unwrap_or(u64::MAX);
    let project_rows = u64::try_from(projects.len()).unwrap_or(u64::MAX);
    let mut users: Vec<UserId> = groups.into_iter().chain(projects).collect();
    users.sort();
    users.dedup();
    Ok(ExpiredMemberships {
        group_rows,
        project_rows,
        users,
    })
}
