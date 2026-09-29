//! Performance keyed on marketplace version: every figure below is grouped
//! by the content hash of the marketplace a skill was served from.
//!
//! The version is the one being served when a conversation first invoked the
//! skill (`conversation_skill_facts`, fixed at that instant). Invocations and
//! failures are the skill's own; users, requests, spend, tokens and latency
//! are the figures of the conversations that invoked it
//! (`conversation_facts`), counted once per conversation within a version.
//! A conversation that used several versions or skills counts under each, so
//! never sum them across versions or skills. Spend is the conversation's
//! turns — its side calls (titles, summaries) are excluded — and latency is
//! the median of the conversations' p50 and the 95th percentile of their
//! p95. Both tables are kept forever, so a version's figures outlive
//! raw-event retention.
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::identifiers::{MarketplaceId, PluginId};

use crate::repositories::sync::marketplace_hash::MarketplaceManifest;

/// Half-open UTC window `[start, end)` applied to the instant a conversation
/// first invoked a skill.
#[derive(Debug, Clone, Copy)]
pub struct VersionWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// One marketplace on the Versions landing: its current version, how many
/// versions it has had, and the window's figures across all of them.
#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceListRow {
    pub marketplace_id: MarketplaceId,
    pub name: Option<String>,
    pub content_hash: Option<String>,
    pub source: Option<String>,
    pub source_hash: Option<String>,
    pub plugin_count: Option<i32>,
    pub skill_count: Option<i32>,
    pub versions: i64,
    pub first_seen_at: DateTime<Utc>,
    pub last_changed_at: DateTime<Utc>,
    pub invocations: i64,
    pub failed_invocations: i64,
    pub users: i64,
    pub requests: i64,
    pub failed: i64,
    pub cost: i64,
}

/// One version of one marketplace with the window's figures.
#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceVersionMetricsRow {
    pub marketplace_id: MarketplaceId,
    pub content_hash: String,
    pub source: String,
    pub source_hash: Option<String>,
    pub origin: String,
    pub manifest: Option<Json<MarketplaceManifest>>,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
    pub effective_until: Option<DateTime<Utc>>,
    pub plugin_count: i32,
    pub skill_count: i32,
    pub invocations: i64,
    pub failed_invocations: i64,
    pub users: i64,
    pub conversations: i64,
    pub requests: i64,
    pub failed: i64,
    pub tokens: i64,
    pub cost: i64,
    pub p50_ms: Option<f64>,
    pub p95_ms: Option<f64>,
    pub latency_measured: i64,
}

/// One skill within one marketplace version.
#[derive(Debug, Clone, Serialize)]
pub struct MarketplaceVersionSkillRow {
    pub marketplace_id: MarketplaceId,
    pub marketplace_hash: String,
    pub plugin_id: PluginId,
    pub skill: String,
    pub invocations: i64,
    pub failed_invocations: i64,
    pub users: i64,
    pub conversations: i64,
    pub requests: i64,
    pub failed: i64,
    pub tokens: i64,
    pub cost: i64,
    pub p50_ms: Option<f64>,
    pub p95_ms: Option<f64>,
    pub latency_measured: i64,
}

pub async fn list_marketplace_rollups(
    pool: &PgPool,
    window: VersionWindow,
) -> Result<Vec<MarketplaceListRow>, sqlx::Error> {
    sqlx::query_file_as!(
        MarketplaceListRow,
        "src/repositories/analysis/marketplace_version_list.sql",
        window.start,
        window.end,
    )
    .fetch_all(pool)
    .await
}

