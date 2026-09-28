//! The sync trail: who refreshed the sources, who applied which plane, and
//! for which marketplaces.
//!
//! Every sync write already leaves a `user_activity` row under entity kind
//! `sync` (`activity::constructors_entity::sync`). This reads those rows back
//! as one list for the Code sync page, so an apply a participant made on
//! their own marketplace is followable by name, plane, mode and time next to
//! every administrator's. A participant's view keeps the instance-wide
//! source refreshes (they move the kits everyone serves) and the applies
//! whose recorded marketplaces intersect their own.
//!
//! The same rows carry the per-entity "keep the database" decisions taken
//! on the access-control review, read back by [`list_kept_reviews`].

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{MarketplaceId, UserId};

const SYNC_ENTITY: &str = "sync";
const SOURCES_PLANE: &str = "sources";
// Why: the action a "keep the database" decision is recorded under — it
// rejects the change code proposes.
pub const KEPT_ACTION: &str = "rejected";

#[derive(Debug, Clone)]
pub struct SyncHistoryRow {
    pub id: String,
    pub user_id: UserId,
    pub display_name: String,
    pub action: String,
    pub plane: Option<String>,
    pub description: String,
    pub marketplaces: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// Which rows a reader may see: every sync row, or the source refreshes plus
/// applies that touched one of the named marketplaces.
#[derive(Debug, Clone, Copy)]
pub enum SyncHistoryScope<'a> {
    All,
    Marketplaces(&'a [MarketplaceId]),
}

// Why: lint-ok: unused-pub — the /admin/sync trail reads it; its page lands
// with the Stage-3 admin port.
pub async fn list_sync_history(
    pool: &PgPool,
    scope: SyncHistoryScope<'_>,
    limit: i64,
) -> Result<Vec<SyncHistoryRow>, sqlx::Error> {
    let marketplaces: Option<Vec<String>> = match scope {
        SyncHistoryScope::All => None,
        SyncHistoryScope::Marketplaces(ids) => {
            Some(ids.iter().map(|m| m.as_str().to_owned()).collect())
        },
    };
    let rows = sqlx::query!(
        r#"SELECT a.id, a.user_id,
                  COALESCE(u.display_name, u.full_name, u.name, u.email, a.user_id) AS "display_name!",
                  a.action, a.entity_id, a.description, a.created_at,
                  COALESCE(
                      ARRAY(SELECT jsonb_array_elements_text(a.metadata -> 'marketplaces')),
                      ARRAY[]::TEXT[]
                  ) AS "marketplaces!: Vec<String>"
           FROM user_activity a
           JOIN users u ON u.id = a.user_id
           WHERE a.entity_type = $1
             AND ($2::TEXT[] IS NULL
                  OR a.entity_id = $3
                  OR EXISTS (
                      SELECT 1 FROM jsonb_array_elements_text(a.metadata -> 'marketplaces') m
                      WHERE m = ANY($2)))
           ORDER BY a.created_at DESC
           LIMIT $4"#,
        SYNC_ENTITY,
        marketplaces.as_deref(),
        SOURCES_PLANE,
        limit
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| SyncHistoryRow {
            id: r.id,
            user_id: UserId::new(r.user_id),
            display_name: r.display_name,
            action: r.action,
            plane: r.entity_id,
            description: r.description,
            marketplaces: r.marketplaces,
            created_at: r.created_at,
        })
        .collect())
}

/// A "keep the database" decision on one access-control entity: who took
/// it, why, and the diff fingerprint it was taken against.
#[derive(Debug, Clone)]
pub struct KeptReview {
    pub key: String,
    pub fingerprint: String,
    pub reason: String,
    pub display_name: String,
    pub created_at: DateTime<Utc>,
}

// Why: the latest decision per entity only. An older one was taken against
// a diff that has since moved, and the fingerprint is how the caller tells.
pub async fn list_kept_reviews(pool: &PgPool) -> Result<Vec<KeptReview>, sqlx::Error> {
    sqlx::query_as!(
        KeptReview,
        r#"SELECT DISTINCT ON (a.metadata ->> 'kept_entity')
                  a.metadata ->> 'kept_entity' AS "key!",
                  COALESCE(a.metadata ->> 'fingerprint', '') AS "fingerprint!",
                  COALESCE(a.metadata ->> 'reason', '') AS "reason!",
                  COALESCE(u.display_name, u.full_name, u.name, u.email, a.user_id) AS "display_name!",
                  a.created_at
           FROM user_activity a
           JOIN users u ON u.id = a.user_id
           WHERE a.entity_type = $1
             AND a.action = $2
             AND a.metadata ->> 'kept_entity' IS NOT NULL
           ORDER BY a.metadata ->> 'kept_entity', a.created_at DESC"#,
        SYNC_ENTITY,
        KEPT_ACTION,
    )
    .fetch_all(pool)
    .await
}
