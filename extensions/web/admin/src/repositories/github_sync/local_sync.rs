use std::path::PathBuf;

use anyhow::Result;
use sqlx::PgPool;
use systemprompt::models::ProfileBootstrap;

use super::types::SyncResult;
use super::{elapsed_ms, finalize_sync, import_plugins_from_entries};

pub async fn sync_marketplace_from_local(
    pool: &PgPool,
    marketplace_id: &str,
) -> Result<SyncResult> {
    let start = std::time::Instant::now();

    let marketplace_json_path =
        PathBuf::from("storage/files/plugins/.claude-plugin/marketplace.json");
    if !marketplace_json_path.exists() {
        anyhow::bail!("Local marketplace.json not found");
    }

    let services_path = ProfileBootstrap::get()
        .map(|p| PathBuf::from(&p.paths.services))
        .map_err(|e| anyhow::anyhow!("Failed to get profile: {e}"))?;

    let content = std::fs::read_to_string(&marketplace_json_path)
        .map_err(|e| anyhow::anyhow!("Failed to read local marketplace.json: {e}"))?;
    let marketplace: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse local marketplace.json: {e}"))?;

    let plugins = marketplace
        .get("plugins")
        .and_then(|v| v.as_array())
        .ok_or_else(|| anyhow::anyhow!("Local marketplace.json missing 'plugins' array"))?;

    let base_path = PathBuf::from(".");
    let mut tally = import_plugins_from_entries(plugins, &base_path, &services_path, "local");

    finalize_sync(
        pool,
        marketplace_id,
        &tally.plugin_ids,
        &mut tally.error_count,
    )
    .await;

    let duration_ms = elapsed_ms(start);

    let _ = super::super::org_marketplaces::insert_sync_log(
        pool,
        &super::super::org_marketplaces::SyncLogEntry {
            marketplace_id,
            operation: "sync",
            status: "success",
            commit_hash: None,
            plugins_synced: i64::try_from(tally.success_count).unwrap_or(i64::MAX),
            errors: i64::try_from(tally.error_count).unwrap_or(i64::MAX),
            error_message: None,
            triggered_by: "local",
            duration_ms: Some(i64::try_from(duration_ms).unwrap_or(i64::MAX)),
        },
    )
    .await;

    tracing::info!(
        marketplace_id,
        plugins = tally.success_count,
        errors = tally.error_count,
        duration_ms,
        "Local marketplace sync completed"
    );

    Ok(SyncResult {
        commit_hash: String::new(),
        plugins_synced: tally.success_count,
        errors: tally.error_count,
        changed: true,
        duration_ms,
    })
}