pub async fn list_marketplace_version_metrics(
    pool: &PgPool,
    window: VersionWindow,
    marketplace_id: &MarketplaceId,
) -> Result<Vec<MarketplaceVersionMetricsRow>, sqlx::Error> {
    sqlx::query_file_as!(
        MarketplaceVersionMetricsRow,
        "src/repositories/analysis/marketplace_version_metrics.sql",
        window.start,
        window.end,
        marketplace_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

// Why: every skill of one marketplace under each of the named versions — the
// compare view names exactly two.
pub async fn list_marketplace_version_skills(
    pool: &PgPool,
    window: VersionWindow,
    marketplace_id: &MarketplaceId,
    hashes: &[String],
) -> Result<Vec<MarketplaceVersionSkillRow>, sqlx::Error> {
    sqlx::query_file_as!(
        MarketplaceVersionSkillRow,
        "src/repositories/analysis/marketplace_version_skills.sql",
        window.start,
        window.end,
        marketplace_id.as_str(),
        hashes,
    )
    .fetch_all(pool)
    .await
}

/// The judge's completion over the conversations a marketplace version served.
///
/// A conversation is tied to the version being served when it first invoked
/// one of the marketplace's skills (`conversation_skill_facts`).
#[derive(Debug, Clone)]
pub struct VersionCompletionRow {
    pub marketplace_id: MarketplaceId,
    pub content_hash: String,
    pub is_current: bool,
    pub conversations: i64,
    pub judged: i64,
    pub completion_avg: Option<f64>,
}

pub async fn list_version_completion(
    pool: &PgPool,
    marketplace: Option<&MarketplaceId>,
) -> Result<Vec<VersionCompletionRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT v.marketplace_id AS "marketplace_id!: MarketplaceId", v.content_hash AS "content_hash!",
                  (v.effective_until IS NULL) AS "is_current!",
                  COUNT(DISTINCT s.context_id)::bigint AS "conversations!",
                  COUNT(DISTINCT s.context_id) FILTER (WHERE a.completion IS NOT NULL)::bigint AS "judged!",
                  AVG(a.completion)::float8 AS completion_avg
           FROM marketplace_versions v
           LEFT JOIN conversation_skill_facts s
                  ON s.marketplace_id = v.marketplace_id AND s.marketplace_hash = v.content_hash
           LEFT JOIN conversation_analyses a ON a.context_id = s.context_id
           WHERE $1::text IS NULL OR v.marketplace_id = $1
           GROUP BY v.marketplace_id, v.content_hash, v.effective_until
           ORDER BY v.marketplace_id, v.first_seen_at DESC"#,
        marketplace.map(MarketplaceId::as_str)
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| VersionCompletionRow {
            marketplace_id: r.marketplace_id,
            content_hash: r.content_hash,
            is_current: r.is_current,
            conversations: r.conversations,
            judged: r.judged,
            completion_avg: r.completion_avg,
        })
        .collect())
}

/// A marketplace's completion on its current version against every earlier
/// one, weighted by judged conversations.
#[derive(Debug, Clone)]
pub struct MarketplaceCompletionRollup {
    pub marketplace_id: MarketplaceId,
    pub current_judged: i64,
    pub current_completion: Option<f64>,
    pub history_judged: i64,
    pub history_completion: Option<f64>,
}

pub async fn list_marketplace_completion_rollups(
    pool: &PgPool,
) -> Result<Vec<MarketplaceCompletionRollup>, sqlx::Error> {
    let rows = list_version_completion(pool, None).await?;
    let mut by_market: std::collections::BTreeMap<String, MarketplaceCompletionRollup> =
        std::collections::BTreeMap::new();
    for row in rows {
        let entry = by_market
            .entry(row.marketplace_id.as_str().to_owned())
            .or_insert_with(|| MarketplaceCompletionRollup {
                marketplace_id: row.marketplace_id.clone(),
                current_judged: 0,
                current_completion: None,
                history_judged: 0,
                history_completion: None,
            });
        let (judged, completion) = if row.is_current {
            (&mut entry.current_judged, &mut entry.current_completion)
        } else {
            (&mut entry.history_judged, &mut entry.history_completion)
        };
        if let Some(avg) = row.completion_avg.filter(|_| row.judged > 0) {
            let weighted = completion
                .unwrap_or(0.0)
                .mul_add(*judged as f64, avg * (row.judged as f64));
            *judged += row.judged;
            *completion = Some(weighted / (*judged as f64));
        }
    }
    Ok(by_market.into_values().collect())
}
