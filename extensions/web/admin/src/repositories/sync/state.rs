//! The `sync_state` table: one row per plane recording what was last applied
//! from code.
//!
//! Written by the boot seed and by every apply; read by the page so the
//! plane card can say which declared hash the database last received, when,
//! from whom and in which mode — and therefore whether the declaration has
//! changed since.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

use super::plane::SyncMode;

// Why: the boot seed is the one write nobody presses, so it carries a fixed
// actor rather than a person.
pub const BOOT_ACTOR: &str = "boot";

#[derive(Debug, Clone, Serialize)]
pub struct SyncStateRow {
    pub plane: String,
    pub declared_hash: String,
    pub applied_hash: Option<String>,
    pub applied_mode: Option<String>,
    pub applied_at: Option<DateTime<Utc>>,
    pub applied_by: Option<String>,
    pub base_tree_hash: Option<String>,
    pub composed_hash: Option<String>,
    pub updated_at: DateTime<Utc>,
}

pub async fn find_sync_state(
    pool: &PgPool,
    plane: &str,
) -> Result<Option<SyncStateRow>, sqlx::Error> {
    sqlx::query_as!(
        SyncStateRow,
        r"SELECT plane, declared_hash, applied_hash, applied_mode, applied_at, applied_by,
                 base_tree_hash, composed_hash, updated_at
            FROM sync_state
           WHERE plane = $1",
        plane
    )
    .fetch_optional(pool)
    .await
}

/// What one apply pins: the tree and composition it ran against.
#[derive(Debug, Clone, Default)]
pub struct AppliedFrom {
    pub base_tree_hash: Option<String>,
    pub composed_hash: Option<String>,
}

// Why: the seed is its own mode because it is the one write nobody chose.
#[must_use]
pub fn applied_mode_label(mode: Option<SyncMode>) -> &'static str {
    mode.map_or("seed", SyncMode::label)
}

/// One apply to record: which plane, which declared hash it made the database
/// equal to, in which mode (`None` = the boot seed), by whom, and from which
/// tree and composition.
#[derive(Debug, Clone)]
pub struct Applied<'a> {
    pub plane: &'a str,
    pub declared_hash: &'a str,
    pub mode: Option<SyncMode>,
    pub actor: &'a str,
    pub from: AppliedFrom,
}

pub async fn record_applied(pool: &PgPool, applied: &Applied<'_>) -> Result<(), sqlx::Error> {
    let Applied {
        plane,
        declared_hash,
        mode,
        actor,
        from,
    } = applied;
    let mode_label = applied_mode_label(*mode);
    sqlx::query!(
        r"INSERT INTO sync_state
              (plane, declared_hash, applied_hash, applied_mode, applied_at, applied_by,
               base_tree_hash, composed_hash, updated_at)
          VALUES ($1, $2, $2, $3, NOW(), $4, $5, $6, NOW())
          ON CONFLICT (plane) DO UPDATE
             SET declared_hash = EXCLUDED.declared_hash,
                 applied_hash = EXCLUDED.applied_hash,
                 applied_mode = EXCLUDED.applied_mode,
                 applied_at = EXCLUDED.applied_at,
                 applied_by = EXCLUDED.applied_by,
                 base_tree_hash = EXCLUDED.base_tree_hash,
                 composed_hash = EXCLUDED.composed_hash,
                 updated_at = NOW()",
        plane,
        declared_hash,
        mode_label,
        actor,
        from.base_tree_hash,
        from.composed_hash,
    )
    .execute(pool)
    .await?;
    Ok(())
}

// Why: the declared hash is refreshed on every read so "declared changed
// since last apply" is answered from the row alone; the applied columns are
// untouched, and a missing row is created with no apply on it.
pub async fn record_declared(
    pool: &PgPool,
    plane: &str,
    declared_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"INSERT INTO sync_state (plane, declared_hash)
          VALUES ($1, $2)
          ON CONFLICT (plane) DO UPDATE
             SET declared_hash = EXCLUDED.declared_hash,
                 updated_at = CASE WHEN sync_state.declared_hash IS DISTINCT FROM EXCLUDED.declared_hash
                                   THEN NOW() ELSE sync_state.updated_at END",
        plane,
        declared_hash,
    )
    .execute(pool)
    .await?;
    Ok(())
}
