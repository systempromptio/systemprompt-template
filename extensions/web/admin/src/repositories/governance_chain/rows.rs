//! `governance_chain` and `governance_chain_settings` reads and writes.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

use super::declared::{DeclaredChain, DeclaredPolicy};

pub const SOURCE_CODE: &str = "code";

#[derive(Debug, Clone, Serialize)]
pub struct ChainPolicyRow {
    pub policy_id: String,
    pub position: i32,
    pub enabled: bool,
    pub mode: String,
    pub entry: serde_yaml::Value,
    pub source: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChainSettingsRow {
    pub enabled: bool,
    pub mode: String,
}

// Why: the staged chain as a declaration, for drift and export.
#[must_use]
pub fn staged_chain(settings: Option<&ChainSettingsRow>, rows: &[ChainPolicyRow]) -> DeclaredChain {
    let defaults = DeclaredChain::default();
    DeclaredChain {
        enabled: settings.map_or(defaults.enabled, |s| s.enabled),
        mode: settings.map_or(defaults.mode, |s| s.mode.clone()),
        policies: rows
            .iter()
            .map(|r| DeclaredPolicy {
                id: r.policy_id.clone(),
                enabled: r.enabled,
                mode: r.mode.clone(),
                entry: r.entry.clone(),
            })
            .collect(),
    }
}

pub async fn list_chain_policies(pool: &PgPool) -> Result<Vec<ChainPolicyRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r"SELECT policy_id, position, enabled, mode, entry, source, updated_at
            FROM governance_chain
           ORDER BY position ASC, policy_id ASC"
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| ChainPolicyRow {
            policy_id: r.policy_id,
            position: r.position,
            enabled: r.enabled,
            mode: r.mode,
            // JSON: JSONB column — the policy entry as written, read back as
            // YAML for the renderer; discard-ok: JSON is always YAML
            entry: serde_yaml::to_value(r.entry).unwrap_or(serde_yaml::Value::Null),
            source: r.source,
            updated_at: r.updated_at,
        })
        .collect())
}

pub async fn count_chain_policies(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(r#"SELECT COUNT(*) AS "count!" FROM governance_chain"#)
        .fetch_one(pool)
        .await
}

pub async fn find_chain_settings(pool: &PgPool) -> Result<Option<ChainSettingsRow>, sqlx::Error> {
    sqlx::query_as!(
        ChainSettingsRow,
        "SELECT enabled, mode FROM governance_chain_settings WHERE singleton"
    )
    .fetch_optional(pool)
    .await
}

pub async fn upsert_chain_settings(
    pool: &PgPool,
    enabled: bool,
    mode: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"INSERT INTO governance_chain_settings (singleton, enabled, mode, updated_at)
          VALUES (TRUE, $1, $2, NOW())
          ON CONFLICT (singleton) DO UPDATE
             SET enabled = EXCLUDED.enabled, mode = EXCLUDED.mode, updated_at = NOW()",
        enabled,
        mode
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn upsert_chain_policy(
    pool: &PgPool,
    policy: &DeclaredPolicy,
    position: i32,
    source: &str,
) -> Result<(), sqlx::Error> {
    // JSON: JSONB column — the entry as JSON; discard-ok: a value that
    // parsed as a policy entry serialises, Null is the visible sign if not
    let entry = serde_json::to_value(&policy.entry).unwrap_or(serde_json::Value::Null);
    sqlx::query!(
        r"INSERT INTO governance_chain (policy_id, position, enabled, mode, entry, source, updated_at)
          VALUES ($1, $2, $3, $4, $5, $6, NOW())
          ON CONFLICT (policy_id) DO UPDATE
             SET position = EXCLUDED.position,
                 enabled = EXCLUDED.enabled,
                 mode = EXCLUDED.mode,
                 entry = EXCLUDED.entry,
                 source = EXCLUDED.source,
                 updated_at = NOW()",
        policy.id,
        position,
        policy.enabled,
        policy.mode,
        entry,
        source
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_chain_policy(pool: &PgPool, policy_id: &str) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query!(
        "DELETE FROM governance_chain WHERE policy_id = $1",
        policy_id
    )
    .execute(pool)
    .await?
    .rows_affected())
}
