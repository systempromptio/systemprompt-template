//! Persists marketplace versions: one row per (marketplace, content hash),
//! the open row being the version served now.
//!
//! Recording is idempotent. A hash seen before reopens its row; a hash that
//! moved closes the previous row at the moment of observation; a marketplace
//! that left the composition has its open row closed. History is never
//! rewritten, so the intervals let a fact be resolved to the version that
//! was serving when it happened.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::identifiers::MarketplaceId;
use systemprompt_web_shared::error::MarketplaceError;

use super::marketplace_hash::{MarketplaceManifest, MarketplaceVersion};

#[derive(Debug, Default, Clone, Copy)]
pub struct VersionsRecorded {
    pub marketplaces: usize,
    pub new_versions: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceVersionRow {
    pub marketplace_id: MarketplaceId,
    pub content_hash: String,
    pub source: String,
    pub source_hash: Option<String>,
    pub manifest: Option<Json<MarketplaceManifest>>,
    pub plugin_count: i32,
    pub skill_count: i32,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub effective_until: Option<DateTime<Utc>>,
    pub origin: String,
}

pub async fn record_marketplace_versions(
    pool: &PgPool,
    versions: &[MarketplaceVersion],
) -> Result<VersionsRecorded, MarketplaceError> {
    let mut tx = pool.begin().await?;
    let ids: Vec<String> = versions
        .iter()
        .map(|v| v.marketplace_id.as_str().to_owned())
        .collect();
    sqlx::query!(
        "UPDATE marketplace_versions SET effective_until = now()
         WHERE effective_until IS NULL AND NOT (marketplace_id = ANY($1::text[]))",
        &ids,
    )
    .execute(&mut *tx)
    .await?;

    let mut new_versions = 0usize;
    for v in versions {
        let closed = sqlx::query!(
            "UPDATE marketplace_versions SET effective_until = now()
             WHERE marketplace_id = $1 AND effective_until IS NULL AND content_hash <> $2",
            v.marketplace_id.as_str(),
            v.content_hash,
        )
        .execute(&mut *tx)
        .await?
        .rows_affected();
        let fresh = sqlx::query_scalar!(
            r#"INSERT INTO marketplace_versions
                 (marketplace_id, content_hash, source, source_hash, manifest, plugin_count, skill_count)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               ON CONFLICT (marketplace_id, content_hash) DO UPDATE SET
                 last_seen_at = now(), effective_until = NULL, origin = 'manifest',
                 source = EXCLUDED.source, source_hash = EXCLUDED.source_hash,
                 manifest = EXCLUDED.manifest, plugin_count = EXCLUDED.plugin_count,
                 skill_count = EXCLUDED.skill_count
               RETURNING (xmax = 0) AS "inserted!""#,
            v.marketplace_id.as_str(),
            v.content_hash,
            v.source,
            v.source_hash,
            Json(&v.manifest) as _,
            v.plugin_count,
            v.skill_count,
        )
        .fetch_one(&mut *tx)
        .await?;
        if fresh || closed > 0 {
            new_versions += 1;
        }
    }
    tx.commit().await?;
    Ok(VersionsRecorded {
        marketplaces: versions.len(),
        new_versions,
    })
}

pub async fn list_current_marketplace_versions(
    pool: &PgPool,
) -> Result<Vec<MarketplaceVersionRow>, sqlx::Error> {
    sqlx::query_as!(
        MarketplaceVersionRow,
        r#"SELECT marketplace_id AS "marketplace_id: MarketplaceId", content_hash, source, source_hash,
                  manifest AS "manifest: Json<MarketplaceManifest>", plugin_count, skill_count,
                  first_seen_at, last_seen_at, effective_until, origin
           FROM marketplace_versions WHERE effective_until IS NULL ORDER BY marketplace_id"#
    )
    .fetch_all(pool)
    .await
}
