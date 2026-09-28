//! `MANIFEST.yaml`: what an export was taken from, so an import can say
//! where its archive came from and a reader can tell two exports apart.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::AdminResult;
use crate::repositories::sync::plane::SyncPlane;
use crate::repositories::sync::sources::build_sources;
use crate::repositories::sync::state::find_sync_state;

pub const MANIFEST_FORMAT: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestPlane {
    pub id: String,
    pub file: String,
    pub declared_hash: Option<String>,
    pub applied_hash: Option<String>,
    pub applied_mode: Option<String>,
    pub applied_at: Option<DateTime<Utc>>,
    pub row_count: usize,
}

/// The header of a configuration archive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveManifest {
    pub format: u32,
    pub release: String,
    pub exported_at: DateTime<Utc>,
    pub exported_by: String,
    pub base_tree_hash: Option<String>,
    pub composed_hash: Option<String>,
    #[serde(default)]
    pub planes: Vec<ManifestPlane>,
}

/// One plane's contribution to the manifest, taken as its export is rendered.
#[derive(Clone)]
pub struct ExportedPlane<'a> {
    pub plane: &'a dyn SyncPlane,
    pub row_count: usize,
}

impl std::fmt::Debug for ExportedPlane<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExportedPlane")
            .field("plane", &self.plane.id())
            .field("row_count", &self.row_count)
            .finish()
    }
}

pub async fn build_manifest(
    pool: &PgPool,
    actor: &str,
    exported: &[ExportedPlane<'_>],
) -> AdminResult<ArchiveManifest> {
    // Why: discard-ok: an unreadable profile or cache leaves the hashes blank
    let sources = build_sources()
        .inspect_err(|e| tracing::warn!(error = %e, "sync export: sources unreadable"))
        .ok();
    let mut planes = Vec::with_capacity(exported.len());
    for e in exported {
        let state = find_sync_state(pool, e.plane.id()).await?;
        planes.push(ManifestPlane {
            id: e.plane.id().to_owned(),
            file: e.plane.source_file().to_owned(),
            declared_hash: state.as_ref().map(|s| s.declared_hash.clone()),
            applied_hash: state.as_ref().and_then(|s| s.applied_hash.clone()),
            applied_mode: state.as_ref().and_then(|s| s.applied_mode.clone()),
            applied_at: state.as_ref().and_then(|s| s.applied_at),
            row_count: e.row_count,
        });
    }
    Ok(ArchiveManifest {
        format: MANIFEST_FORMAT,
        release: env!("CARGO_PKG_VERSION").to_owned(),
        exported_at: Utc::now(),
        exported_by: actor.to_owned(),
        base_tree_hash: sources.as_ref().and_then(|s| s.base.tree_hash.clone()),
        composed_hash: sources.and_then(|s| s.composed_hash),
        planes,
    })
}

pub fn render_manifest(manifest: &ArchiveManifest) -> AdminResult<String> {
    let body = serde_yaml::to_string(manifest)
        .map_err(|e| crate::error::AdminError::invalid("manifest could not be rendered", e))?;
    Ok(format!(
        "# Configuration archive — exported from the console.\n\
         # Each services/<file> is that plane's database rendered as its declaration.\n\
         {body}"
    ))
}

pub fn parse_manifest(text: &str) -> Result<ArchiveManifest, String> {
    serde_yaml::from_str(text).map_err(|e| e.to_string())
}
